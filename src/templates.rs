use crate::{filters, post::Post, Blog};
use askama::Template;

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
#[template(path = "post.html")]
pub struct PostTemplate {
    pub post: Option<Post>,
}

#[derive(Template)]
#[template(path = "error.html")]
pub struct ErrorTemplate {}
