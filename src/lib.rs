pub mod post;
pub mod templates;

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
