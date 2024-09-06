use crate::Blog;
use askama::Template;

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    pub blog: Blog,
}

#[derive(Template)]
#[template(path = "blog.html")]
pub struct BlogTemplate {
    pub blog: Blog,
}
