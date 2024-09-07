use axum::{
    extract::{Path, State},
    routing::get,
    Router,
};
use blog::{
    post::{get_recent_posts, load_posts},
    templates::{
        BlogTemplate, DiaryTemplate, ErrorTemplate, IndexTemplate, PostTemplate, ReviewTemplate,
    },
    AppState, Blog,
};
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::services::{ServeDir, ServeFile};
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let app_state: AppState = Arc::new(RwLock::new(Vec::new()));
    load_posts(Arc::clone(&app_state)).await?;

    let app = create_router(app_state);

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

fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/", get(handle_index))
        .route("/blog", get(handle_blog))
        .route("/review", get(handle_review))
        .route("/diary", get(handle_diary))
        .route("/post/:id", get(handle_post))
        .nest_service("/assets", ServeDir::new("assets"))
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
    let paths = ["assets", "templates"];
    for path in paths.iter() {
        watcher
            .watch(std::path::Path::new(path), notify::RecursiveMode::Recursive)
            .unwrap();
    }
    app.layer(livereload)
}

async fn handle_index(State(state): State<AppState>) -> IndexTemplate {
    let posts = state.read().await;
    let recent_posts = get_recent_posts(&posts);

    IndexTemplate {
        blog: Blog::new().set_title("miniex"),
        recent_posts,
    }
}

async fn handle_blog() -> BlogTemplate {
    BlogTemplate {
        blog: Blog::new().set_title("miniex::blog"),
    }
}

async fn handle_review() -> ReviewTemplate {
    ReviewTemplate {
        blog: Blog::new().set_title("miniex::review"),
    }
}

async fn handle_diary() -> DiaryTemplate {
    DiaryTemplate {
        blog: Blog::new().set_title("miniex::diary"),
    }
}

async fn handle_post(Path(id): Path<String>, State(state): State<AppState>) -> PostTemplate {
    let posts = state.read().await;
    let post = posts
        .iter()
        .find(|p| p.metadata.title.to_lowercase().replace(" ", "-") == id)
        .cloned();
    PostTemplate { post }
}

async fn handle_error() -> ErrorTemplate {
    ErrorTemplate {}
}
