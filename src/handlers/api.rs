use crate::{
    db::{Comment, Guestbook},
    i18n::Lang,
    post::{dedup_by_translation, Post},
    SharedState,
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde::{Deserialize, Serialize};

// --- Search API ---

#[derive(Deserialize)]
pub struct SearchQuery {
    q: String,
    lang: String,
}

#[derive(Serialize)]
pub struct SearchResult {
    slug: String,
    title: String,
    description: String,
    post_type: String,
    tags: Vec<String>,
    created_at: String,
    reading_time_min: u32,
    lang: String,
}

pub async fn handle_search(
    State(state): State<SharedState>,
    Query(query): Query<SearchQuery>,
) -> Json<Vec<SearchResult>> {
    let posts = state.posts.read().await;
    let search_lang = Lang::parse(&query.lang);
    let q = query.q.to_lowercase();

    let matching: Vec<Post> = posts
        .iter()
        .filter(|post| {
            post.metadata.title.to_lowercase().contains(&q)
                || post.metadata.description.to_lowercase().contains(&q)
                || post
                    .metadata
                    .tags
                    .iter()
                    .any(|tag| tag.to_lowercase().contains(&q))
        })
        .cloned()
        .collect();

    let deduped = dedup_by_translation(matching, search_lang);
    let mut sorted = deduped;
    sorted.sort_by(|a, b| b.metadata.created_at.cmp(&a.metadata.created_at));

    let results: Vec<SearchResult> = sorted
        .into_iter()
        .take(20)
        .map(|post| SearchResult {
            slug: post.slug.clone(),
            title: post.metadata.title.clone(),
            description: post.metadata.description.clone(),
            post_type: post.post_type.to_string().to_lowercase(),
            tags: post.metadata.tags.clone(),
            created_at: post.metadata.created_at.format("%Y/%m/%d").to_string(),
            reading_time_min: post.reading_time_min,
            lang: post.lang.as_str().to_string(),
        })
        .collect();

    Json(results)
}

// --- Language Switch ---

#[derive(Deserialize)]
pub struct SetLangQuery {
    lang: String,
}

pub async fn handle_set_lang(
    Query(query): Query<SetLangQuery>,
    headers: axum::http::HeaderMap,
) -> impl IntoResponse {
    let lang = Lang::parse(&query.lang);
    let cookie = format!(
        "lang={}; Path=/; Max-Age=31536000; SameSite=Lax",
        lang.as_str()
    );

    let referer = headers
        .get(axum::http::header::REFERER)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("/");

    (
        StatusCode::SEE_OTHER,
        [
            (axum::http::header::SET_COOKIE, cookie),
            (axum::http::header::LOCATION, referer.to_string()),
        ],
    )
}

// --- Comments & Guestbook API ---

#[derive(Deserialize)]
pub struct CreateCommentRequest {
    author: String,
    content: String,
    password: Option<String>,
}

#[derive(Serialize)]
pub struct ApiResponse<T> {
    data: T,
    message: String,
}

pub async fn get_comments(
    Path(post_id): Path<String>,
    State(state): State<SharedState>,
) -> Result<Json<ApiResponse<Vec<Comment>>>, StatusCode> {
    match state.db.get_comments_by_post(&post_id).await {
        Ok(comments) => Ok(Json(ApiResponse {
            data: comments,
            message: "Comments retrieved successfully".to_string(),
        })),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[derive(Deserialize)]
pub struct CreateCommentWithPostRequest {
    post_id: String,
    author: String,
    content: String,
    password: Option<String>,
}

#[derive(Deserialize)]
pub struct EditCommentRequest {
    content: String,
    password: String,
}

#[derive(Deserialize)]
pub struct DeleteRequest {
    password: String,
}

#[derive(Serialize)]
pub struct EditResponse {
    success: bool,
    message: String,
}

pub async fn create_comment(
    State(state): State<SharedState>,
    Json(payload): Json<CreateCommentWithPostRequest>,
) -> Result<Json<ApiResponse<Comment>>, StatusCode> {
    match state
        .db
        .create_comment(
            &payload.post_id,
            &payload.author,
            &payload.content,
            payload.password.as_deref(),
        )
        .await
    {
        Ok(comment) => Ok(Json(ApiResponse {
            data: comment,
            message: "Comment created successfully".to_string(),
        })),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn edit_comment(
    Path(comment_id): Path<String>,
    State(state): State<SharedState>,
    Json(payload): Json<EditCommentRequest>,
) -> Result<Json<EditResponse>, StatusCode> {
    match state
        .db
        .update_comment(&comment_id, &payload.content, &payload.password)
        .await
    {
        Ok(success) => Ok(Json(EditResponse {
            success,
            message: if success {
                "Comment updated successfully".to_string()
            } else {
                "Wrong password or comment not found".to_string()
            },
        })),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn delete_comment(
    Path(comment_id): Path<String>,
    State(state): State<SharedState>,
    Json(payload): Json<DeleteRequest>,
) -> Result<Json<EditResponse>, StatusCode> {
    match state
        .db
        .delete_comment(&comment_id, &payload.password)
        .await
    {
        Ok(success) => Ok(Json(EditResponse {
            success,
            message: if success {
                "Comment deleted successfully".to_string()
            } else {
                "Wrong password or comment not found".to_string()
            },
        })),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn get_guestbook_entries(
    State(state): State<SharedState>,
) -> Result<Json<ApiResponse<Vec<Guestbook>>>, StatusCode> {
    match state.db.get_guestbook_entries(Some(50)).await {
        Ok(entries) => Ok(Json(ApiResponse {
            data: entries,
            message: "Guestbook entries retrieved successfully".to_string(),
        })),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn create_guestbook_entry(
    State(state): State<SharedState>,
    Json(payload): Json<CreateCommentRequest>,
) -> Result<Json<ApiResponse<Guestbook>>, StatusCode> {
    match state
        .db
        .create_guestbook_entry(
            &payload.author,
            &payload.content,
            payload.password.as_deref(),
        )
        .await
    {
        Ok(entry) => Ok(Json(ApiResponse {
            data: entry,
            message: "Guestbook entry created successfully".to_string(),
        })),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn edit_guestbook_entry(
    Path(entry_id): Path<String>,
    State(state): State<SharedState>,
    Json(payload): Json<EditCommentRequest>,
) -> Result<Json<EditResponse>, StatusCode> {
    match state
        .db
        .update_guestbook_entry(&entry_id, &payload.content, &payload.password)
        .await
    {
        Ok(success) => Ok(Json(EditResponse {
            success,
            message: if success {
                "Guestbook entry updated successfully".to_string()
            } else {
                "Wrong password or entry not found".to_string()
            },
        })),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn delete_guestbook_entry(
    Path(entry_id): Path<String>,
    State(state): State<SharedState>,
    Json(payload): Json<DeleteRequest>,
) -> Result<Json<EditResponse>, StatusCode> {
    match state
        .db
        .delete_guestbook_entry(&entry_id, &payload.password)
        .await
    {
        Ok(success) => Ok(Json(EditResponse {
            success,
            message: if success {
                "Guestbook entry deleted successfully".to_string()
            } else {
                "Wrong password or entry not found".to_string()
            },
        })),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}
