use axum::{routing::get, Router};
use blog::{
    templates::{BlogTemplate, IndexTemplate},
    Blog,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let app = Router::new()
        .route("/", get(index))
        .route("/blog", get(blog))
        .nest_service("/assets", tower_http::services::ServeDir::new("assets"))
        .nest_service(
            "/favicon.ico",
            tower_http::services::ServeFile::new("assets/favicon/favicon.ico"),
        )
        .layer(tower_http::trace::TraceLayer::new_for_http());

    #[cfg(debug_assertions)]
    let app = {
        use notify::Watcher;
        let livereload = tower_livereload::LiveReloadLayer::new().request_predicate(
            |req: &axum::http::Request<axum::body::Body>| !req.headers().contains_key("hx-request"),
        );
        let reloader = livereload.reloader();
        let mut watcher = notify::recommended_watcher(move |_| reloader.reload()).unwrap();
        watcher
            .watch(
                std::path::Path::new("assets"),
                notify::RecursiveMode::Recursive,
            )
            .unwrap();
        watcher
            .watch(
                std::path::Path::new("templates"),
                notify::RecursiveMode::Recursive,
            )
            .unwrap();
        app.layer(livereload)
    };

    #[cfg(not(debug_assertions))]
    let address = "127.0.0.1:80";
    #[cfg(debug_assertions)]
    let address = "127.0.0.1:3000";

    let listener = tokio::net::TcpListener::bind(address).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn index() -> IndexTemplate {
    let blog = Blog::new().set_title("miniex");

    IndexTemplate { blog }
}

async fn blog() -> BlogTemplate {
    let blog = Blog::new().set_title("miniex::blog");

    BlogTemplate { blog }
}
