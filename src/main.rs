#[macro_use]
extern crate rocket;

use std::collections::HashMap;

use rocket::{
    form::{Context, Contextual, Error, Form},
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

    let ctx: HashMap<_, _> = HashMap::from_iter([("code_snippets", snippets)]);
    Template::render("home", ctx)
}

#[get("/register")]
fn register() -> Template {
    Template::render("register", &Context::default())
}

#[post("/register", data = "<form>")]
async fn register_api(
    mut form: Form<Contextual<'_, forms::RegisterForm<'_>>>,
    db_state: &State<DBState>,
) -> (Status, Template) {
    let template: Template = match form.value {
        Some(ref register_user) => {
            let pool = &db_state.pool;

            let username = register_user.username;
            let email = register_user.email;
            let password = register_user.password;

            let (username_valid, email_valid) = models::User::verify(pool, username, email).await;

            if !username_valid {
                let error = Error::validation("Username already taken").with_name("username");
                form.context.push_error(error);
            }

            if !email_valid {
                let error = Error::validation("Email already being used").with_name("email");
                form.context.push_error(error);
            }

            if username_valid && email_valid {
                models::User::create(pool, username, email, password).await;
                login()
            } else {
                Template::render("register", &form.context)
            }
        }
        None => Template::render("register", &form.context),
    };

    (form.context.status(), template)
}

#[get("/login")]
fn login() -> Template {
    Template::render("login", &Context::default())
}

#[post("/login", data = "<form>")]
async fn login_api(mut form: Form<Contextual<'_, forms::LoginForm<'_>>>, db_state: &State<DBState>) -> (Status, Template) {
    let template: Template = todo!();
    (form.context.status(), template)
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
        .mount("/", routes![index, register, register_api, login, login_api])
        .mount("/static", FileServer::from("static"))
}
