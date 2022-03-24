use rocket::{
    form::{Form, Strict},
    response::Redirect,
    routes, Build, Rocket, State,
};

use crate::{forms, models, DBState};

#[post("/register", data = "<form>")]
async fn register_api(
    form: Form<Strict<forms::RegisterForm>>,
    db_state: &State<DBState>,
) -> Redirect {
    let pool = &db_state.pool;
    models::User::create(
        pool,
        form.username.clone(),
        form.email.clone(),
        form.password.clone(),
    )
    .await;

    Redirect::to("/")
}

pub fn mount_routes(app: Rocket<Build>) -> Rocket<Build> {
    app.mount("/api", routes![register_api])
}
