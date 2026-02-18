use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{get, post},
    Router,
};
use blog::{
    db::{Comment, Database, Guestbook},
    i18n::{Lang, LangExtractor, Translations},
    post::{
        dedup_by_translation, get_available_translations, get_posts_by_category,
        get_posts_by_series, get_recent_posts, get_series, get_series_nav_info, load_posts, Post,
        PostType,
    },
    templates::{
        BlogTemplate, DiaryTemplate, ErrorTemplate, GuestbookTemplate, IndexTemplate, PostTemplate,
        ResumeTemplate, ReviewTemplate, SeriesDetailTemplate, SeriesTemplate,
    },
    AppState, Blog, SharedState,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::services::{ServeDir, ServeFile};
use tracing::info;

const SITE_URL: &str = "https://miniex.blog";
const SITE_DESCRIPTION: &str = "miniex의 개발 블로그 - Rust, JavaScript, App, Server";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let app_state: AppState = Arc::new(RwLock::new(Vec::new()));
    load_posts(Arc::clone(&app_state)).await?;

    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        // Ensure data directory exists
        std::fs::create_dir_all("./data").unwrap_or_default();
        "sqlite:./data/blog.db?mode=rwc".to_string()
    });
    let db = Database::new(&database_url).await?;

    let shared_state = SharedState {
        posts: app_state,
        db,
    };

    let app = create_router(shared_state);

    #[cfg(debug_assertions)]
    let app = add_live_reload(app);

    let address = if cfg!(debug_assertions) {
        "0.0.0.0:3000"
    } else {
        "0.0.0.0:80"
    };

    info!("Starting server on {}", address);
    let listener = tokio::net::TcpListener::bind(address).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

fn create_router(state: SharedState) -> Router {
    // Get resume route from environment variable or use default
    let resume_route = if cfg!(debug_assertions) {
        "/resume/ytm".to_string()
    } else {
        std::env::var("RESUME_TAG")
            .map(|tag| format!("/resume/{}", tag))
            .unwrap_or_else(|_| "/resume/ytm".to_string())
    };

    Router::new()
        .route("/", get(handle_index))
        .route("/blog", get(handle_blog))
        .route("/review", get(handle_review))
        .route("/diary", get(handle_diary))
        .route("/series", get(handle_series))
        .route("/series/:name", get(handle_series_detail))
        .route("/post/:id", get(handle_post))
        .route(&resume_route, get(handle_resume))
        .route("/guestbook", get(handle_guestbook))
        .route("/feed.xml", get(handle_feed))
        .route("/sitemap.xml", get(handle_sitemap))
        .route("/api/search", get(handle_search))
        .route("/api/set-lang", get(handle_set_lang))
        .route("/api/comments/:post_id", get(get_comments))
        .route("/api/comments", post(create_comment))
        .route(
            "/api/comments/edit/:comment_id",
            axum::routing::put(edit_comment),
        )
        .route(
            "/api/comments/delete/:comment_id",
            axum::routing::delete(delete_comment),
        )
        .route("/api/guestbook", get(get_guestbook_entries))
        .route("/api/guestbook", post(create_guestbook_entry))
        .route(
            "/api/guestbook/edit/:entry_id",
            axum::routing::put(edit_guestbook_entry),
        )
        .route(
            "/api/guestbook/delete/:entry_id",
            axum::routing::delete(delete_guestbook_entry),
        )
        .nest_service("/assets", ServeDir::new("assets"))
        .nest_service("/favicon.ico", ServeFile::new("assets/favicon/favicon.ico"))
        .nest_service("/robots.txt", ServeFile::new("assets/robots.txt"))
        .layer(tower_http::trace::TraceLayer::new_for_http())
        .fallback(handle_error)
        .with_state(state)
}

#[cfg(debug_assertions)]
fn add_live_reload(app: Router) -> Router {
    use notify::Watcher;
    let livereload = tower_livereload::LiveReloadLayer::new().request_predicate(
        |req: &axum::http::Request<axum::body::Body>| !req.headers().contains_key("hx-request"),
    );
    let reloader = livereload.reloader();
    let mut watcher = notify::recommended_watcher(move |_| reloader.reload()).unwrap();
    let paths = ["assets", "templates"];
    for path in paths.iter() {
        watcher
            .watch(std::path::Path::new(path), notify::RecursiveMode::Recursive)
            .unwrap();
    }
    app.layer(livereload)
}

async fn handle_index(
    State(state): State<SharedState>,
    LangExtractor(lang): LangExtractor,
) -> IndexTemplate {
    let posts = state.posts.read().await;
    let recent_posts = get_recent_posts(&posts, lang);
    let t = Translations::for_lang(lang);

    IndexTemplate {
        blog: Blog::new()
            .set_title("miniex")
            .set_description(SITE_DESCRIPTION)
            .set_url(SITE_URL),
        recent_posts,
        t,
        lang,
    }
}

#[derive(Deserialize)]
struct BlogQuery {
    category: Option<String>,
    page: Option<u32>,
    sort: Option<String>,
}

async fn handle_blog(
    State(state): State<SharedState>,
    LangExtractor(lang): LangExtractor,
    Query(query): Query<BlogQuery>,
) -> BlogTemplate {
    let posts = state.posts.read().await;
    let category = query.category.as_deref();
    let page = query.page.unwrap_or(1);
    let sort_asc = query.sort.as_deref() == Some("asc");
    let posts_per_page = 10;
    let t = Translations::for_lang(lang);

    let filtered_posts = get_posts_by_category(&posts, PostType::Blog, category, lang, sort_asc);
    let total_posts = filtered_posts.len();
    let total_pages = (total_posts as f32 / posts_per_page as f32).ceil() as u32;

    let start = (page - 1) * posts_per_page;
    let current_posts = filtered_posts
        .into_iter()
        .skip(start as usize)
        .take(posts_per_page as usize)
        .collect();

    let categories = posts
        .iter()
        .filter(|p| matches!(p.post_type, PostType::Blog))
        .flat_map(|p| p.metadata.tags.clone())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    let page_numbers = if total_pages <= 5 {
        (1..=total_pages).collect()
    } else {
        let start = std::cmp::max(1, std::cmp::min(page - 2, total_pages - 4));
        let end = std::cmp::min(start + 4, total_pages);
        (start..=end).collect()
    };

    BlogTemplate {
        blog: Blog::new()
            .set_title("miniex::blog")
            .set_description("miniex의 기술 블로그 글 목록")
            .set_url(&format!("{}/blog", SITE_URL)),
        posts: current_posts,
        categories,
        current_category: query.category,
        current_page: page,
        total_pages,
        prev_page: if page > 1 { Some(page - 1) } else { None },
        next_page: if page < total_pages {
            Some(page + 1)
        } else {
            None
        },
        page_numbers,
        t,
        lang,
        sort_asc,
    }
}

#[derive(Deserialize)]
struct ReviewQuery {
    category: Option<String>,
    page: Option<u32>,
    sort: Option<String>,
}

async fn handle_review(
    State(state): State<SharedState>,
    LangExtractor(lang): LangExtractor,
    Query(query): Query<ReviewQuery>,
) -> ReviewTemplate {
    let posts = state.posts.read().await;
    let category = query.category.as_deref();
    let page = query.page.unwrap_or(1);
    let sort_asc = query.sort.as_deref() == Some("asc");
    let posts_per_page = 10;
    let t = Translations::for_lang(lang);

    let filtered_posts = get_posts_by_category(&posts, PostType::Review, category, lang, sort_asc);
    let total_posts = filtered_posts.len();
    let total_pages = (total_posts as f32 / posts_per_page as f32).ceil() as u32;

    let start = (page - 1) * posts_per_page;
    let current_posts = filtered_posts
        .into_iter()
        .skip(start as usize)
        .take(posts_per_page as usize)
        .collect();

    let categories = posts
        .iter()
        .filter(|p| matches!(p.post_type, PostType::Review))
        .flat_map(|p| p.metadata.tags.clone())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    let page_numbers = if total_pages <= 5 {
        (1..=total_pages).collect()
    } else {
        let start = std::cmp::max(1, std::cmp::min(page - 2, total_pages - 4));
        let end = std::cmp::min(start + 4, total_pages);
        (start..=end).collect()
    };

    ReviewTemplate {
        blog: Blog::new()
            .set_title("miniex::review")
            .set_description("miniex의 리뷰 글 목록")
            .set_url(&format!("{}/review", SITE_URL)),
        posts: current_posts,
        categories,
        current_category: query.category,
        current_page: page,
        total_pages,
        prev_page: if page > 1 { Some(page - 1) } else { None },
        next_page: if page < total_pages {
            Some(page + 1)
        } else {
            None
        },
        page_numbers,
        t,
        lang,
        sort_asc,
    }
}

#[derive(Deserialize)]
struct DiaryQuery {
    category: Option<String>,
    page: Option<u32>,
    sort: Option<String>,
}

async fn handle_diary(
    State(state): State<SharedState>,
    LangExtractor(lang): LangExtractor,
    Query(query): Query<DiaryQuery>,
) -> DiaryTemplate {
    let posts = state.posts.read().await;
    let category = query.category.as_deref();
    let page = query.page.unwrap_or(1);
    let sort_asc = query.sort.as_deref() == Some("asc");
    let posts_per_page = 10;
    let t = Translations::for_lang(lang);

    let filtered_posts = get_posts_by_category(&posts, PostType::Diary, category, lang, sort_asc);
    let total_posts = filtered_posts.len();
    let total_pages = (total_posts as f32 / posts_per_page as f32).ceil() as u32;

    let start = (page - 1) * posts_per_page;
    let current_posts = filtered_posts
        .into_iter()
        .skip(start as usize)
        .take(posts_per_page as usize)
        .collect();

    let categories = posts
        .iter()
        .filter(|p| matches!(p.post_type, PostType::Diary))
        .flat_map(|p| p.metadata.tags.clone())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    let page_numbers = if total_pages <= 5 {
        (1..=total_pages).collect()
    } else {
        let start = std::cmp::max(1, std::cmp::min(page - 2, total_pages - 4));
        let end = std::cmp::min(start + 4, total_pages);
        (start..=end).collect()
    };

    DiaryTemplate {
        blog: Blog::new()
            .set_title("miniex::diary")
            .set_description("miniex의 일기 목록")
            .set_url(&format!("{}/diary", SITE_URL)),
        posts: current_posts,
        categories,
        current_category: query.category,
        current_page: page,
        total_pages,
        prev_page: if page > 1 { Some(page - 1) } else { None },
        next_page: if page < total_pages {
            Some(page + 1)
        } else {
            None
        },
        page_numbers,
        t,
        lang,
        sort_asc,
    }
}

#[derive(Deserialize)]
struct SeriesQuery {
    sort: Option<String>,
}

async fn handle_series(
    State(state): State<SharedState>,
    LangExtractor(lang): LangExtractor,
    Query(query): Query<SeriesQuery>,
) -> SeriesTemplate {
    let posts = state.posts.read().await;
    let sort_asc = query.sort.as_deref() == Some("asc");
    let series = get_series(&posts, lang, sort_asc);
    let t = Translations::for_lang(lang);

    SeriesTemplate {
        blog: Blog::new()
            .set_title("miniex::series")
            .set_description("miniex의 시리즈 목록")
            .set_url(&format!("{}/series", SITE_URL)),
        series,
        t,
        lang,
        sort_asc,
    }
}

#[derive(Deserialize)]
struct SeriesDetailQuery {
    sort: Option<String>,
}

async fn handle_series_detail(
    Path(series_name): Path<String>,
    State(state): State<SharedState>,
    LangExtractor(lang): LangExtractor,
    Query(query): Query<SeriesDetailQuery>,
) -> SeriesDetailTemplate {
    let posts = state.posts.read().await;
    let sort_asc = query.sort.as_deref() == Some("asc");
    let series_posts = get_posts_by_series(&posts, &series_name, lang, sort_asc);
    let t = Translations::for_lang(lang);

    let series = get_series(&posts, lang, false)
        .into_iter()
        .find(|s| s.name == series_name)
        .expect("Series should exist");

    SeriesDetailTemplate {
        blog: Blog::new()
            .set_title(&format!("miniex::series::{}", series_name))
            .set_description(
                &series
                    .description
                    .clone()
                    .unwrap_or_else(|| format!("{} 시리즈", series_name)),
            )
            .set_url(&format!("{}/series/{}", SITE_URL, series_name)),
        series_description: series.description,
        series_status: series.status,
        series_name,
        posts: series_posts,
        authors: series.authors,
        updated_at: series.updated_at,
        t,
        lang,
        sort_asc,
    }
}

async fn handle_post(
    Path(id): Path<String>,
    State(state): State<SharedState>,
    LangExtractor(lang): LangExtractor,
) -> PostTemplate {
    let posts = state.posts.read().await;
    let t = Translations::for_lang(lang);

    // 1. Try slug + lang match first
    let current_post = posts
        .iter()
        .find(|p| p.slug == id && p.lang == lang)
        .cloned()
        // 2. Fallback: slug-only match (any language)
        .or_else(|| posts.iter().find(|p| p.slug == id).cloned());

    let available_langs = current_post
        .as_ref()
        .map(|p| get_available_translations(&posts, &p.translation_key))
        .unwrap_or_default();

    let series_nav = current_post
        .as_ref()
        .and_then(|p| get_series_nav_info(&posts, p));

    let blog = if let Some(ref p) = current_post {
        Blog::new()
            .set_title(&p.metadata.title)
            .set_description(&p.metadata.description)
            .set_url(&format!("{}/post/{}", SITE_URL, p.slug))
            .set_og_type("article")
    } else {
        Blog::new().set_title("Post Not Found")
    };

    PostTemplate {
        blog,
        current_post,
        series_nav,
        t,
        lang,
        available_langs,
    }
}

async fn handle_resume(LangExtractor(lang): LangExtractor) -> Result<ResumeTemplate, StatusCode> {
    use gray_matter::{engine::YAML, Matter};
    use pulldown_cmark::{html, Options, Parser};
    use tokio::fs;

    // Read the resume.mdx file
    let content = fs::read_to_string("contents/resume.mdx")
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Parse front matter
    let matter = Matter::<YAML>::new();
    let parsed = matter.parse(&content);

    // Parse markdown to HTML
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_HEADING_ATTRIBUTES);
    let parser = Parser::new_ext(parsed.content.as_str(), options);

    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);

    // Get resume title from environment variable or use default
    let resume_title =
        std::env::var("RESUME_TITLE").unwrap_or_else(|_| "miniex::resume".to_string());

    let t = Translations::for_lang(lang);
    Ok(ResumeTemplate {
        blog: Blog::new().set_title(&resume_title),
        content: html_output,
        t,
        lang,
    })
}

#[derive(Deserialize)]
struct GuestbookQuery {
    sort: Option<String>,
}

async fn handle_guestbook(
    State(state): State<SharedState>,
    LangExtractor(lang): LangExtractor,
    Query(query): Query<GuestbookQuery>,
) -> Result<GuestbookTemplate, StatusCode> {
    let sort_asc = query.sort.as_deref() == Some("asc");
    let mut guestbook_entries = match state.db.get_guestbook_entries(Some(20)).await {
        Ok(entries) => entries,
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };
    if sort_asc {
        guestbook_entries.reverse();
    }
    let t = Translations::for_lang(lang);

    Ok(GuestbookTemplate {
        entries: guestbook_entries,
        t,
        lang,
        sort_asc,
    })
}

// --- Search API ---

#[derive(Deserialize)]
struct SearchQuery {
    q: String,
    lang: String,
}

#[derive(Serialize)]
struct SearchResult {
    slug: String,
    title: String,
    description: String,
    post_type: String,
    tags: Vec<String>,
    created_at: String,
    reading_time_min: u32,
    lang: String,
}

async fn handle_search(
    State(state): State<SharedState>,
    Query(query): Query<SearchQuery>,
) -> Json<Vec<SearchResult>> {
    let posts = state.posts.read().await;
    let search_lang = Lang::parse(&query.lang);
    let q = query.q.to_lowercase();

    let matching: Vec<Post> = posts
        .iter()
        .filter(|post| {
            post.metadata.title.to_lowercase().contains(&q)
                || post.metadata.description.to_lowercase().contains(&q)
                || post
                    .metadata
                    .tags
                    .iter()
                    .any(|tag| tag.to_lowercase().contains(&q))
        })
        .cloned()
        .collect();

    let deduped = dedup_by_translation(matching, search_lang);
    let mut sorted = deduped;
    sorted.sort_by(|a, b| b.metadata.created_at.cmp(&a.metadata.created_at));

    let results: Vec<SearchResult> = sorted
        .into_iter()
        .take(20)
        .map(|post| SearchResult {
            slug: post.slug.clone(),
            title: post.metadata.title.clone(),
            description: post.metadata.description.clone(),
            post_type: post.post_type.to_string().to_lowercase(),
            tags: post.metadata.tags.clone(),
            created_at: post.metadata.created_at.format("%Y/%m/%d").to_string(),
            reading_time_min: post.reading_time_min,
            lang: post.lang.as_str().to_string(),
        })
        .collect();

    Json(results)
}

// --- Language Switch ---

#[derive(Deserialize)]
struct SetLangQuery {
    lang: String,
}

async fn handle_set_lang(
    Query(query): Query<SetLangQuery>,
    headers: axum::http::HeaderMap,
) -> impl IntoResponse {
    let lang = Lang::parse(&query.lang);
    let cookie = format!(
        "lang={}; Path=/; Max-Age=31536000; SameSite=Lax",
        lang.as_str()
    );

    let referer = headers
        .get(axum::http::header::REFERER)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("/");

    (
        StatusCode::SEE_OTHER,
        [
            (axum::http::header::SET_COOKIE, cookie),
            (axum::http::header::LOCATION, referer.to_string()),
        ],
    )
}

// --- Atom Feed ---

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[derive(Deserialize)]
struct FeedQuery {
    lang: Option<String>,
}

async fn handle_feed(
    State(state): State<SharedState>,
    Query(query): Query<FeedQuery>,
) -> impl IntoResponse {
    let posts = state.posts.read().await;
    let lang_filter = query.lang.as_deref().map(Lang::parse);

    let mut recent_posts: Vec<&Post> = posts
        .iter()
        .filter(|p| lang_filter.map(|l| p.lang == l).unwrap_or(true))
        .collect();
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
        [("content-type", "application/atom+xml; charset=utf-8")],
        xml,
    )
}

async fn handle_sitemap(State(state): State<SharedState>) -> impl IntoResponse {
    let posts = state.posts.read().await;

    let mut xml = String::new();
    xml.push_str("<?xml version=\"1.0\" encoding=\"utf-8\"?>\n");
    xml.push_str("<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n");

    // Static pages
    for path in &["/", "/blog", "/review", "/diary", "/series", "/guestbook"] {
        xml.push_str("  <url>\n");
        xml.push_str(&format!("    <loc>{}{}</loc>\n", SITE_URL, path));
        xml.push_str("    <changefreq>weekly</changefreq>\n");
        xml.push_str("  </url>\n");
    }

    // Series pages
    let series_list = get_series(&posts, Lang::En, false);
    for s in &series_list {
        let lastmod = s.updated_at.format("%Y-%m-%d").to_string();
        xml.push_str("  <url>\n");
        xml.push_str(&format!("    <loc>{}/series/{}</loc>\n", SITE_URL, s.name));
        xml.push_str(&format!("    <lastmod>{}</lastmod>\n", lastmod));
        xml.push_str("    <changefreq>weekly</changefreq>\n");
        xml.push_str("  </url>\n");
    }

    // Post pages
    let mut sorted_posts: Vec<&Post> = posts.iter().collect();
    sorted_posts.sort_by(|a, b| b.metadata.updated_at.cmp(&a.metadata.updated_at));

    for post in &sorted_posts {
        let lastmod = post.metadata.updated_at.format("%Y-%m-%d").to_string();
        xml.push_str("  <url>\n");
        xml.push_str(&format!("    <loc>{}/post/{}</loc>\n", SITE_URL, post.slug));
        xml.push_str(&format!("    <lastmod>{}</lastmod>\n", lastmod));
        xml.push_str("    <changefreq>monthly</changefreq>\n");
        xml.push_str("  </url>\n");
    }

    xml.push_str("</urlset>\n");

    (
        StatusCode::OK,
        [("content-type", "application/xml; charset=utf-8")],
        xml,
    )
}

// --- Comments & Guestbook API ---

#[derive(Deserialize)]
struct CreateCommentRequest {
    author: String,
    content: String,
    password: Option<String>,
}

#[derive(Serialize)]
struct ApiResponse<T> {
    data: T,
    message: String,
}

async fn get_comments(
    Path(post_id): Path<String>,
    State(state): State<SharedState>,
) -> Result<Json<ApiResponse<Vec<Comment>>>, StatusCode> {
    match state.db.get_comments_by_post(&post_id).await {
        Ok(comments) => Ok(Json(ApiResponse {
            data: comments,
            message: "Comments retrieved successfully".to_string(),
        })),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[derive(Deserialize)]
struct CreateCommentWithPostRequest {
    post_id: String,
    author: String,
    content: String,
    password: Option<String>,
}

#[derive(Deserialize)]
struct EditCommentRequest {
    content: String,
    password: String,
}

#[derive(Deserialize)]
struct DeleteRequest {
    password: String,
}

#[derive(Serialize)]
struct EditResponse {
    success: bool,
    message: String,
}

async fn create_comment(
    State(state): State<SharedState>,
    Json(payload): Json<CreateCommentWithPostRequest>,
) -> Result<Json<ApiResponse<Comment>>, StatusCode> {
    match state
        .db
        .create_comment(
            &payload.post_id,
            &payload.author,
            &payload.content,
            payload.password.as_deref(),
        )
        .await
    {
        Ok(comment) => Ok(Json(ApiResponse {
            data: comment,
            message: "Comment created successfully".to_string(),
        })),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn edit_comment(
    Path(comment_id): Path<String>,
    State(state): State<SharedState>,
    Json(payload): Json<EditCommentRequest>,
) -> Result<Json<EditResponse>, StatusCode> {
    match state
        .db
        .update_comment(&comment_id, &payload.content, &payload.password)
        .await
    {
        Ok(success) => Ok(Json(EditResponse {
            success,
            message: if success {
                "Comment updated successfully".to_string()
            } else {
                "Wrong password or comment not found".to_string()
            },
        })),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn delete_comment(
    Path(comment_id): Path<String>,
    State(state): State<SharedState>,
    Json(payload): Json<DeleteRequest>,
) -> Result<Json<EditResponse>, StatusCode> {
    match state
        .db
        .delete_comment(&comment_id, &payload.password)
        .await
    {
        Ok(success) => Ok(Json(EditResponse {
            success,
            message: if success {
                "Comment deleted successfully".to_string()
            } else {
                "Wrong password or comment not found".to_string()
            },
        })),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn get_guestbook_entries(
    State(state): State<SharedState>,
) -> Result<Json<ApiResponse<Vec<Guestbook>>>, StatusCode> {
    match state.db.get_guestbook_entries(Some(50)).await {
        Ok(entries) => Ok(Json(ApiResponse {
            data: entries,
            message: "Guestbook entries retrieved successfully".to_string(),
        })),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn create_guestbook_entry(
    State(state): State<SharedState>,
    Json(payload): Json<CreateCommentRequest>,
) -> Result<Json<ApiResponse<Guestbook>>, StatusCode> {
    match state
        .db
        .create_guestbook_entry(
            &payload.author,
            &payload.content,
            payload.password.as_deref(),
        )
        .await
    {
        Ok(entry) => Ok(Json(ApiResponse {
            data: entry,
            message: "Guestbook entry created successfully".to_string(),
        })),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn edit_guestbook_entry(
    Path(entry_id): Path<String>,
    State(state): State<SharedState>,
    Json(payload): Json<EditCommentRequest>,
) -> Result<Json<EditResponse>, StatusCode> {
    match state
        .db
        .update_guestbook_entry(&entry_id, &payload.content, &payload.password)
        .await
    {
        Ok(success) => Ok(Json(EditResponse {
            success,
            message: if success {
                "Guestbook entry updated successfully".to_string()
            } else {
                "Wrong password or entry not found".to_string()
            },
        })),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn delete_guestbook_entry(
    Path(entry_id): Path<String>,
    State(state): State<SharedState>,
    Json(payload): Json<DeleteRequest>,
) -> Result<Json<EditResponse>, StatusCode> {
    match state
        .db
        .delete_guestbook_entry(&entry_id, &payload.password)
        .await
    {
        Ok(success) => Ok(Json(EditResponse {
            success,
            message: if success {
                "Guestbook entry deleted successfully".to_string()
            } else {
                "Wrong password or entry not found".to_string()
            },
        })),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn handle_error(LangExtractor(lang): LangExtractor) -> ErrorTemplate {
    let t = Translations::for_lang(lang);
    ErrorTemplate { t, lang }
}
