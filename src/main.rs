use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use blog::{
    db::{Comment, Database, Guestbook},
    post::{
        get_posts_by_category, get_posts_by_series, get_recent_posts, get_series, load_posts,
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let app_state: AppState = Arc::new(RwLock::new(Vec::new()));
    load_posts(Arc::clone(&app_state)).await?;

    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        // Ensure data directory exists
        std::fs::create_dir_all("./data").unwrap_or_default();
        "sqlite:./data/blog.db".to_string()
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
        "/resume".to_string()
    } else {
        std::env::var("RESUME_TAG")
            .map(|tag| format!("/resume/{}", tag))
            .unwrap_or_else(|_| "/resume".to_string())
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
        .nest_service("/js", ServeDir::new("js"))
        .nest_service("/favicon.ico", ServeFile::new("assets/favicon/favicon.ico"))
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
    let paths = ["assets", "templates", "js"];
    for path in paths.iter() {
        watcher
            .watch(std::path::Path::new(path), notify::RecursiveMode::Recursive)
            .unwrap();
    }
    app.layer(livereload)
}

async fn handle_index(State(state): State<SharedState>) -> IndexTemplate {
    let posts = state.posts.read().await;
    let recent_posts = get_recent_posts(&posts);

    IndexTemplate {
        blog: Blog::new().set_title("miniex"),
        recent_posts,
    }
}

#[derive(Deserialize)]
struct BlogQuery {
    category: Option<String>,
    page: Option<u32>,
}

async fn handle_blog(
    State(state): State<SharedState>,
    Query(query): Query<BlogQuery>,
) -> BlogTemplate {
    let posts = state.posts.read().await;
    let category = query.category.as_deref();
    let page = query.page.unwrap_or(1);
    let posts_per_page = 10;

    let filtered_posts = get_posts_by_category(&posts, PostType::Blog, category);
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
        blog: Blog::new().set_title("miniex::blog"),
        posts: current_posts,
        categories,
        current_page: page,
        total_pages,
        prev_page: if page > 1 { Some(page - 1) } else { None },
        next_page: if page < total_pages {
            Some(page + 1)
        } else {
            None
        },
        page_numbers,
    }
}

#[derive(Deserialize)]
struct ReviewQuery {
    category: Option<String>,
    page: Option<u32>,
}

async fn handle_review(
    State(state): State<SharedState>,
    Query(query): Query<ReviewQuery>,
) -> ReviewTemplate {
    let posts = state.posts.read().await;
    let category = query.category.as_deref();
    let page = query.page.unwrap_or(1);
    let posts_per_page = 10;

    let filtered_posts = get_posts_by_category(&posts, PostType::Review, category);
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
        blog: Blog::new().set_title("miniex::review"),
        posts: current_posts,
        categories,
        current_page: page,
        total_pages,
        prev_page: if page > 1 { Some(page - 1) } else { None },
        next_page: if page < total_pages {
            Some(page + 1)
        } else {
            None
        },
        page_numbers,
    }
}

#[derive(Deserialize)]
struct DiaryQuery {
    category: Option<String>,
    page: Option<u32>,
}

async fn handle_diary(
    State(state): State<SharedState>,
    Query(query): Query<DiaryQuery>,
) -> DiaryTemplate {
    let posts = state.posts.read().await;
    let category = query.category.as_deref();
    let page = query.page.unwrap_or(1);
    let posts_per_page = 10;

    let filtered_posts = get_posts_by_category(&posts, PostType::Diary, category);
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
        blog: Blog::new().set_title("miniex::diary"),
        posts: current_posts,
        categories,
        current_page: page,
        total_pages,
        prev_page: if page > 1 { Some(page - 1) } else { None },
        next_page: if page < total_pages {
            Some(page + 1)
        } else {
            None
        },
        page_numbers,
    }
}

async fn handle_series(State(state): State<SharedState>) -> SeriesTemplate {
    let posts = state.posts.read().await;
    let series = get_series(&posts);

    SeriesTemplate {
        blog: Blog::new().set_title("miniex::series"),
        series,
    }
}

async fn handle_series_detail(
    Path(series_name): Path<String>,
    State(state): State<SharedState>,
) -> SeriesDetailTemplate {
    let posts = state.posts.read().await;
    let series_posts = get_posts_by_series(&posts, &series_name);

    let series = get_series(&posts)
        .into_iter()
        .find(|s| s.name == series_name)
        .expect("Series should exist");

    SeriesDetailTemplate {
        blog: Blog::new().set_title(&format!("miniex::series::{}", series_name)),
        series_name,
        posts: series_posts,
        authors: series.authors,
        updated_at: series.updated_at,
    }
}

async fn handle_post(Path(id): Path<String>, State(state): State<SharedState>) -> PostTemplate {
    let posts = state.posts.read().await;
    let current_post = posts.iter().find(|p| p.slug == id).cloned();
    PostTemplate { current_post }
}

async fn handle_resume() -> Result<ResumeTemplate, StatusCode> {
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
    let parser = Parser::new_ext(parsed.content.as_str(), options);

    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);

    Ok(ResumeTemplate {
        blog: Blog::new().set_title("miniex::resume"),
        content: html_output,
    })
}

async fn handle_guestbook(
    State(state): State<SharedState>,
) -> Result<GuestbookTemplate, StatusCode> {
    let guestbook_entries = match state.db.get_guestbook_entries(Some(20)).await {
        Ok(entries) => entries,
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    Ok(GuestbookTemplate {
        entries: guestbook_entries,
    })
}

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
) -> Result<Json<ApiResponse<()>>, StatusCode> {
    match state.db.delete_comment(&comment_id).await {
        Ok(_) => Ok(Json(ApiResponse {
            data: (),
            message: "Comment deleted successfully".to_string(),
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
) -> Result<Json<ApiResponse<()>>, StatusCode> {
    match state.db.delete_guestbook_entry(&entry_id).await {
        Ok(_) => Ok(Json(ApiResponse {
            data: (),
            message: "Guestbook entry deleted successfully".to_string(),
        })),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn handle_error() -> ErrorTemplate {
    ErrorTemplate {}
}
