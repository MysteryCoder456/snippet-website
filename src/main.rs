use rocket::{routes, State};
use sqlx::postgres::PgPool;

#[macro_use]
extern crate rocket;

struct DBState {
    pool: PgPool,
}

#[get("/")]
async fn index(db_state: &State<DBState>) -> String {
    let pool = &db_state.pool;
    let snippets = sqlx::query!("SELECT * FROM code_snippets")
        .fetch_all(pool)
        .await
        .unwrap();

    snippets
        .iter()
        .map(|snippet| format!("{}", snippet.title))
        .collect::<Vec<String>>()
        .join("\n")
}

#[launch]
async fn rocket() -> _ {
    dotenv::dotenv().ok();

    let db_url = std::env::var("DATABASE_URL").unwrap();
    let pool = PgPool::connect(db_url.as_str()).await.unwrap();

    rocket::build()
        .manage(DBState { pool })
        .mount("/", routes![index])
}
