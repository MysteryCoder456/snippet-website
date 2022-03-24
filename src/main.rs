use std::collections::HashMap;

#[macro_use]
extern crate rocket;

use rocket::{fs::FileServer, routes, State};
use rocket_dyn_templates::Template;
use sqlx::postgres::{PgPool, PgPoolOptions};

mod api;
mod forms;
mod models;

struct DBState {
    pool: PgPool,
}

#[get("/")]
async fn index(db_state: &State<DBState>) -> Template {
    let pool = &db_state.pool;
    let snippets = models::CodeSnippet::query_all(pool).await;

    let ctx: HashMap<&str, _> = HashMap::from_iter([("code_snippets", snippets)]);
    Template::render("home", ctx)
}

#[launch]
async fn rocket() -> _ {
    dotenv::dotenv().ok();

    let db_url = env!("DATABASE_URL");
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(db_url)
        .await
        .unwrap();

    let mut app = rocket::build()
        .attach(Template::fairing())
        .manage(DBState { pool })
        .mount("/", routes![index])
        .mount("/static", FileServer::from("static"));

    app = api::mount_routes(app);
    app
}
