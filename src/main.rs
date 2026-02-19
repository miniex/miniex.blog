use blog::{
    db::Database,
    i18n::Lang,
    post::{get_series, load_posts},
    router::create_router,
    SharedState,
};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let app_state = Arc::new(RwLock::new(Vec::new()));
    load_posts(Arc::clone(&app_state)).await?;

    // Pre-compute series cache from loaded posts
    let series_cache = {
        let posts = app_state.read().await;
        get_series(&posts, Lang::En, false)
    };

    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        std::fs::create_dir_all("./data").unwrap_or_default();
        "sqlite:./data/blog.db?mode=rwc".to_string()
    });
    let db = Database::new(&database_url).await?;

    let shared_state = SharedState {
        posts: app_state,
        db,
        series_cache: Arc::new(RwLock::new(series_cache)),
    };

    let app = create_router(shared_state.clone());

    #[cfg(debug_assertions)]
    let app = blog::router::add_live_reload(app, shared_state);

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
