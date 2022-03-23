use serde::{Deserialize, Serialize};
use sqlx::PgPool;

#[derive(Serialize, Deserialize)]
pub struct User {
    id: i32,
    username: String,
    email: String,
    passwd: String,
    created_at: i64,
}

impl User {
    pub async fn from_id(pool: &PgPool, id: i32) -> Option<User> {
        let result = sqlx::query!("SELECT * FROM users WHERE id = $1", id)
            .fetch_one(pool)
            .await
            .ok()?;

        Some(User {
            id: result.id,
            username: result.username,
            email: result.email,
            passwd: result.passwd,
            created_at: result.created_at?.timestamp(),
        })
    }
}

#[derive(Serialize, Deserialize)]
pub struct CodeSnippet {
    id: i32,
    author: User,
    title: String,
    code: String,
    language: String,
    created_at: i64,
    updated_at: i64,
}

impl CodeSnippet {
    pub async fn query_all(pool: &PgPool) -> Vec<CodeSnippet> {
        let results = sqlx::query!("SELECT * FROM code_snippets")
            .fetch_all(pool)
            .await
            .unwrap();
        let mut snippets = Vec::<CodeSnippet>::new();

        for record in results {
            let updated_at = if record.updated_at.is_some() {
                record.updated_at.unwrap().timestamp()
            } else {
                0
            };

            snippets.push(CodeSnippet {
                id: record.id,
                author: User::from_id(pool, record.author_id).await.unwrap(),
                title: record.title,
                code: record.code,
                language: record.lang.unwrap(),
                created_at: record.created_at.unwrap().timestamp(),
                updated_at,
            });
        }

        snippets
    }
}
