use crate::{
    db::Guestbook,
    filters,
    i18n::{Lang, Translations},
    post::{Post, Series, SeriesNavInfo, SeriesStatus},
    Blog,
};
use askama::Template;
use chrono::{DateTime, FixedOffset};

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    pub blog: Blog,
    pub recent_posts: Vec<Post>,
    pub t: Translations,
    pub lang: Lang,
}

#[derive(Template)]
#[template(path = "blog.html")]
pub struct BlogTemplate {
    pub blog: Blog,
    pub posts: Vec<Post>,
    pub categories: Vec<String>,
    pub current_category: Option<String>,
    pub current_page: u32,
    pub total_pages: u32,
    pub prev_page: Option<u32>,
    pub next_page: Option<u32>,
    pub page_numbers: Vec<u32>,
    pub t: Translations,
    pub lang: Lang,
    pub sort_asc: bool,
}

#[derive(Template)]
#[template(path = "review.html")]
pub struct ReviewTemplate {
    pub blog: Blog,
    pub posts: Vec<Post>,
    pub categories: Vec<String>,
    pub current_category: Option<String>,
    pub current_page: u32,
    pub total_pages: u32,
    pub prev_page: Option<u32>,
    pub next_page: Option<u32>,
    pub page_numbers: Vec<u32>,
    pub t: Translations,
    pub lang: Lang,
    pub sort_asc: bool,
}

#[derive(Template)]
#[template(path = "diary.html")]
pub struct DiaryTemplate {
    pub blog: Blog,
    pub posts: Vec<Post>,
    pub categories: Vec<String>,
    pub current_category: Option<String>,
    pub current_page: u32,
    pub total_pages: u32,
    pub prev_page: Option<u32>,
    pub next_page: Option<u32>,
    pub page_numbers: Vec<u32>,
    pub t: Translations,
    pub lang: Lang,
    pub sort_asc: bool,
}

#[derive(Template)]
#[template(path = "series.html")]
pub struct SeriesTemplate {
    pub blog: Blog,
    pub series: Vec<Series>,
    pub t: Translations,
    pub lang: Lang,
    pub sort_asc: bool,
}

#[derive(Template)]
#[template(path = "series_detail.html")]
pub struct SeriesDetailTemplate {
    pub blog: Blog,
    pub series_name: String,
    pub posts: Vec<Post>,
    pub authors: Vec<String>,
    pub updated_at: DateTime<FixedOffset>,
    pub series_description: Option<String>,
    pub series_status: SeriesStatus,
    pub t: Translations,
    pub lang: Lang,
    pub sort_asc: bool,
    pub current_page: u32,
    pub total_pages: u32,
    pub prev_page: Option<u32>,
    pub next_page: Option<u32>,
    pub page_numbers: Vec<u32>,
    pub total_posts: u32,
}

#[derive(Template)]
#[template(path = "post.html")]
pub struct PostTemplate {
    pub blog: Blog,
    pub current_post: Option<Post>,
    pub series_nav: Option<SeriesNavInfo>,
    pub t: Translations,
    pub lang: Lang,
    pub available_langs: Vec<Lang>,
}

#[derive(Template)]
#[template(path = "resume.html")]
pub struct ResumeTemplate {
    pub blog: Blog,
    pub content: String,
    pub t: Translations,
    pub lang: Lang,
}

#[derive(Template)]
#[template(path = "guestbook.html")]
pub struct GuestbookTemplate {
    pub blog: Blog,
    pub entries: Vec<Guestbook>,
    pub t: Translations,
    pub lang: Lang,
    pub sort_asc: bool,
    pub current_page: u32,
    pub total_pages: u32,
    pub prev_page: Option<u32>,
    pub next_page: Option<u32>,
    pub page_numbers: Vec<u32>,
    pub total_entries: u32,
}

#[derive(Template)]
#[template(path = "error.html")]
pub struct ErrorTemplate {
    pub blog: Blog,
    pub t: Translations,
    pub lang: Lang,
}
