use crate::AppState;
use anyhow::{Context, Result};
use gray_matter::{engine::YAML, Matter};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::fs;

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

//
pub async fn load_posts(state: AppState) -> Result<()> {
    let matter = Matter::<YAML>::new();
    let content_dir = PathBuf::from("contents");
    process_content_directory(&content_dir, &matter, &state).await?;

    Ok(())
}

#[async_recursion::async_recursion]
async fn process_content_directory(
    path: &Path,
    matter: &Matter<YAML>,
    state: &AppState,
) -> Result<()> {
    let mut entries = fs::read_dir(path).await?;
    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if path.is_dir() {
            if let Some(post_type) = get_post_type(&path) {
                process_type_directory(&path, post_type, matter, state).await?;
            } else {
                process_content_directory(&path, matter, state).await?;
            }
        }
    }
    Ok(())
}

fn get_post_type(path: &Path) -> Option<PostType> {
    path.file_name()?
        .to_str()
        .and_then(|s| match s.to_lowercase().as_str() {
            "blog" => Some(PostType::Blog),
            "review" => Some(PostType::Review),
            "diary" => Some(PostType::Diary),
            _ => None,
        })
}

async fn process_type_directory(
    path: &Path,
    post_type: PostType,
    matter: &Matter<YAML>,
    state: &AppState,
) -> Result<()> {
    let mut entries = fs::read_dir(path).await?;
    while let Some(entry) = entries.next_entry().await? {
        let file_path = entry.path();
        if file_path.extension().and_then(|e| e.to_str()) == Some("mdx") {
            let post = process_mdx_file(&file_path, post_type.clone(), matter)
                .await
                .with_context(|| format!("Failed to process file: {:?}", file_path))?;
            state.write().await.push(post);
        }
    }
    Ok(())
}

async fn process_mdx_file(
    file_path: &Path,
    post_type: PostType,
    matter: &Matter<YAML>,
) -> Result<Post> {
    let content = fs::read_to_string(file_path).await?;
    let parsed = matter.parse(&content);
    let metadata = parsed
        .data
        .ok_or_else(|| anyhow::anyhow!("No front matter found"))?
        .deserialize()?;
    Ok(Post {
        post_type,
        metadata,
        content: parsed.content,
    })
}
