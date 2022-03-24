#[macro_use]
extern crate rocket;

use std::collections::HashMap;

use rocket::{
    form::{Context, Contextual, Form},
    fs::FileServer,
    http::Status,
    routes, State,
};
use rocket_dyn_templates::Template;
use sqlx::postgres::{PgPool, PgPoolOptions};

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

#[get("/register")]
fn register() -> Template {
    Template::render("register", &Context::default())
}

#[post("/register", data = "<form>")]
async fn register_api(
    form: Form<Contextual<'_, forms::RegisterForm<'_>>>,
    db_state: &State<DBState>,
) -> (Status, Template) {
    if let Some(ref register_user) = form.value {
        let pool = &db_state.pool;

        let username = register_user.username.to_owned();
        let email = register_user.email.to_owned();
        let password = register_user.password.to_owned();

        models::User::create(pool, username, email, password).await;
    }

    (
        form.context.status(),
        Template::render("register", &form.context),
    )
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

    rocket::build()
        .attach(Template::fairing())
        .manage(DBState { pool })
        .mount("/", routes![index, register, register_api])
        .mount("/static", FileServer::from("static"))
}
