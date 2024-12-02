use crate::{filters, post::{Post, Series}, Blog};
use askama::Template;
use chrono::{DateTime, Utc};

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    pub blog: Blog,
    pub recent_posts: Vec<Post>,
}

#[derive(Template)]
#[template(path = "blog.html")]
pub struct BlogTemplate {
    pub blog: Blog,
    pub posts: Vec<Post>,
    pub categories: Vec<String>,
    pub current_page: u32,
    pub total_pages: u32,
    pub prev_page: Option<u32>,
    pub next_page: Option<u32>,
    pub page_numbers: Vec<u32>,
}

#[derive(Template)]
#[template(path = "review.html")]
pub struct ReviewTemplate {
    pub blog: Blog,
}

#[derive(Template)]
#[template(path = "diary.html")]
pub struct DiaryTemplate {
    pub blog: Blog,
}

#[derive(Template)]
#[template(path = "series.html")]
pub struct SeriesTemplate {
    pub blog: Blog,
    pub series: Vec<Series>
}

#[derive(Template)]
#[template(path = "series_detail.html")]
pub struct SeriesDetailTemplate {
    pub blog: Blog, 
    pub series_name: String,
    pub posts: Vec<Post>,
    pub authors: Vec<String>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Template)]
#[template(path = "post.html")]
pub struct PostTemplate {
    pub post: Option<Post>,
}

#[derive(Template)]
#[template(path = "error.html")]
pub struct ErrorTemplate {}
