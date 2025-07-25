pub mod filters;
pub mod post;
pub mod templates;
pub mod db;

use post::Post;
use db::Database;
use std::sync::Arc;
use tokio::sync::RwLock;

pub type AppState = Arc<RwLock<Vec<Post>>>;

#[derive(Clone)]
pub struct SharedState {
    pub posts: AppState,
    pub db: Database,
}

#[derive(Default)]
pub struct Blog {
    pub title: String,
}

impl Blog {
    pub fn new() -> Self {
        Blog::default()
    }

    pub fn set_title(mut self, title: &str) -> Self {
        self.title = title.to_string();
        self
    }
}
