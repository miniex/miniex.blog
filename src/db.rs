use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{sqlite::SqlitePoolOptions, Pool, Row, Sqlite};
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
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(database_url)
            .await?;

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

        // Create indexes for query performance
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_comments_post_id ON comments(post_id)")
            .execute(&pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_guestbook_created_at ON guestbook(created_at)")
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
        .bind(comment.created_at.to_rfc3339())
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
                self.rehash_if_legacy("comments", "id", comment_id, password, &stored_hash)
                    .await?;
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
                self.rehash_if_legacy("comments", "id", comment_id, password, &stored_hash)
                    .await?;
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
        .bind(entry.created_at.to_rfc3339())
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

    pub async fn get_guestbook_entries_paged(
        &self,
        offset: i32,
        limit: i32,
        sort_asc: bool,
    ) -> Result<Vec<Guestbook>, sqlx::Error> {
        let order = if sort_asc { "ASC" } else { "DESC" };
        let query = format!(
            "SELECT id, author, content, created_at, password_hash FROM guestbook ORDER BY created_at {} LIMIT ? OFFSET ?",
            order
        );

        let rows = sqlx::query(&query)
            .bind(limit)
            .bind(offset)
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

    pub async fn count_guestbook_entries(&self) -> Result<u32, sqlx::Error> {
        let row: (i32,) = sqlx::query_as("SELECT COUNT(*) FROM guestbook")
            .fetch_one(&self.pool)
            .await?;
        Ok(row.0 as u32)
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
                self.rehash_if_legacy("guestbook", "id", entry_id, password, &stored_hash)
                    .await?;
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
                self.rehash_if_legacy("guestbook", "id", entry_id, password, &stored_hash)
                    .await?;
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
        use argon2::password_hash::rand_core::OsRng;
        use argon2::password_hash::SaltString;
        use argon2::{Argon2, PasswordHasher};

        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        argon2
            .hash_password(password.as_bytes(), &salt)
            .expect("password hashing failed")
            .to_string()
    }

    fn verify_password(&self, password: &str, stored_hash: &str) -> bool {
        if stored_hash.starts_with("$argon2") {
            use argon2::password_hash::PasswordHash;
            use argon2::{Argon2, PasswordVerifier};
            let parsed = PasswordHash::new(stored_hash).ok();
            parsed
                .map(|h| {
                    Argon2::default()
                        .verify_password(password.as_bytes(), &h)
                        .is_ok()
                })
                .unwrap_or(false)
        } else {
            // Legacy DefaultHasher format: verify for transparent migration
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};
            let mut hasher = DefaultHasher::new();
            password.hash(&mut hasher);
            format!("{:x}", hasher.finish()) == stored_hash
        }
    }

    async fn rehash_if_legacy(
        &self,
        table: &str,
        id_column: &str,
        id: &str,
        password: &str,
        stored_hash: &str,
    ) -> Result<(), sqlx::Error> {
        if !stored_hash.starts_with("$argon2") {
            let new_hash = self.hash_password(password);
            let query = format!(
                "UPDATE {} SET password_hash = ? WHERE {} = ?",
                table, id_column
            );
            sqlx::query(&query)
                .bind(&new_hash)
                .bind(id)
                .execute(&self.pool)
                .await?;
        }
        Ok(())
    }
}
