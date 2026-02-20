use crate::{
    handlers::{api, feed, pages},
    SharedState,
};
use axum::{
    extract::Request,
    http::header,
    middleware::Next,
    response::{IntoResponse, Redirect},
    routing::{get, post},
    Router,
};
use tower_governor::{governor::GovernorConfigBuilder, GovernorLayer};
use tower_http::{
    compression::CompressionLayer,
    services::{ServeDir, ServeFile},
    set_header::{SetResponseHeader, SetResponseHeaderLayer},
};

pub fn create_router(state: SharedState) -> Router {
    let resume_route = if cfg!(debug_assertions) {
        "/resume/ytm".to_string()
    } else {
        std::env::var("RESUME_TAG")
            .map(|tag| format!("/resume/{}", tag))
            .unwrap_or_else(|_| "/resume/ytm".to_string())
    };

    // Static assets with long-lived cache headers
    let assets_service = SetResponseHeader::overriding(
        ServeDir::new("assets"),
        header::CACHE_CONTROL,
        header::HeaderValue::from_static("public, max-age=31536000, immutable"),
    );

    // Rate limiting for write API routes
    let governor_conf = GovernorConfigBuilder::default()
        .per_second(2)
        .burst_size(5)
        .finish()
        .unwrap();

    let api_write_routes = Router::new()
        .route("/api/comments", post(api::create_comment))
        .route(
            "/api/comments/edit/:comment_id",
            axum::routing::put(api::edit_comment),
        )
        .route(
            "/api/comments/delete/:comment_id",
            axum::routing::delete(api::delete_comment),
        )
        .route("/api/guestbook", post(api::create_guestbook_entry))
        .route(
            "/api/guestbook/edit/:entry_id",
            axum::routing::put(api::edit_guestbook_entry),
        )
        .route(
            "/api/guestbook/delete/:entry_id",
            axum::routing::delete(api::delete_guestbook_entry),
        )
        .route("/api/post/:slug/like", post(api::toggle_like))
        .route("/api/visit", post(api::record_visit))
        .layer(GovernorLayer {
            config: governor_conf.into(),
        });

    Router::new()
        .route("/", get(pages::handle_index))
        .route("/blog", get(pages::handle_blog))
        .route("/review", get(pages::handle_review))
        .route("/diary", get(pages::handle_diary))
        .route("/series", get(pages::handle_series))
        .route("/series/:name", get(pages::handle_series_detail))
        .route("/post/:id", get(pages::handle_post))
        .route(&resume_route, get(pages::handle_resume))
        .route("/guestbook", get(pages::handle_guestbook))
        .route("/feed.xml", get(feed::handle_feed))
        .route("/sitemap.xml", get(feed::handle_sitemap))
        .route("/health", get(api::health_check))
        .route("/api/search", get(api::handle_search))
        .route("/api/set-lang", get(api::handle_set_lang))
        .route("/api/comments/:post_id", get(api::get_comments))
        .route("/api/guestbook", get(api::get_guestbook_entries))
        .route("/api/visitor-stats", get(api::get_visitor_stats))
        .merge(api_write_routes)
        .nest_service("/assets", assets_service)
        .nest_service("/favicon.ico", ServeFile::new("assets/favicon/favicon.ico"))
        .nest_service("/robots.txt", ServeFile::new("assets/robots.txt"))
        .layer(axum::middleware::from_fn(trailing_slash_redirect))
        .layer(SetResponseHeaderLayer::overriding(
            header::HeaderName::from_static("x-content-type-options"),
            header::HeaderValue::from_static("nosniff"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            header::HeaderName::from_static("x-frame-options"),
            header::HeaderValue::from_static("DENY"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            header::HeaderName::from_static("referrer-policy"),
            header::HeaderValue::from_static("strict-origin-when-cross-origin"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            header::HeaderName::from_static("strict-transport-security"),
            header::HeaderValue::from_static("max-age=63072000; includeSubDomains; preload"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            header::HeaderName::from_static("content-security-policy"),
            header::HeaderValue::from_static(
                "default-src 'self'; \
                 script-src 'self' 'unsafe-inline' https://unpkg.com https://cdn.jsdelivr.net https://cdnjs.cloudflare.com https://www.googletagmanager.com; \
                 style-src 'self' 'unsafe-inline' https://fonts.googleapis.com https://unpkg.com https://cdn.jsdelivr.net https://cdnjs.cloudflare.com; \
                 font-src 'self' https://fonts.gstatic.com https://unpkg.com https://cdn.jsdelivr.net; \
                 img-src 'self' data: https:; \
                 connect-src 'self' https://www.google-analytics.com; \
                 frame-ancestors 'none'"
            ),
        ))
        .layer(CompressionLayer::new())
        .layer(tower_http::trace::TraceLayer::new_for_http())
        .fallback(pages::handle_error)
        .with_state(state)
}

async fn trailing_slash_redirect(req: Request, next: Next) -> impl IntoResponse {
    let uri = req.uri().clone();
    let path = uri.path();

    if path.len() > 1 && path.ends_with('/') {
        let new_path = path.trim_end_matches('/');
        let new_uri = if let Some(query) = uri.query() {
            format!("{}?{}", new_path, query)
        } else {
            new_path.to_string()
        };
        return Redirect::permanent(&new_uri).into_response();
    }

    next.run(req).await.into_response()
}

#[cfg(debug_assertions)]
pub fn add_live_reload(app: Router, state: SharedState) -> Router {
    use crate::i18n::Lang;
    use crate::post::{get_series, load_posts};
    use notify::Watcher;

    let livereload = tower_livereload::LiveReloadLayer::new().request_predicate(
        |req: &axum::http::Request<axum::body::Body>| !req.headers().contains_key("hx-request"),
    );
    let reloader = livereload.reloader();

    let state_clone = state.clone();
    let mut watcher = notify::recommended_watcher(move |evt: Result<notify::Event, _>| {
        if let Ok(evt) = evt {
            let is_content_change = evt
                .paths
                .iter()
                .any(|p| p.to_string_lossy().contains("contents"));
            if is_content_change {
                let posts_state = state_clone.posts.clone();
                let series_cache = state_clone.series_cache.clone();
                tokio::spawn(async move {
                    // Clear and reload posts
                    {
                        let mut posts = posts_state.write().await;
                        posts.clear();
                    }
                    if let Err(e) = load_posts(posts_state.clone()).await {
                        tracing::error!("Failed to reload posts: {}", e);
                        return;
                    }
                    // Refresh series cache
                    let new_series = {
                        let posts = posts_state.read().await;
                        get_series(&posts, Lang::En, false)
                    };
                    *series_cache.write().await = new_series;
                    tracing::info!("Contents reloaded successfully");
                });
            }
        }
        reloader.reload();
    })
    .unwrap();

    let paths = ["assets", "templates", "contents"];
    for path in paths.iter() {
        watcher
            .watch(std::path::Path::new(path), notify::RecursiveMode::Recursive)
            .unwrap();
    }
    // Leak the watcher so it stays alive for the process lifetime
    std::mem::forget(watcher);
    app.layer(livereload)
}
