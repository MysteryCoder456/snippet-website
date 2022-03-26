use rand::{distributions::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

fn generate_salt() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(8)
        .map(char::from)
        .collect()
}

fn generate_hash(password: &str, salt: &str) -> String {
    let mut hash = sha256::digest(password.to_owned() + salt);

    // 5 rounds of SHA256 hashing
    for _ in 1..=5 {
        hash = sha256::digest(hash);
    }

    hash
}

#[derive(Serialize, Deserialize)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub password: String,
    pub created_at: i64,
    pub salt: String,
}

impl User {
    pub async fn verify(pool: &PgPool, username: &str, email: &str) -> (bool, bool) {
        let result = sqlx::query!(
            r#"
                SELECT *
                FROM users
                WHERE username = $1 OR email = $2
            "#,
            username,
            email
        )
        .fetch_one(pool)
        .await;

        if let Ok(user) = result {
            (user.username != username, user.email != email)
        } else {
            (true, true)
        }
    }

    pub async fn create(pool: &PgPool, username: &str, email: &str, password: &str) -> User {
        let (username, email) = (username.trim(), email.trim());
        let salt = generate_salt();
        let hashed_password = generate_hash(password, salt.as_str());

        let result = sqlx::query!(
            r#"
            INSERT INTO users (username, email, passwd, salt)
            VALUES ($1, $2, $3, $4)
            RETURNING id, created_at
            "#,
            username,
            email,
            hashed_password,
            salt
        )
        .fetch_one(pool)
        .await
        .unwrap();

        // TODO: Make a function to get user from row
        User {
            id: result.id,
            username: username.to_owned(),
            email: email.to_owned(),
            password: password.to_owned(),
            created_at: result.created_at.timestamp(),
            salt,
        }
    }

    pub async fn from_id(pool: &PgPool, id: i32) -> Option<User> {
        let result = sqlx::query!("SELECT * FROM users WHERE id = $1", id)
            .fetch_one(pool)
            .await
            .ok()?;

        Some(User {
            id: result.id,
            username: result.username,
            email: result.email,
            password: result.passwd,
            created_at: result.created_at.timestamp(),
            salt: result.salt,
        })
    }
}

#[derive(Serialize, Deserialize)]
pub struct CodeSnippet {
    pub id: i32,
    pub author: User,
    pub title: String,
    pub code: String,
    pub language: String,
    pub created_at: i64,
    pub updated_at: i64,
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
                created_at: record.created_at.timestamp(),
                updated_at,
            });
        }

        snippets
    }
}
