use axum::{
    extract::{Path, State},
    routing::get,
    Router,
};
use blog::{
    post::{Post, PostMetadata, PostType},
    templates::{BlogTemplate, ErrorTemplate, IndexTemplate, PostTemplate},
    Blog,
};
use gray_matter::{engine::YAML, Matter};
use std::{path::PathBuf, sync::Arc};
use tokio::sync::RwLock;
use tower_http::services::{ServeDir, ServeFile};
use tracing::info;

type AppState = Arc<RwLock<Vec<Post>>>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let app_state = Arc::new(RwLock::new(Vec::new()));
    load_posts(Arc::clone(&app_state)).await;

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
        .route("/post/:name", get(handle_post))
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

async fn handle_index() -> IndexTemplate {
    IndexTemplate {
        blog: Blog::new().set_title("miniex"),
    }
}

async fn handle_blog() -> BlogTemplate {
    BlogTemplate {
        blog: Blog::new().set_title("miniex::blog"),
    }
}

async fn handle_post(Path(name): Path<String>, State(state): State<AppState>) -> PostTemplate {
    let posts = state.read().await;
    let post = posts
        .iter()
        .find(|p| p.metadata.title.to_lowercase().replace(" ", "-") == name)
        .cloned();
    PostTemplate { post }
}

async fn handle_error() -> ErrorTemplate {
    ErrorTemplate {}
}

async fn load_posts(state: AppState) {
    let matter = Matter::<YAML>::new();
    let content_dir = PathBuf::from("contents");
    for entry in std::fs::read_dir(content_dir)
        .expect("Failed to read contents directory")
        .flatten()
    {
        let path = entry.path();
        if path.is_dir() {
            if let Some(post_type) = get_post_type(&path) {
                process_directory(&path, post_type, &matter, &state).await;
            }
        }
    }
}

fn get_post_type(path: &std::path::Path) -> Option<PostType> {
    path.file_name()?
        .to_str()
        .and_then(|s| match s.to_lowercase().as_str() {
            "blog" => Some(PostType::Blog),
            "review" => Some(PostType::Review),
            "diary" => Some(PostType::Diary),
            _ => None,
        })
}

async fn process_directory(
    path: &PathBuf,
    post_type: PostType,
    matter: &Matter<YAML>,
    state: &AppState,
) {
    for file in std::fs::read_dir(path)
        .expect("Failed to read directory")
        .flatten()
    {
        let file_path = file.path();
        if file_path.extension().unwrap_or_default() == "mdx" {
            if let Some(post) = process_file(&file_path, post_type.clone(), matter) {
                state.write().await.push(post);
            }
        }
    }
}

fn process_file(file_path: &PathBuf, post_type: PostType, matter: &Matter<YAML>) -> Option<Post> {
    let content = std::fs::read_to_string(file_path).ok()?;
    let parsed = matter.parse(&content);

    parsed.data.map(|data| {
        let metadata: PostMetadata = data.deserialize().expect("Failed to deserialize metadata");

        Post {
            post_type,
            metadata,
            content: parsed.content,
        }
    })
}
