use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone)]
pub struct Post {
    pub post_type: PostType,
    pub metadata: PostMetadata,
    pub content: String,
}

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum PostType {
    Blog,
    Review,
    Diary,
}

impl std::fmt::Display for PostType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PostType::Blog => write!(f, "Blog"),
            PostType::Review => write!(f, "Review"),
            PostType::Diary => write!(f, "Diary"),
        }
    }
}

#[derive(Deserialize, Serialize, Clone)]
pub struct PostMetadata {
    pub title: String,
    pub date: String,
    pub author: String,
    pub tags: Vec<String>,
}
