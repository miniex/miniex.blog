mod de;

use crate::i18n::Lang;
use crate::AppState;
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use gray_matter::{engine::YAML, Matter};
use pulldown_cmark::{html, CodeBlockKind, Event, Options, Parser, Tag, TagEnd};
use serde::{Deserialize, Serialize};
use slug::slugify;
use std::{
    cmp::Ordering,
    collections::HashMap,
    path::{Path, PathBuf},
};
use tokio::fs;

#[derive(Deserialize, Serialize, Clone)]
pub struct Post {
    pub post_type: PostType,
    pub metadata: PostMetadata,
    pub content: String,
    pub slug: String,
    pub toc: Vec<TocEntry>,
    pub reading_time_min: u32,
    pub lang: Lang,
    pub translation_key: String,
}

#[derive(Deserialize, Serialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PostType {
    Blog,
    Review,
    Diary,
}

#[derive(Deserialize, Serialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SeriesStatus {
    Ongoing,
    Completed,
}

impl std::fmt::Display for SeriesStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SeriesStatus::Ongoing => write!(f, "Ongoing"),
            SeriesStatus::Completed => write!(f, "Completed"),
        }
    }
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Series {
    pub name: String,
    pub description: Option<String>,
    pub status: SeriesStatus,
    pub authors: Vec<String>,
    #[serde(with = "de::date_format")]
    pub updated_at: DateTime<Utc>,
    pub post_count: usize,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct TocEntry {
    pub level: u8,
    pub text: String,
    pub id: String,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct SeriesNavInfo {
    pub series_name: String,
    pub current_index: usize,
    pub total_count: usize,
    pub prev_slug: Option<String>,
    pub next_slug: Option<String>,
    pub prev_title: Option<String>,
    pub next_title: Option<String>,
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
    #[serde(default)]
    pub series: Option<String>,
    #[serde(default)]
    pub series_order: Option<u32>,
    #[serde(default)]
    pub series_description: Option<String>,
    #[serde(default)]
    pub series_status: Option<String>,
    #[serde(default)]
    pub prev_post: Option<String>,
    #[serde(default)]
    pub next_post: Option<String>,
    // -- og image (optional) --
    #[serde(default)]
    pub og_image: Option<String>,
    // -- i18n (optional) --
    #[serde(default)]
    pub lang: Option<String>,
    #[serde(default)]
    pub slug: Option<String>,
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

/// Parse language suffix from file stem: "my-post.ko" -> ("my-post", Some(Ko))
fn parse_file_lang(stem: &str) -> (String, Option<Lang>) {
    for suffix in &[".ko", ".ja", ".en"] {
        if let Some(base) = stem.strip_suffix(suffix) {
            return (base.to_string(), Some(Lang::parse(&suffix[1..])));
        }
    }
    (stem.to_string(), None)
}

/// Deduplicate posts by translation_key, preferring the given language.
/// For each group of posts sharing the same translation_key, pick the one
/// matching `lang`; if none matches, pick the first one (arbitrary fallback).
pub fn dedup_by_translation(posts: Vec<Post>, lang: Lang) -> Vec<Post> {
    let mut seen: HashMap<String, Post> = HashMap::new();
    for post in posts {
        seen.entry(post.translation_key.clone())
            .and_modify(|existing| {
                // Replace if the new post matches the preferred language
                // and the existing one does not
                if post.lang == lang && existing.lang != lang {
                    *existing = post.clone();
                }
            })
            .or_insert(post);
    }
    seen.into_values().collect()
}

/// get recent posts with language fallback
pub fn get_recent_posts(posts: &[Post], lang: Lang) -> Vec<Post> {
    let all_posts: Vec<Post> = posts.to_vec();
    let mut deduped = dedup_by_translation(all_posts, lang);
    deduped.sort();
    deduped.into_iter().take(5).collect()
}

/// get posts by category with language fallback
pub fn get_posts_by_category(
    posts: &[Post],
    post_type: PostType,
    category: Option<&str>,
    lang: Lang,
    sort_asc: bool,
) -> Vec<Post> {
    let filtered: Vec<Post> = posts
        .iter()
        .filter(|post| {
            post.post_type == post_type
                && category
                    .map(|c| post.metadata.tags.contains(&c.to_string()))
                    .unwrap_or(true)
        })
        .cloned()
        .collect();

    let mut deduped = dedup_by_translation(filtered, lang);
    if sort_asc {
        deduped.sort_by(|a, b| a.metadata.created_at.cmp(&b.metadata.created_at));
    } else {
        deduped.sort();
    }
    deduped
}

// load posts from mdx files
pub async fn load_posts(state: AppState) -> Result<()> {
    let matter = Matter::<YAML>::new();
    let content_dir = PathBuf::from("contents");
    process_content_directory(&content_dir, &matter, &state).await?;

    // Compute series navigation after all posts are loaded
    compute_series_navigation(&state).await;

    Ok(())
}

/// get all series information from posts (all languages), sorted by updated_at DESC
pub fn get_series(posts: &[Post], _lang: Lang, sort_asc: bool) -> Vec<Series> {
    let mut series_map: HashMap<String, Vec<String>> = HashMap::new();
    let mut latest_updates: HashMap<String, DateTime<Utc>> = HashMap::new();
    let mut descriptions: HashMap<String, Option<String>> = HashMap::new();
    let mut statuses: HashMap<String, SeriesStatus> = HashMap::new();
    let mut post_counts: HashMap<String, usize> = HashMap::new();

    for post in posts.iter() {
        if let Some(series_name) = &post.metadata.series {
            series_map
                .entry(series_name.clone())
                .or_default()
                .push(post.metadata.author.clone());

            latest_updates
                .entry(series_name.clone())
                .and_modify(|e| *e = (*e).max(post.metadata.updated_at))
                .or_insert(post.metadata.updated_at);

            *post_counts.entry(series_name.clone()).or_default() += 1;

            // Take description from the first post that has one
            if !descriptions.contains_key(series_name) || descriptions[series_name].is_none() {
                if let Some(desc) = &post.metadata.series_description {
                    descriptions.insert(series_name.clone(), Some(desc.clone()));
                }
            }

            // Take status from the first post that has one
            if !statuses.contains_key(series_name) {
                if let Some(status_str) = &post.metadata.series_status {
                    let status = match status_str.to_lowercase().as_str() {
                        "completed" => SeriesStatus::Completed,
                        _ => SeriesStatus::Ongoing,
                    };
                    statuses.insert(series_name.clone(), status);
                }
            }
        }
    }

    let mut series: Vec<Series> = series_map
        .into_iter()
        .map(|(name, authors)| {
            let unique_authors: Vec<String> = authors
                .into_iter()
                .collect::<std::collections::HashSet<_>>()
                .into_iter()
                .collect();
            Series {
                description: descriptions.get(&name).cloned().flatten(),
                status: statuses
                    .get(&name)
                    .cloned()
                    .unwrap_or(SeriesStatus::Ongoing),
                post_count: post_counts.get(&name).copied().unwrap_or(0),
                authors: unique_authors,
                updated_at: latest_updates.get(&name).cloned().unwrap_or_default(),
                name,
            }
        })
        .collect();

    if sort_asc {
        series.sort_by(|a, b| a.updated_at.cmp(&b.updated_at));
    } else {
        series.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    }
    series
}

/// get all series names from posts (all languages), sorted alphabetically
pub fn get_series_names(posts: &[Post], _lang: Lang) -> Vec<String> {
    let mut series_names: Vec<String> = posts
        .iter()
        .filter_map(|post| post.metadata.series.clone())
        .collect::<std::collections::HashSet<String>>()
        .into_iter()
        .collect();

    series_names.sort();
    series_names
}

/// get posts by series name with language fallback, sorted by created_at DESC by default
pub fn get_posts_by_series(
    posts: &[Post],
    series_name: &str,
    lang: Lang,
    sort_asc: bool,
) -> Vec<Post> {
    let filtered: Vec<Post> = posts
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

    let mut series_posts = dedup_by_translation(filtered, lang);

    if sort_asc {
        series_posts.sort_by(|a, b| a.metadata.created_at.cmp(&b.metadata.created_at));
    } else {
        series_posts.sort_by(|a, b| b.metadata.created_at.cmp(&a.metadata.created_at));
    }
    series_posts
}

/// Get series navigation info for a specific post (scoped to same language)
pub fn get_series_nav_info(posts: &[Post], current_post: &Post) -> Option<SeriesNavInfo> {
    let series_name = current_post.metadata.series.as_ref()?;
    // Always use ASC order for series navigation (prev/next)
    let series_posts = get_posts_by_series(posts, series_name, current_post.lang, true);
    let total_count = series_posts.len();

    let current_idx = series_posts
        .iter()
        .position(|p| p.slug == current_post.slug)?;

    let prev = if current_idx > 0 {
        series_posts.get(current_idx - 1)
    } else {
        None
    };
    let next = series_posts.get(current_idx + 1);

    Some(SeriesNavInfo {
        series_name: series_name.clone(),
        current_index: current_idx + 1, // 1-based
        total_count,
        prev_slug: prev.map(|p| p.slug.clone()),
        next_slug: next.map(|p| p.slug.clone()),
        prev_title: prev.map(|p| p.metadata.title.clone()),
        next_title: next.map(|p| p.metadata.title.clone()),
    })
}

/// Get available translations for a post by its translation_key
pub fn get_available_translations(posts: &[Post], translation_key: &str) -> Vec<Lang> {
    let mut langs: Vec<Lang> = posts
        .iter()
        .filter(|p| p.translation_key == translation_key)
        .map(|p| p.lang)
        .collect();
    langs.sort_by_key(|l| match l {
        Lang::Ko => 0,
        Lang::Ja => 1,
        Lang::En => 2,
    });
    langs.dedup();
    langs
}

/// Compute series navigation (prev/next) automatically for posts that don't have manual values
/// Groups by (series_name, lang) so each language has independent prev/next chains
async fn compute_series_navigation(state: &AppState) {
    let mut posts = state.write().await;

    // Collect series groups keyed by (series_name, lang)
    let mut series_groups: HashMap<(String, Lang), Vec<usize>> = HashMap::new();
    for (idx, post) in posts.iter().enumerate() {
        if let Some(series_name) = &post.metadata.series {
            series_groups
                .entry((series_name.clone(), post.lang))
                .or_default()
                .push(idx);
        }
    }

    // For each series+lang group, sort indices by series_order/created_at and assign prev/next
    for (_key, mut indices) in series_groups {
        // Sort indices by the post's series_order then created_at
        indices.sort_by(|&a, &b| {
            let pa = &posts[a];
            let pb = &posts[b];
            match (pa.metadata.series_order, pb.metadata.series_order) {
                (Some(a_order), Some(b_order)) => a_order.cmp(&b_order),
                (Some(_), None) => Ordering::Less,
                (None, Some(_)) => Ordering::Greater,
                (None, None) => pa.metadata.created_at.cmp(&pb.metadata.created_at),
            }
        });

        // Collect slugs for assignment
        let slugs: Vec<String> = indices.iter().map(|&i| posts[i].slug.clone()).collect();

        for (pos, &idx) in indices.iter().enumerate() {
            // Only set prev_post if not manually specified
            if posts[idx].metadata.prev_post.is_none() && pos > 0 {
                posts[idx].metadata.prev_post = Some(slugs[pos - 1].clone());
            }
            // Only set next_post if not manually specified
            if posts[idx].metadata.next_post.is_none() && pos + 1 < slugs.len() {
                posts[idx].metadata.next_post = Some(slugs[pos + 1].clone());
            }
        }
    }
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
    let metadata: PostMetadata = parsed
        .data
        .ok_or_else(|| anyhow::anyhow!("No front matter found"))?
        .deserialize()?;

    // Determine language and translation_key from filename
    let file_stem = file_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("untitled");
    let (translation_key, file_lang) = parse_file_lang(file_stem);

    // Language priority: filename suffix > frontmatter lang > default En
    let lang = file_lang.unwrap_or_else(|| {
        metadata
            .lang
            .as_ref()
            .map(|s| Lang::parse(s))
            .unwrap_or(Lang::En)
    });

    // Slug priority: frontmatter slug > translation_key (filename-based)
    let slug = metadata
        .slug
        .clone()
        .unwrap_or_else(|| translation_key.clone());

    // Parse the markdown with event interception for TOC and reading time
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_MATH);
    options.insert(Options::ENABLE_TABLES);
    let parser = Parser::new_ext(parsed.content.as_str(), options);

    let mut toc: Vec<TocEntry> = Vec::new();
    let mut word_count: usize = 0;
    let mut heading_id_counts: HashMap<String, usize> = HashMap::new();

    // State for heading processing
    let mut in_heading = false;
    let mut current_heading_level: u8 = 0;
    let mut current_heading_text = String::new();

    // State for graph/chart code block processing
    let mut in_graph_block = false;
    let mut in_chart_block = false;
    let mut in_plot3d_block = false;
    let mut block_content = String::new();

    // Collect events and process headings
    let events: Vec<Event> = parser.collect();
    let mut processed_events: Vec<Event> = Vec::new();

    let mut i = 0;
    while i < events.len() {
        match &events[i] {
            // Intercept graph/chart fenced code blocks
            Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(lang))) => {
                let lang_str = lang.trim().to_lowercase();
                if lang_str == "graph" {
                    in_graph_block = true;
                    block_content.clear();
                    i += 1;
                    continue;
                } else if lang_str == "chart" {
                    in_chart_block = true;
                    block_content.clear();
                    i += 1;
                    continue;
                } else if lang_str == "plot3d" {
                    in_plot3d_block = true;
                    block_content.clear();
                    i += 1;
                    continue;
                } else {
                    processed_events.push(events[i].clone());
                }
            }
            Event::End(TagEnd::CodeBlock) => {
                if in_graph_block {
                    in_graph_block = false;
                    let escaped = block_content
                        .replace('&', "&amp;")
                        .replace('<', "&lt;")
                        .replace('>', "&gt;");
                    processed_events.push(Event::Html(
                        format!("<div class=\"function-plot-target\">{}</div>", escaped).into(),
                    ));
                    block_content.clear();
                    i += 1;
                    continue;
                } else if in_chart_block {
                    in_chart_block = false;
                    let escaped = block_content
                        .replace('&', "&amp;")
                        .replace('<', "&lt;")
                        .replace('>', "&gt;");
                    processed_events.push(Event::Html(
                        format!("<div class=\"chart-js-target\">{}</div>", escaped).into(),
                    ));
                    block_content.clear();
                    i += 1;
                    continue;
                } else if in_plot3d_block {
                    in_plot3d_block = false;
                    let escaped = block_content
                        .replace('&', "&amp;")
                        .replace('<', "&lt;")
                        .replace('>', "&gt;");
                    processed_events.push(Event::Html(
                        format!("<div class=\"plotly-target\">{}</div>", escaped).into(),
                    ));
                    block_content.clear();
                    i += 1;
                    continue;
                } else {
                    processed_events.push(events[i].clone());
                }
            }
            Event::Start(Tag::Heading { level, .. }) => {
                let lvl = *level as u8;
                if lvl == 2 || lvl == 3 {
                    in_heading = true;
                    current_heading_level = lvl;
                    current_heading_text.clear();
                    i += 1;
                    continue;
                } else {
                    processed_events.push(events[i].clone());
                }
            }
            Event::End(TagEnd::Heading(level)) => {
                let lvl = *level as u8;
                if in_heading && (lvl == 2 || lvl == 3) {
                    in_heading = false;
                    // Generate slug for heading
                    let mut slug_id = slugify(&current_heading_text);
                    if slug_id.is_empty() {
                        slug_id = format!("heading-{}", toc.len());
                    }

                    // Handle duplicate IDs
                    let count = heading_id_counts.entry(slug_id.clone()).or_insert(0);
                    let final_id = if *count > 0 {
                        format!("{}-{}", slug_id, count)
                    } else {
                        slug_id.clone()
                    };
                    *count += 1;

                    toc.push(TocEntry {
                        level: current_heading_level,
                        text: current_heading_text.clone(),
                        id: final_id.clone(),
                    });

                    // Emit heading as raw HTML with id attribute
                    let tag = format!("h{}", current_heading_level);
                    processed_events.push(Event::Html(
                        format!(
                            "<{} id=\"{}\">{}</{}>",
                            tag, final_id, current_heading_text, tag
                        )
                        .into(),
                    ));

                    i += 1;
                    continue;
                } else {
                    processed_events.push(events[i].clone());
                }
            }
            Event::Text(text) => {
                if in_graph_block || in_chart_block || in_plot3d_block {
                    block_content.push_str(text);
                    i += 1;
                    continue;
                }
                if in_heading {
                    current_heading_text.push_str(text);
                }
                // Count words for reading time
                word_count += text.split_whitespace().count();
                if !in_heading {
                    processed_events.push(events[i].clone());
                }
            }
            Event::Code(code) => {
                if in_graph_block || in_chart_block || in_plot3d_block {
                    block_content.push_str(code);
                    i += 1;
                    continue;
                }
                if in_heading {
                    current_heading_text.push_str(code);
                }
                word_count += code.split_whitespace().count();
                if !in_heading {
                    processed_events.push(events[i].clone());
                }
            }
            _ => {
                if in_graph_block || in_chart_block || in_plot3d_block {
                    i += 1;
                    continue;
                }
                if !in_heading {
                    processed_events.push(events[i].clone());
                }
            }
        }
        i += 1;
    }

    // Calculate reading time (200 wpm for Korean-heavy content, minimum 1 min)
    let reading_time_min = std::cmp::max(1, (word_count as u32) / 200);

    // Write to String buffer
    let mut html_output = String::new();
    html::push_html(&mut html_output, processed_events.into_iter());

    let post = Post {
        post_type,
        metadata,
        content: html_output,
        slug,
        toc,
        reading_time_min,
        lang,
        translation_key,
    };

    Ok(post)
}
