use crate::{
    post::{dedup_refs_by_translation, Post},
    SharedState, SITE_DESCRIPTION, SITE_URL,
};
use axum::{
    extract::{Query, State},
    http::{header, HeaderMap, StatusCode},
    response::IntoResponse,
};
use serde::Deserialize;

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[derive(Deserialize)]
pub struct FeedQuery {
    lang: Option<String>,
}

pub async fn handle_feed(
    State(state): State<SharedState>,
    Query(query): Query<FeedQuery>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let posts = state.posts.read().await;
    let lang_filter = query.lang.as_deref().map(crate::i18n::Lang::parse);
    let dedup_lang = lang_filter.unwrap_or(crate::i18n::Lang::En);

    let mut recent_posts = dedup_refs_by_translation(
        posts
            .iter()
            .filter(|p| lang_filter.map(|l| p.lang == l).unwrap_or(true)),
        dedup_lang,
    );
    recent_posts.sort_by(|a, b| b.metadata.created_at.cmp(&a.metadata.created_at));
    let recent_posts: Vec<_> = recent_posts.into_iter().take(20).collect();

    let updated = recent_posts
        .first()
        .map(|p| {
            p.metadata
                .updated_at
                .format("%Y-%m-%dT%H:%M:%SZ")
                .to_string()
        })
        .unwrap_or_else(|| "2024-01-01T00:00:00Z".to_string());

    // ETag based on latest updated_at timestamp
    let etag = recent_posts
        .first()
        .map(|p| format!("\"feed-{}\"", p.metadata.updated_at.timestamp()))
        .unwrap_or_else(|| "\"feed-0\"".to_string());

    if let Some(if_none_match) = headers.get(header::IF_NONE_MATCH) {
        if if_none_match.to_str().ok() == Some(&etag) {
            return StatusCode::NOT_MODIFIED.into_response();
        }
    }

    let mut xml = String::new();
    xml.push_str("<?xml version=\"1.0\" encoding=\"utf-8\"?>\n");
    xml.push_str("<feed xmlns=\"http://www.w3.org/2005/Atom\">\n");
    xml.push_str(&format!(
        "  <title>{}</title>\n",
        html_escape("miniex.blog")
    ));
    xml.push_str(&format!(
        "  <subtitle>{}</subtitle>\n",
        html_escape(SITE_DESCRIPTION)
    ));
    xml.push_str(&format!(
        "  <link href=\"{}/feed.xml\" rel=\"self\" type=\"application/atom+xml\"/>\n",
        SITE_URL
    ));
    xml.push_str(&format!(
        "  <link href=\"{}\" rel=\"alternate\" type=\"text/html\"/>\n",
        SITE_URL
    ));
    xml.push_str(&format!("  <id>{}/</id>\n", SITE_URL));
    xml.push_str(&format!("  <updated>{}</updated>\n", updated));
    xml.push_str("  <author>\n");
    xml.push_str("    <name>Han Damin</name>\n");
    xml.push_str("  </author>\n");

    for post in &recent_posts {
        let post_url = format!("{}/post/{}", SITE_URL, post.slug);
        let published = post
            .metadata
            .created_at
            .format("%Y-%m-%dT%H:%M:%SZ")
            .to_string();
        let post_updated = post
            .metadata
            .updated_at
            .format("%Y-%m-%dT%H:%M:%SZ")
            .to_string();

        xml.push_str("  <entry>\n");
        xml.push_str(&format!(
            "    <title>{}</title>\n",
            html_escape(&post.metadata.title)
        ));
        xml.push_str(&format!(
            "    <link href=\"{}\" rel=\"alternate\" type=\"text/html\"/>\n",
            post_url
        ));
        xml.push_str(&format!("    <id>{}</id>\n", post_url));
        xml.push_str(&format!("    <published>{}</published>\n", published));
        xml.push_str(&format!("    <updated>{}</updated>\n", post_updated));
        xml.push_str("    <author>\n");
        xml.push_str(&format!(
            "      <name>{}</name>\n",
            html_escape(&post.metadata.author)
        ));
        xml.push_str("    </author>\n");
        xml.push_str(&format!(
            "    <summary>{}</summary>\n",
            html_escape(&post.metadata.description)
        ));
        xml.push_str(&format!(
            "    <category term=\"{}\"/>\n",
            html_escape(&post.post_type.to_string().to_lowercase())
        ));
        for tag in &post.metadata.tags {
            xml.push_str(&format!("    <category term=\"{}\"/>\n", html_escape(tag)));
        }
        xml.push_str("  </entry>\n");
    }

    xml.push_str("</feed>\n");

    (
        StatusCode::OK,
        [
            (
                header::CONTENT_TYPE,
                "application/atom+xml; charset=utf-8".to_string(),
            ),
            (header::ETAG, etag),
            (header::CACHE_CONTROL, "public, max-age=3600".to_string()),
        ],
        xml,
    )
        .into_response()
}

pub async fn handle_sitemap(
    State(state): State<SharedState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let posts = state.posts.read().await;

    // ETag based on latest updated_at timestamp across all posts
    let latest_ts = posts
        .iter()
        .map(|p| p.metadata.updated_at.timestamp())
        .max()
        .unwrap_or(0);
    let etag = format!("\"sitemap-{}\"", latest_ts);

    if let Some(if_none_match) = headers.get(header::IF_NONE_MATCH) {
        if if_none_match.to_str().ok() == Some(&etag) {
            return StatusCode::NOT_MODIFIED.into_response();
        }
    }

    let mut xml = String::new();
    xml.push_str("<?xml version=\"1.0\" encoding=\"utf-8\"?>\n");
    xml.push_str("<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\"\n");
    xml.push_str("        xmlns:xhtml=\"http://www.w3.org/1999/xhtml\">\n");

    // Static pages
    for path in &["/", "/blog", "/review", "/diary", "/series", "/guestbook"] {
        xml.push_str("  <url>\n");
        xml.push_str(&format!("    <loc>{}{}</loc>\n", SITE_URL, path));
        xml.push_str("    <changefreq>weekly</changefreq>\n");
        xml.push_str("  </url>\n");
    }

    // Series pages
    {
        let series_list = state.series_cache.read().await;
        for s in series_list.iter() {
            let lastmod = s.updated_at.format("%Y-%m-%d").to_string();
            xml.push_str("  <url>\n");
            xml.push_str(&format!("    <loc>{}/series/{}</loc>\n", SITE_URL, s.name));
            xml.push_str(&format!("    <lastmod>{}</lastmod>\n", lastmod));
            xml.push_str("    <changefreq>weekly</changefreq>\n");
            xml.push_str("  </url>\n");
        }
    }

    // Build translation groups for hreflang
    let mut translation_groups: std::collections::HashMap<&str, Vec<&Post>> =
        std::collections::HashMap::new();
    for post in posts.iter() {
        translation_groups
            .entry(&post.translation_key)
            .or_default()
            .push(post);
    }

    // Post pages (deduplicated by translation_key for canonical URLs)
    let mut sorted_posts: Vec<&Post> = posts.iter().collect();
    sorted_posts.sort_by(|a, b| b.metadata.updated_at.cmp(&a.metadata.updated_at));

    let mut seen_keys = std::collections::HashSet::new();
    for post in &sorted_posts {
        if !seen_keys.insert(&post.translation_key) {
            continue;
        }
        let lastmod = post.metadata.updated_at.format("%Y-%m-%d").to_string();
        xml.push_str("  <url>\n");
        xml.push_str(&format!("    <loc>{}/post/{}</loc>\n", SITE_URL, post.slug));
        xml.push_str(&format!("    <lastmod>{}</lastmod>\n", lastmod));
        xml.push_str("    <changefreq>monthly</changefreq>\n");

        // Add hreflang alternates for multilingual posts
        if let Some(translations) = translation_groups.get(post.translation_key.as_str()) {
            if translations.len() > 1 {
                for t_post in translations {
                    xml.push_str(&format!(
                        "    <xhtml:link rel=\"alternate\" hreflang=\"{}\" href=\"{}/post/{}\"/>\n",
                        t_post.lang.as_str(),
                        SITE_URL,
                        t_post.slug
                    ));
                }
            }
        }

        xml.push_str("  </url>\n");
    }

    xml.push_str("</urlset>\n");

    (
        StatusCode::OK,
        [
            (
                header::CONTENT_TYPE,
                "application/xml; charset=utf-8".to_string(),
            ),
            (header::ETAG, etag),
            (header::CACHE_CONTROL, "public, max-age=3600".to_string()),
        ],
        xml,
    )
        .into_response()
}
