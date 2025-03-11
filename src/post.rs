mod de;

use crate::AppState;
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use gray_matter::{engine::YAML, Matter};
use pulldown_cmark::{html, Options, Parser};
use serde::{Deserialize, Serialize};
use slug::slugify;
use std::{
    cmp::Ordering,
    path::{Path, PathBuf},
};
use tokio::fs;

#[derive(Deserialize, Serialize, Clone)]
pub struct Post {
    pub post_type: PostType,
    pub metadata: PostMetadata,
    pub content: String,
    pub slug: String,
}

#[derive(Deserialize, Serialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PostType {
    Blog,
    Review,
    Diary,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Series {
    pub name: String,
    pub authors: Vec<String>,
    #[serde(with = "de::date_format")]
    pub updated_at: DateTime<Utc>,
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
    pub description: String,
    pub author: String,
    pub tags: Vec<String>,
    #[serde(with = "de::date_format")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "de::date_format")]
    pub updated_at: DateTime<Utc>,
    // -- series (optional) --
    pub series: Option<String>,
    pub prev_post: Option<String>,
    pub next_post: Option<String>,
}

impl Ord for Post {
    fn cmp(&self, other: &Self) -> Ordering {
        other.metadata.created_at.cmp(&self.metadata.created_at)
    }
}

impl PartialOrd for Post {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Post {
    fn eq(&self, other: &Self) -> bool {
        self.metadata.created_at == other.metadata.created_at
    }
}

impl Eq for Post {}

impl Post {
    pub fn generate_slug(&mut self) {
        self.slug = slugify(&self.metadata.title);
    }
}

/// get recent posts
pub fn get_recent_posts(posts: &[Post]) -> Vec<Post> {
    // Clone all posts
    let mut all_posts: Vec<Post> = posts.to_vec();

    // Sort by newest first
    all_posts.sort();

    // Return only the 5 most recent posts
    all_posts.into_iter().take(5).collect()
}

/// get posts by category
pub fn get_posts_by_category(
    posts: &[Post],
    post_type: PostType,
    category: Option<&str>,
) -> Vec<Post> {
    let mut filtered_posts = posts
        .iter()
        .filter(|post| {
            post.post_type == post_type
                && category
                    .map(|c| post.metadata.tags.contains(&c.to_string()))
                    .unwrap_or(true)
        })
        .cloned()
        .collect::<Vec<Post>>();

    // Sort by newest first
    filtered_posts.sort();
    filtered_posts
}

// load posts from mdx files
pub async fn load_posts(state: AppState) -> Result<()> {
    let matter = Matter::<YAML>::new();
    let content_dir = PathBuf::from("contents");
    process_content_directory(&content_dir, &matter, &state).await?;

    Ok(())
}

/// get all series information from posts, sorted by name
pub fn get_series(posts: &[Post]) -> Vec<Series> {
    let mut series_map: std::collections::HashMap<String, Vec<String>> =
        std::collections::HashMap::new();
    let mut latest_updates: std::collections::HashMap<String, DateTime<Utc>> =
        std::collections::HashMap::new();

    // Collect authors and latest update times for each series
    for post in posts.iter() {
        if let Some(series_name) = &post.metadata.series {
            // Update authors
            series_map
                .entry(series_name.clone())
                .or_default()
                .push(post.metadata.author.clone());

            // Update latest update time
            latest_updates
                .entry(series_name.clone())
                .and_modify(|e| *e = (*e).max(post.metadata.updated_at))
                .or_insert(post.metadata.updated_at);
        }
    }

    // Convert to Vec<Series>
    let mut series: Vec<Series> = series_map
        .into_iter()
        .map(|(name, authors)| {
            let unique_authors: Vec<String> = authors
                .into_iter()
                .collect::<std::collections::HashSet<_>>()
                .into_iter()
                .collect();
            Series {
                name: name.clone(),
                authors: unique_authors,
                updated_at: latest_updates.get(&name).cloned().unwrap_or_default(),
            }
        })
        .collect();

    // Sort by name
    series.sort_by(|a, b| a.name.cmp(&b.name));
    series
}

/// get all series names from posts, sorted alphabetically
pub fn get_series_names(posts: &[Post]) -> Vec<String> {
    let mut series_names: Vec<String> = posts
        .iter()
        .filter_map(|post| post.metadata.series.clone())
        .collect::<std::collections::HashSet<String>>()
        .into_iter()
        .collect();

    // Sort alphabetically
    series_names.sort();
    series_names
}

/// get posts by series name
pub fn get_posts_by_series(posts: &[Post], series_name: &str) -> Vec<Post> {
    let mut series_posts: Vec<Post> = posts
        .iter()
        .filter(|post| {
            post.metadata
                .series
                .as_ref()
                .map(|s| s == series_name)
                .unwrap_or(false)
        })
        .cloned()
        .collect();

    // Sort by creation date to maintain chronological order
    series_posts.sort();
    series_posts
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

    // Parse the markdown
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    let parser = Parser::new_ext(parsed.content.as_str(), options);

    // Write to String buffer
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);

    let mut post = Post {
        post_type,
        metadata,
        content: html_output,
        slug: String::new(),
    };

    post.generate_slug();

    Ok(post)
}
