pub mod db;
pub mod error;
pub mod filters;
pub mod handlers;
pub mod i18n;
pub mod post;
pub mod router;
pub mod templates;

use db::Database;
use post::{Post, Series};
use std::sync::Arc;
use tokio::sync::RwLock;

pub const SITE_URL: &str = "https://miniex.blog";
pub const SITE_DESCRIPTION: &str =
    "miniex dev blog - Rust development, study notes, tech reviews, and more";

pub type AppState = Arc<RwLock<Vec<Post>>>;

#[derive(Clone)]
pub struct SharedState {
    pub posts: AppState,
    pub db: Database,
    pub series_cache: Arc<RwLock<Vec<Series>>>,
}

#[derive(Default)]
pub struct Blog {
    pub title: String,
    pub description: String,
    pub url: String,
    pub og_type: String,
    pub v: String,
}

impl Blog {
    pub fn new() -> Self {
        use std::sync::OnceLock;
        use std::time::{SystemTime, UNIX_EPOCH};

        static VERSION: OnceLock<String> = OnceLock::new();
        let v = VERSION
            .get_or_init(|| {
                let secs = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                format!("{:x}", secs)
            })
            .clone();

        Blog {
            og_type: "website".to_string(),
            v,
            ..Blog::default()
        }
    }

    pub fn set_title(mut self, title: &str) -> Self {
        self.title = title.to_string();
        self
    }

    pub fn set_description(mut self, description: &str) -> Self {
        self.description = description.to_string();
        self
    }

    pub fn set_url(mut self, url: &str) -> Self {
        self.url = url.to_string();
        self
    }

    pub fn set_og_type(mut self, og_type: &str) -> Self {
        self.og_type = og_type.to_string();
        self
    }
}
