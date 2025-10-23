use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Row, Sqlite, SqlitePool};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comment {
    pub id: String,
    pub post_id: String,
    pub author: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
    #[serde(skip_serializing)]
    pub password_hash: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Guestbook {
    pub id: String,
    pub author: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
    #[serde(skip_serializing)]
    pub password_hash: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Database {
    pub pool: Pool<Sqlite>,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self, sqlx::Error> {
        let pool = SqlitePool::connect(database_url).await?;

        // Create tables
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS comments (
                id TEXT PRIMARY KEY,
                post_id TEXT NOT NULL,
                author TEXT NOT NULL,
                content TEXT NOT NULL,
                created_at TEXT NOT NULL,
                password_hash TEXT
            )
            "#,
        )
        .execute(&pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS guestbook (
                id TEXT PRIMARY KEY,
                author TEXT NOT NULL,
                content TEXT NOT NULL,
                created_at TEXT NOT NULL,
                password_hash TEXT
            )
            "#,
        )
        .execute(&pool)
        .await?;

        Ok(Database { pool })
    }

    // Comment methods
    pub async fn create_comment(
        &self,
        post_id: &str,
        author: &str,
        content: &str,
        password: Option<&str>,
    ) -> Result<Comment, sqlx::Error> {
        let password_hash = password.map(|p| self.hash_password(p));

        let comment = Comment {
            id: Uuid::new_v4().to_string(),
            post_id: post_id.to_string(),
            author: author.to_string(),
            content: content.to_string(),
            created_at: Utc::now(),
            password_hash: password_hash.clone(),
        };

        sqlx::query(
            "INSERT INTO comments (id, post_id, author, content, created_at, password_hash) VALUES (?, ?, ?, ?, ?, ?)"
        )
        .bind(&comment.id)
        .bind(&comment.post_id)
        .bind(&comment.author)
        .bind(&comment.content)
        .bind(&comment.created_at.to_rfc3339())
        .bind(&password_hash)
        .execute(&self.pool)
        .await?;

        Ok(comment)
    }

    pub async fn get_comments_by_post(&self, post_id: &str) -> Result<Vec<Comment>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT id, post_id, author, content, created_at, password_hash FROM comments WHERE post_id = ? ORDER BY created_at DESC"
        )
        .bind(post_id)
        .fetch_all(&self.pool)
        .await?;

        let comments = rows
            .into_iter()
            .map(|row| Comment {
                id: row.get("id"),
                post_id: row.get("post_id"),
                author: row.get("author"),
                content: row.get("content"),
                created_at: DateTime::parse_from_rfc3339(&row.get::<String, _>("created_at"))
                    .unwrap()
                    .with_timezone(&Utc),
                password_hash: row.get("password_hash"),
            })
            .collect();

        Ok(comments)
    }

    pub async fn update_comment(
        &self,
        comment_id: &str,
        content: &str,
        password: &str,
    ) -> Result<bool, sqlx::Error> {
        let row = sqlx::query("SELECT password_hash FROM comments WHERE id = ?")
            .bind(comment_id)
            .fetch_optional(&self.pool)
            .await?;

        if let Some(row) = row {
            let stored_hash: Option<String> = row.get("password_hash");

            if let Some(stored_hash) = stored_hash {
                if !self.verify_password(password, &stored_hash) {
                    return Ok(false);
                }
            } else {
                return Ok(false);
            }

            sqlx::query("UPDATE comments SET content = ? WHERE id = ?")
                .bind(content)
                .bind(comment_id)
                .execute(&self.pool)
                .await?;

            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub async fn delete_comment(
        &self,
        comment_id: &str,
        password: &str,
    ) -> Result<bool, sqlx::Error> {
        let row = sqlx::query("SELECT password_hash FROM comments WHERE id = ?")
            .bind(comment_id)
            .fetch_optional(&self.pool)
            .await?;

        if let Some(row) = row {
            let stored_hash: Option<String> = row.get("password_hash");

            if let Some(stored_hash) = stored_hash {
                if !self.verify_password(password, &stored_hash) {
                    return Ok(false);
                }
            } else {
                return Ok(false);
            }

            sqlx::query("DELETE FROM comments WHERE id = ?")
                .bind(comment_id)
                .execute(&self.pool)
                .await?;

            Ok(true)
        } else {
            Ok(false)
        }
    }

    // Guestbook methods
    pub async fn create_guestbook_entry(
        &self,
        author: &str,
        content: &str,
        password: Option<&str>,
    ) -> Result<Guestbook, sqlx::Error> {
        let password_hash = password.map(|p| self.hash_password(p));

        let entry = Guestbook {
            id: Uuid::new_v4().to_string(),
            author: author.to_string(),
            content: content.to_string(),
            created_at: Utc::now(),
            password_hash: password_hash.clone(),
        };

        sqlx::query(
            "INSERT INTO guestbook (id, author, content, created_at, password_hash) VALUES (?, ?, ?, ?, ?)"
        )
        .bind(&entry.id)
        .bind(&entry.author)
        .bind(&entry.content)
        .bind(&entry.created_at.to_rfc3339())
        .bind(&password_hash)
        .execute(&self.pool)
        .await?;

        Ok(entry)
    }

    pub async fn get_guestbook_entries(
        &self,
        limit: Option<i32>,
    ) -> Result<Vec<Guestbook>, sqlx::Error> {
        let limit = limit.unwrap_or(50);

        let rows = sqlx::query(
            "SELECT id, author, content, created_at, password_hash FROM guestbook ORDER BY created_at DESC LIMIT ?"
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        let entries = rows
            .into_iter()
            .map(|row| Guestbook {
                id: row.get("id"),
                author: row.get("author"),
                content: row.get("content"),
                created_at: DateTime::parse_from_rfc3339(&row.get::<String, _>("created_at"))
                    .unwrap()
                    .with_timezone(&Utc),
                password_hash: row.get("password_hash"),
            })
            .collect();

        Ok(entries)
    }

    pub async fn update_guestbook_entry(
        &self,
        entry_id: &str,
        content: &str,
        password: &str,
    ) -> Result<bool, sqlx::Error> {
        let row = sqlx::query("SELECT password_hash FROM guestbook WHERE id = ?")
            .bind(entry_id)
            .fetch_optional(&self.pool)
            .await?;

        if let Some(row) = row {
            let stored_hash: Option<String> = row.get("password_hash");

            if let Some(stored_hash) = stored_hash {
                if !self.verify_password(password, &stored_hash) {
                    return Ok(false);
                }
            } else {
                return Ok(false);
            }

            sqlx::query("UPDATE guestbook SET content = ? WHERE id = ?")
                .bind(content)
                .bind(entry_id)
                .execute(&self.pool)
                .await?;

            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub async fn delete_guestbook_entry(
        &self,
        entry_id: &str,
        password: &str,
    ) -> Result<bool, sqlx::Error> {
        let row = sqlx::query("SELECT password_hash FROM guestbook WHERE id = ?")
            .bind(entry_id)
            .fetch_optional(&self.pool)
            .await?;

        if let Some(row) = row {
            let stored_hash: Option<String> = row.get("password_hash");

            if let Some(stored_hash) = stored_hash {
                if !self.verify_password(password, &stored_hash) {
                    return Ok(false);
                }
            } else {
                return Ok(false);
            }

            sqlx::query("DELETE FROM guestbook WHERE id = ?")
                .bind(entry_id)
                .execute(&self.pool)
                .await?;

            Ok(true)
        } else {
            Ok(false)
        }
    }

    // Password hashing functions
    fn hash_password(&self, password: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        password.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    fn verify_password(&self, password: &str, hash: &str) -> bool {
        self.hash_password(password) == hash
    }
}
