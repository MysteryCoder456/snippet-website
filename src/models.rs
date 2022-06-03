use rand::{distributions::Alphanumeric, Rng};
use rocket::{
    futures::future::join_all,
    http::Cookie,
    outcome::IntoOutcome,
    request::{FromRequest, Outcome, Request},
};
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

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub password: String,
    pub created_at: i64,
    pub salt: String,

    // Profile fields
    pub bio: String,
    pub occupation: String,
    pub avatar_path: Option<String>,
}

impl User {
    pub async fn authenticate(
        pool: &PgPool,
        username: &str,
        password: &str,
    ) -> Result<i32, (&'static str, &'static str)> {
        let result = sqlx::query!(
            r#"SELECT id, salt, passwd FROM users WHERE username = $1"#,
            username
        )
        .fetch_one(pool)
        .await;

        if let Ok(record) = result {
            let salt = record.salt.as_str();
            let hashed = generate_hash(password, salt);

            if hashed == record.passwd {
                Ok(record.id)
            } else {
                Err(("password", "Incorrect password"))
            }
        } else {
            Err(("username", "Invalid username"))
        }
    }

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

    pub async fn create(pool: &PgPool, username: &str, email: &str, password: &str) -> i32 {
        let (username, email) = (username.trim(), email.trim());
        let salt = generate_salt();
        let hashed_password = generate_hash(password, salt.as_str());

        let record = sqlx::query!(
            r#"
            INSERT INTO users (username, email, passwd, salt)
            VALUES ($1, $2, $3, $4)
            RETURNING id
            "#,
            username,
            email,
            hashed_password,
            salt
        )
        .fetch_one(pool)
        .await
        .unwrap();

        record.id
    }

    pub async fn from_id(pool: &PgPool, id: i32, auth_details: bool) -> Option<Self> {
        let result = sqlx::query!("SELECT * FROM users WHERE id = $1", id)
            .fetch_one(pool)
            .await
            .ok()?;

        let (passwd, salt) = if auth_details {
            (result.passwd, result.salt)
        } else {
            ("".to_owned(), "".to_owned())
        };

        Some(Self {
            id: result.id,
            username: result.username,
            email: result.email,
            password: passwd,
            created_at: result.created_at.timestamp(),
            salt,
            bio: result.bio,
            occupation: result.occupation,
            avatar_path: result.avatar_path,
        })
    }

    pub async fn from_username(pool: &PgPool, username: &str) -> Option<Self> {
        let result = sqlx::query!("SELECT * FROM users WHERE username = $1", username)
            .fetch_one(pool)
            .await
            .ok()?;

        Some(Self {
            id: result.id,
            username: result.username,
            email: result.email,
            password: result.passwd,
            created_at: result.created_at.timestamp(),
            salt: result.salt,
            bio: result.bio,
            occupation: result.occupation,
            avatar_path: result.avatar_path,
        })
    }

    pub async fn get_oldest_snippet(&self, pool: &PgPool) -> Option<CodeSnippet> {
        let result = sqlx::query!(
            "SELECT * FROM code_snippets WHERE author_id = $1 ORDER BY created_at ASC",
            self.id
        )
        .fetch_one(pool)
        .await
        .ok()?;

        Some(CodeSnippet {
            id: result.id,
            author: self.clone(),
            title: result.title,
            code: result.code,
            language: result.lang,
            created_at: result.created_at.timestamp(),
            updated_at: result.updated_at.map(|updated| updated.timestamp()),
        })
    }

    pub async fn get_newest_snippet(&self, pool: &PgPool) -> Option<CodeSnippet> {
        let result = sqlx::query!(
            "SELECT * FROM code_snippets WHERE author_id = $1 ORDER BY created_at DESC",
            self.id
        )
        .fetch_one(pool)
        .await
        .ok()?;

        Some(CodeSnippet {
            id: result.id,
            author: self.clone(),
            title: result.title,
            code: result.code,
            language: result.lang,
            created_at: result.created_at.timestamp(),
            updated_at: result.updated_at.map(|updated| updated.timestamp()),
        })
    }

    pub fn display_avatar_path(&self) -> String {
        if let Some(ref avatar_path) = self.avatar_path {
            avatar_path.clone()
        } else {
            "/static/images/default_avatar.png".to_owned()
        }
    }

    pub async fn edit_profile(
        &mut self,
        pool: &PgPool,
        bio: &str,
        occupation: &str,
        avatar_path: Option<String>,
    ) {
        self.bio = bio.to_owned();
        self.occupation = occupation.to_owned();
        self.avatar_path = avatar_path.clone().map(|p| p.to_owned());

        sqlx::query!(
            "UPDATE users SET bio = $1, occupation = $2, avatar_path = $3 WHERE id = $4",
            bio,
            occupation,
            avatar_path,
            self.id
        )
        .execute(pool)
        .await
        .unwrap();
    }

    pub async fn get_channels(&self, pool: &PgPool) -> Vec<Channel> {
        let records = sqlx::query!(
            r#"
            SELECT * FROM channels
            WHERE channels.id IN (
                SELECT channel_id FROM channels_users
                WHERE user_id = $1
            )"#,
            self.id
        )
        .fetch_all(pool)
        .await
        .unwrap();

        let mut channels = vec![];

        // TODO: Perhaps use Channel::from_id function instead of this loop
        for record in records {
            let member_id_records = sqlx::query!(
                r#"SELECT user_id FROM channels_users WHERE channel_id = $1"#,
                record.id
            )
            .fetch_all(pool)
            .await
            .unwrap();

            let member_fetch_futures = member_id_records
                .iter()
                .map(|r| User::from_id(pool, r.user_id, false));

            let members = join_all(member_fetch_futures)
                .await
                .iter()
                .filter_map(|u: &Option<_>| u.to_owned())
                .collect::<Vec<_>>();

            channels.push(Channel {
                id: record.id,
                name: record.name.clone(),
                members,
            });
        }

        channels
    }
}

impl PartialEq for User {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for User {
    type Error = &'r str;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let db_state = req.rocket().state::<crate::DBState>().unwrap();
        let cookie_jar = req.cookies();

        if let Some(auth_cookie) = cookie_jar.get_private("current_user") {
            let user_id = auth_cookie.value().parse::<i32>().unwrap();
            Self::from_id(&db_state.pool, user_id, true)
                .await
                .or_forward(())
        } else {
            let post_login_cookie = Cookie::new("post_login_uri", req.uri().path().to_string());
            cookie_jar.add(post_login_cookie);
            Outcome::Forward(())
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct CodeSnippet {
    pub id: i32,
    pub author: User,
    pub title: String,
    pub code: String,
    pub language: String,
    pub created_at: i64,
    pub updated_at: Option<i64>,
}

impl CodeSnippet {
    pub async fn query_all(pool: &PgPool) -> Vec<Self> {
        let results = sqlx::query!("SELECT * FROM code_snippets ORDER BY created_at DESC")
            .fetch_all(pool)
            .await
            .unwrap();
        let mut snippets = Vec::<CodeSnippet>::new();

        for record in results {
            let author: User =
                if let Some(user) = User::from_id(pool, record.author_id, false).await {
                    user
                } else {
                    continue;
                };

            snippets.push(CodeSnippet {
                id: record.id,
                author,
                title: record.title,
                code: record.code,
                language: record.lang,
                created_at: record.created_at.timestamp(),
                updated_at: record.updated_at.map(|updated| updated.timestamp()),
            });
        }

        snippets
    }

    pub async fn from_id(pool: &PgPool, id: i32) -> Option<Self> {
        let record = sqlx::query!("SELECT * FROM code_snippets WHERE id = $1", id)
            .fetch_one(pool)
            .await
            .ok()?;

        Some(CodeSnippet {
            id: record.id,
            author: User::from_id(pool, record.author_id, false).await?,
            title: record.title,
            code: record.code,
            language: record.lang,
            created_at: record.created_at.timestamp(),
            updated_at: record.updated_at.map(|updated| updated.timestamp()),
        })
    }

    pub async fn create(
        pool: &PgPool,
        author: &User,
        title: &str,
        language: &str,
        code: &str,
    ) -> i32 {
        sqlx::query!(
            r#"
            INSERT INTO code_snippets (author_id, title, lang, code)
            VALUES ($1, $2, $3, $4)
            RETURNING id
            "#,
            author.id,
            title,
            language,
            code
        )
        .fetch_one(pool)
        .await
        .unwrap()
        .id
    }

    pub async fn get_comments(&self, pool: &PgPool) -> Vec<Comment> {
        let results = sqlx::query!(
            r#"SELECT * FROM comments WHERE code_snippet_id = $1"#,
            self.id
        )
        .fetch_all(pool)
        .await
        .unwrap();

        let mut comments = vec![];

        for record in results {
            if let Some(author) = User::from_id(pool, record.author_id, false).await {
                comments.push(Comment {
                    id: record.id,
                    code_snippet: self.clone(),
                    author: author.clone(),
                    author_avatar_url: author.display_avatar_path(),
                    content: record.content.clone(),
                });
            }
        }

        comments
    }
}

#[derive(Serialize, Deserialize)]
pub struct Comment {
    pub id: i32,
    pub code_snippet: CodeSnippet,
    pub author: User,
    pub author_avatar_url: String,
    pub content: String,
}

impl Comment {
    pub async fn create(pool: &PgPool, snippet_id: i32, author_id: i32, content: &str) -> i32 {
        sqlx::query!(
            r#"
            INSERT INTO comments (code_snippet_id, author_id, content)
            VALUES ($1, $2, $3)
            RETURNING id
            "#,
            snippet_id,
            author_id,
            content
        )
        .fetch_one(pool)
        .await
        .unwrap()
        .id
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Message {
    pub id: i32,
    pub sender: User,
    pub channel: Channel,
    pub content: String,
    pub sent_at: i64,
}

impl Message {
    pub async fn create(pool: &PgPool, channel_id: i32, sender_id: i32, content: &str) -> i32 {
        sqlx::query!(
            r#"
            INSERT INTO messages (channel_id, sender_id, content)
            VALUES ($1, $2, $3)
            RETURNING id
            "#,
            channel_id,
            sender_id,
            content
        )
        .fetch_one(pool)
        .await
        .unwrap()
        .id
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Channel {
    pub id: i32,
    pub name: String,
    pub members: Vec<User>,
}

impl Channel {
    pub async fn create(pool: &PgPool, name: &str, initial_member_ids: Vec<i32>) -> i32 {
        // Create new channel
        let record = sqlx::query!(
            r#"INSERT INTO channels (name) VALUES ($1) RETURNING id"#,
            name
        )
        .fetch_one(pool)
        .await
        .unwrap();

        // Add users to channel
        for user_id in initial_member_ids {
            let _ = sqlx::query!(
                r#"INSERT INTO channels_users VALUES ($1, $2)"#,
                record.id,
                user_id
            )
            .execute(pool)
            .await;
        }

        record.id
    }

    pub async fn from_id(pool: &PgPool, channel_id: i32) -> Option<Self> {
        let record = sqlx::query!(
            r#"
            SELECT * FROM channels
            WHERE id = $1
            "#,
            channel_id
        )
        .fetch_one(pool)
        .await
        .ok()?;

        let member_id_records = sqlx::query!(
            r#"SELECT user_id FROM channels_users WHERE channel_id = $1"#,
            channel_id
        )
        .fetch_all(pool)
        .await
        .unwrap();

        let member_fetch_futures = member_id_records
            .iter()
            .map(|r| User::from_id(pool, r.user_id, false));

        let members = join_all(member_fetch_futures)
            .await
            .iter()
            .filter_map(|u: &Option<_>| u.to_owned())
            .collect::<Vec<_>>();

        Some(Channel {
            id: record.id,
            name: record.name.clone(),
            members,
        })
    }

    pub async fn get_all_messages(&self, pool: &PgPool) -> Vec<Message> {
        sqlx::query!(
            r#"
            SELECT
                messages.id, messages.sender_id, messages.content, messages.sent_at,
                users.username, users.email, users.created_at, users.bio, users.occupation, users.avatar_path
            FROM messages
            INNER JOIN users ON messages.sender_id = users.id
            WHERE messages.channel_id = $1
            ORDER BY sent_at ASC
            "#,
            self.id
        )
        .fetch_all(pool)
        .await
        .unwrap()
        .iter()
        .map(|r| {
            let sender = User {
                id: r.sender_id,
                username: r.username.clone(),
                email: r.email.clone(),
                created_at: r.created_at.timestamp(),
                bio: r.bio.clone(),
                occupation: r.occupation.clone(),
                avatar_path: r.avatar_path.clone(),
                ..Default::default()
            };

            Message {
                id: r.id,
                sender,
                channel: self.clone(),
                content: r.content.clone(),
                sent_at: r.sent_at.timestamp(),
            }
        })
        .collect()
    }
}
