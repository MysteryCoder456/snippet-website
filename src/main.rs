#[macro_use]
extern crate rocket;

use rocket::{
    form::{Context, Contextual, Error, Form, Errors, name::{Name, NameBuf}},
    fs::FileServer,
    http::{Cookie, CookieJar, Status},
    response::Redirect,
    routes, State,
};
use rocket_dyn_templates::Template;
use serde::Serialize;
use sqlx::postgres::{PgPool, PgPoolOptions};
use indexmap::{IndexMap, IndexSet};

mod forms;
mod models;

struct DBState {
    pool: PgPool,
}

#[derive(Serialize)]
struct IndexContext {
    user: Option<models::User>,
    code_snippets: Vec<models::CodeSnippet>,
}

#[derive(Default, Serialize)]
struct AddSnippetContext<'v> {
    user: Option<models::User>,
    errors: IndexMap<NameBuf<'v>, Errors<'v>>,
    values: IndexMap<&'v Name, Vec<&'v str>>,
    data_fields: IndexSet<&'v Name>,
    form_errors: Errors<'v>,
    #[serde(skip)]
    status: Status,

}

#[get("/")]
async fn index(db_state: &State<DBState>, user: Option<models::User>) -> Template {
    let pool = &db_state.pool;
    let snippets = models::CodeSnippet::query_all(pool).await;

    let ctx = IndexContext {
        user,
        code_snippets: snippets,
    };
    Template::render("home", ctx)
}

#[get("/new")]
fn add_snippet(user: models::User) -> Template {
    let mut ctx = AddSnippetContext::default();
    ctx.user = Some(user);
    Template::render("add_snippet", &ctx)
}

#[get("/register")]
fn register() -> Template {
    Template::render("register", &Context::default())
}

#[post("/register", data = "<form>")]
async fn register_api(
    mut form: Form<Contextual<'_, forms::RegisterForm<'_>>>,
    db_state: &State<DBState>,
    cookie_jar: &CookieJar<'_>,
) -> Result<Redirect, Template> {
    match form.value {
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
                let user_id = models::User::create(pool, username, email, password).await;
                let auth_cookie = Cookie::new("current_user", user_id.to_string());
                cookie_jar.add_private(auth_cookie);
                Ok(Redirect::to(uri!(index)))
            } else {
                Err(Template::render("register", &form.context))
            }
        }
        None => Err(Template::render("register", &form.context)),
    }
}

#[get("/login")]
fn login() -> Template {
    Template::render("login", &Context::default())
}

#[post("/login", data = "<form>")]
async fn login_api(
    mut form: Form<Contextual<'_, forms::LoginForm<'_>>>,
    db_state: &State<DBState>,
    cookie_jar: &CookieJar<'_>,
) -> Result<Redirect, Template> {
    match form.value {
        Some(ref login_user) => {
            let pool = &db_state.pool;

            let username = login_user.username;
            let password = login_user.password;

            let auth_result = models::User::authenticate(pool, username, password).await;

            match auth_result {
                Ok(user_id) => {
                    let auth_cookie = Cookie::new("current_user", user_id.to_string());
                    cookie_jar.add_private(auth_cookie);
                    Ok(Redirect::to(uri!(index)))
                }
                Err((name, error)) => {
                    let e = Error::validation(error).with_name(name);
                    form.context.push_error(e);
                    Err(Template::render("login", &form.context))
                }
            }
        }
        None => Err(Template::render("login", &form.context)),
    }
}

#[get("/logout")]
fn logout(cookie_jar: &CookieJar<'_>) -> Redirect {
    cookie_jar.remove_private(Cookie::named("current_user"));
    Redirect::to(uri!(index))
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
        .mount(
            "/",
            routes![index, add_snippet, register, register_api, login, login_api, logout],
        )
        .mount("/static", FileServer::from("static"))
}
