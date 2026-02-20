use crate::{
    i18n::{LangExtractor, Translations},
    post::{
        get_available_translations, get_posts_by_category, get_posts_by_series, get_recent_posts,
        get_series_nav_info, PostType,
    },
    templates::{
        BlogTemplate, DiaryTemplate, ErrorTemplate, GuestbookTemplate, IndexTemplate, PostTemplate,
        ResumeTemplate, ReviewTemplate, SeriesDetailTemplate, SeriesTemplate,
    },
    Blog, SharedState, SITE_DESCRIPTION, SITE_URL,
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::Deserialize;

fn compute_page_numbers(current: u32, total: u32) -> Vec<u32> {
    if total <= 5 {
        (1..=total).collect()
    } else {
        let start = std::cmp::max(
            1,
            std::cmp::min(current.saturating_sub(2), total.saturating_sub(4)),
        );
        let end = std::cmp::min(start + 4, total);
        (start..=end).collect()
    }
}

pub async fn handle_index(
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
pub struct BlogQuery {
    category: Option<String>,
    page: Option<u32>,
    sort: Option<String>,
}

pub async fn handle_blog(
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

    let page_numbers = compute_page_numbers(page, total_pages);

    BlogTemplate {
        blog: Blog::new()
            .set_title("miniex::blog")
            .set_description(
                "Technical blog posts about Rust, JavaScript, web development, and more",
            )
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
pub struct ReviewQuery {
    category: Option<String>,
    page: Option<u32>,
    sort: Option<String>,
}

pub async fn handle_review(
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

    let page_numbers = compute_page_numbers(page, total_pages);

    ReviewTemplate {
        blog: Blog::new()
            .set_title("miniex::review")
            .set_description("Tech reviews and software analysis")
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
pub struct DiaryQuery {
    category: Option<String>,
    page: Option<u32>,
    sort: Option<String>,
}

pub async fn handle_diary(
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

    let page_numbers = compute_page_numbers(page, total_pages);

    DiaryTemplate {
        blog: Blog::new()
            .set_title("miniex::diary")
            .set_description("Development diary and personal notes")
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
pub struct SeriesQuery {
    sort: Option<String>,
}

pub async fn handle_series(
    State(state): State<SharedState>,
    LangExtractor(lang): LangExtractor,
    Query(query): Query<SeriesQuery>,
) -> SeriesTemplate {
    let sort_asc = query.sort.as_deref() == Some("asc");
    let mut series = state.series_cache.read().await.clone();
    if sort_asc {
        series.sort_by(|a, b| a.updated_at.cmp(&b.updated_at));
    }
    let t = Translations::for_lang(lang);

    SeriesTemplate {
        blog: Blog::new()
            .set_title("miniex::series")
            .set_description("Development tutorial series and in-depth guides")
            .set_url(&format!("{}/series", SITE_URL)),
        series,
        t,
        lang,
        sort_asc,
    }
}

#[derive(Deserialize)]
pub struct SeriesDetailQuery {
    sort: Option<String>,
    page: Option<u32>,
}

pub async fn handle_series_detail(
    Path(series_name): Path<String>,
    State(state): State<SharedState>,
    LangExtractor(lang): LangExtractor,
    Query(query): Query<SeriesDetailQuery>,
) -> impl IntoResponse {
    let sort_asc = query.sort.as_deref() == Some("asc");
    let page = query.page.unwrap_or(1);
    let posts_per_page: u32 = 6;
    let t = Translations::for_lang(lang);

    let series = state
        .series_cache
        .read()
        .await
        .iter()
        .find(|s| s.name == series_name)
        .cloned();

    match series {
        Some(series) => {
            let posts = state.posts.read().await;
            let all_series_posts = get_posts_by_series(&posts, &series_name, lang, sort_asc);
            let total_posts = all_series_posts.len();
            let total_pages = (total_posts as f32 / posts_per_page as f32).ceil() as u32;

            let start = ((page - 1) * posts_per_page) as usize;
            let series_posts: Vec<_> = all_series_posts
                .into_iter()
                .skip(start)
                .take(posts_per_page as usize)
                .collect();

            let page_numbers = compute_page_numbers(page, total_pages);

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
                current_page: page,
                total_pages,
                prev_page: if page > 1 { Some(page - 1) } else { None },
                next_page: if page < total_pages {
                    Some(page + 1)
                } else {
                    None
                },
                page_numbers,
                total_posts: total_posts as u32,
            }
            .into_response()
        }
        None => (StatusCode::NOT_FOUND, ErrorTemplate { t, lang }).into_response(),
    }
}

pub async fn handle_post(
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

pub async fn handle_resume(
    LangExtractor(lang): LangExtractor,
) -> Result<ResumeTemplate, StatusCode> {
    use gray_matter::{engine::YAML, Matter};
    use pulldown_cmark::{html, Options, Parser};
    use tokio::fs;

    let content = fs::read_to_string("contents/resume.mdx")
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let matter = Matter::<YAML>::new();
    let parsed = matter.parse(&content);

    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_HEADING_ATTRIBUTES);
    let parser = Parser::new_ext(parsed.content.as_str(), options);

    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);

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
pub struct GuestbookQuery {
    sort: Option<String>,
    page: Option<u32>,
}

pub async fn handle_guestbook(
    State(state): State<SharedState>,
    LangExtractor(lang): LangExtractor,
    Query(query): Query<GuestbookQuery>,
) -> Result<GuestbookTemplate, StatusCode> {
    let sort_asc = query.sort.as_deref() == Some("asc");
    let page = query.page.unwrap_or(1);
    let per_page: u32 = 10;
    let t = Translations::for_lang(lang);

    let total_entries = state
        .db
        .count_guestbook_entries()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let total_pages = if total_entries == 0 {
        1
    } else {
        (total_entries as f32 / per_page as f32).ceil() as u32
    };

    let offset = ((page - 1) * per_page) as i32;
    let guestbook_entries = state
        .db
        .get_guestbook_entries_paged(offset, per_page as i32, sort_asc)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let page_numbers = compute_page_numbers(page, total_pages);

    Ok(GuestbookTemplate {
        entries: guestbook_entries,
        t,
        lang,
        sort_asc,
        current_page: page,
        total_pages,
        prev_page: if page > 1 { Some(page - 1) } else { None },
        next_page: if page < total_pages {
            Some(page + 1)
        } else {
            None
        },
        page_numbers,
        total_entries,
    })
}

pub async fn handle_error(LangExtractor(lang): LangExtractor) -> ErrorTemplate {
    let t = Translations::for_lang(lang);
    ErrorTemplate { t, lang }
}
