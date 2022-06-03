#[macro_use]
extern crate rocket;

use std::time::{SystemTime, UNIX_EPOCH};

use piston_rs::{Client, Executor, File};
use rocket::{
    form::{Context, Contextual, Error, Form},
    fs::FileServer,
    http::{Cookie, CookieJar},
    request::FlashMessage,
    response::{
        stream::{Event, EventStream},
        Flash, Redirect,
    },
    routes,
    serde::json::{json, Value},
    Shutdown, State,
};
use rocket_dyn_templates::Template;
use sqlx::postgres::{PgPool, PgPoolOptions};
use tokio::{
    select,
    sync::broadcast::{channel, error::RecvError, Sender},
};
use uuid::Uuid;

mod contexts;
mod forms;
mod models;

struct DBState {
    pool: PgPool,
}

// TODO: Code prettify option when posting.

#[get("/")]
async fn index(
    db_state: &State<DBState>,
    user: Option<models::User>,
    flash: Option<FlashMessage<'_>>,
) -> Template {
    let pool = &db_state.pool;
    let snippets = models::CodeSnippet::query_all(pool).await;

    let ctx = contexts::IndexContext {
        user,
        code_snippets: snippets,
        flash: flash.map(|f| f.into_inner()),
    };
    Template::render("home", ctx)
}

#[get("/new")]
fn add_snippet(user: models::User, flash: Option<FlashMessage<'_>>) -> Template {
    let ctx = contexts::AddSnippetContext {
        user,
        form: &Context::default(),
        flash: flash.map(|f| f.into_inner()),
    };
    Template::render("add_snippet", &ctx)
}

#[get("/new", rank = 2)]
fn add_snippet_no_auth() -> Flash<Redirect> {
    Flash::warning(
        Redirect::to(uri!(login)),
        "You must login to make a new snippet",
    )
}

#[post("/new", data = "<form>")]
async fn add_snippet_api(
    form: Form<Contextual<'_, forms::AddSnippetForm<'_>>>,
    db_state: &State<DBState>,
    user: models::User,
) -> Result<Flash<Redirect>, Template> {
    match form.value {
        Some(ref new_snippet) => {
            let pool = &db_state.pool;

            let title = new_snippet.title;
            let language = new_snippet.language;
            let code = new_snippet.code;

            let snippet_id = models::CodeSnippet::create(pool, &user, title, language, code).await;
            Ok(Flash::success(
                Redirect::to(uri!(snippet_detail(id = snippet_id))),
                "Created new snippet!",
            ))
        }
        None => {
            let ctx = contexts::AddSnippetContext {
                user,
                form: &form.context,
                flash: None,
            };
            Err(Template::render("add_snippet", &ctx))
        }
    }
}

#[get("/snippet/<id>")]
async fn snippet_detail(
    id: i32,
    db_state: &State<DBState>,
    user: Option<models::User>,
    flash: Option<FlashMessage<'_>>,
) -> Option<Template> {
    let pool = &db_state.pool;
    let snippet = models::CodeSnippet::from_id(pool, id).await?;
    let comments = snippet.get_comments(pool).await;

    let form_ctx = Context::default();
    let form = if user.is_some() {
        Some(&form_ctx)
    } else {
        None
    };

    let ctx = contexts::SnippetDetailContext {
        user,
        snippet,
        comments,
        form,
        flash: flash.map(|f| f.into_inner()),
    };
    Some(Template::render("snippet_detail", &ctx))
}

#[post("/snippet/<snippet_id>", data = "<form>")]
async fn add_comment_api(
    snippet_id: i32,
    user: models::User,
    db_state: &State<DBState>,
    form: Form<Contextual<'_, forms::AddCommentForm<'_>>>,
) -> Option<Result<Redirect, Template>> {
    let pool = &db_state.pool;

    match form.value {
        Some(ref new_comment) => {
            let _new_comment_id =
                models::Comment::create(pool, snippet_id, user.id, new_comment.content).await;
            // TODO - Focus on new comment when created
            Some(Ok(Redirect::to(uri!(snippet_detail(id = snippet_id)))))
        }
        None => {
            let snippet = models::CodeSnippet::from_id(pool, snippet_id).await?;
            let comments = snippet.get_comments(pool).await;

            let ctx = contexts::SnippetDetailContext {
                user: Some(user),
                snippet,
                comments,
                form: Some(&form.context),
                flash: None,
            };
            Some(Err(Template::render("snippet_detail", &ctx)))
        }
    }
}

#[get("/snippet/<id>/run")]
async fn snippet_run(id: i32, db_state: &State<DBState>) -> Option<String> {
    let pool = &db_state.pool;
    let snippet = models::CodeSnippet::from_id(pool, id).await?;

    let piston_client = Client::new();
    let executor = Executor::new()
        .set_language(&snippet.language.to_lowercase())
        .set_version("*")
        .add_file(File::default().set_content(&snippet.code));

    let response = piston_client.execute(&executor).await.ok()?;
    let result = match response.compile {
        Some(c) => format!("{}{}", c.output, response.run.output),
        None => response.run.output,
    };
    Some(result)
}

#[get("/msg")]
async fn channels_list(
    db_state: &State<DBState>,
    user: models::User,
    flash: Option<FlashMessage<'_>>,
) -> Template {
    let pool = &db_state.pool;
    let channels = user.get_channels(pool).await;

    let ctx = contexts::ChannelsListContext {
        user,
        channels,
        flash: flash.map(|f| f.into_inner()),
    };
    Template::render("channels_list", &ctx)
}

#[get("/msg", rank = 2)]
fn channels_list_no_auth() -> Flash<Redirect> {
    Flash::warning(
        Redirect::to(uri!(login)),
        "You must login to access your channels",
    )
}

#[get("/msg/new")]
fn add_channel(user: models::User, flash: Option<FlashMessage<'_>>) -> Template {
    let ctx = contexts::AddChannelContext {
        user,
        form: &Context::default(),
        flash: flash.map(|f| f.into_inner()),
    };
    Template::render("add_channel", &ctx)
}

#[get("/msg/new", rank = 2)]
fn add_channel_no_auth() -> Flash<Redirect> {
    Flash::warning(
        Redirect::to(uri!(login)),
        "You must login to create new channels",
    )
}

#[post("/msg/new", data = "<form>")]
async fn add_channel_api(
    db_state: &State<DBState>,
    user: models::User,
    form: Form<Contextual<'_, forms::NewChannelForm<'_>>>,
) -> Result<Flash<Redirect>, Template> {
    match form.value {
        Some(ref new_channel) => {
            let pool = &db_state.pool;
            let mut initial_member_ids = vec![user.id];

            if let Some(initial_members_split) = new_channel.initial_members.map(|m| m.split(",")) {
                for username in initial_members_split {
                    if let Some(u) = models::User::from_username(pool, username.trim()).await {
                        // Prevent duplication of current user
                        if user.id != u.id {
                            initial_member_ids.push(u.id);
                        }
                    }
                }
            }

            let new_channel_id =
                models::Channel::create(pool, new_channel.name, initial_member_ids).await;
            Ok(Flash::success(
                Redirect::to(uri!(channel_messages(channel_id = new_channel_id))),
                "Successfully created new channel!",
            ))
        }
        None => {
            let ctx = contexts::AddChannelContext {
                user,
                form: &form.context,
                flash: None,
            };
            Err(Template::render("add_channel", &ctx))
        }
    }
}

#[get("/msg/<channel_id>")]
async fn channel_messages(
    channel_id: i32,
    db_state: &State<DBState>,
    user: models::User,
    flash: Option<FlashMessage<'_>>,
) -> Option<Template> {
    let pool = &db_state.pool;
    let channel = models::Channel::from_id(pool, channel_id).await?;
    let messages = channel.get_all_messages(pool).await;

    let ctx = contexts::ChannelMessagesContext {
        user,
        channel,
        messages,
        form: &Context::default(),
        flash: flash.map(|f| f.into_inner()),
    };
    Some(Template::render("channel_messages", &ctx))
}

#[post("/msg/<channel_id>/send", data = "<form>")]
async fn message_send_api(
    channel_id: i32,
    db_state: &State<DBState>,
    user: models::User,
    form: Form<Contextual<'_, forms::MessageSendForm<'_>>>,
    queue: &State<Sender<models::Message>>,
) -> Value {
    match form.value {
        Some(ref new_message) => {
            let pool = &db_state.pool;
            let channel = models::Channel::from_id(pool, channel_id).await;

            if !channel.is_some() {
                return json!({
                    "status": 404,
                    "message": "Channel not found!",
                });
            }

            let channel = channel.unwrap();

            if channel.members.contains(&user) {
                let new_msg_id =
                    models::Message::create(pool, channel.id, user.id, new_message.content).await;

                let _ = queue.send(models::Message {
                    id: new_msg_id,
                    sender: user.clone(),
                    channel: channel.clone(),
                    content: new_message.content.to_owned(),
                    sent_at: SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs() as i64,
                });

                json!({
                    "status": 200,
                    "message": "Successfully sent message",
                    "message_id": new_msg_id,
                    "channel_id": channel.id,
                })
            } else {
                json!({
                    "status": 403,
                    "message": "User is not in channel",
                })
            }
        }
        None => json!({
            "status": 400,
            "message": "Please provide a 'content' field",
        }),
    }
}

#[get("/msg/<channel_id>/events")]
async fn message_events(
    channel_id: i32,
    user: models::User,
    db_state: &State<DBState>,
    queue: &State<Sender<models::Message>>,
    mut end: Shutdown,
) -> Option<EventStream![]> {
    let pool = &db_state.pool;
    let channel = models::Channel::from_id(pool, channel_id).await?;

    // Don't initiate event stream if user is not a member
    if !channel.members.contains(&user) {
        return None;
    }

    let mut rx = queue.subscribe();

    Some(EventStream! {
        loop {
            let msg = select! {
                msg = rx.recv() => match msg {
                    Ok(msg) => {
                        if msg.channel.id == channel.id {
                            msg
                        } else {
                            continue
                        }
                    },
                    Err(RecvError::Closed) => break,
                    Err(RecvError::Lagged(_)) => continue,
                },
                _ = &mut end => break,
            };

            yield Event::json(&msg);
        }
    })
}

#[get("/profile/<user_id>")]
async fn profile(
    user_id: i32,
    db_state: &State<DBState>,
    user: Option<models::User>,
) -> Option<Template> {
    let pool = &db_state.pool;

    let requested_user = models::User::from_id(pool, user_id).await?;
    let avatar_image_url = requested_user.display_avatar_path();
    let first_snippet = requested_user.get_oldest_snippet(pool).await;
    let latest_snippet = requested_user.get_newest_snippet(pool).await;

    let ctx = contexts::ProfileContext {
        user,
        requested_user,
        avatar_image_url,
        first_snippet,
        latest_snippet,
    };
    Some(Template::render("profile", &ctx))
}

#[get("/profile/edit")]
async fn edit_profile(user: models::User, flash: Option<FlashMessage<'_>>) -> Template {
    let avatar_image_url = user.display_avatar_path();

    let ctx = contexts::EditProfileContext {
        user,
        avatar_image_url,
        form: &Context::default(),
        flash: flash.map(|f| f.into_inner()),
    };
    Template::render("edit_profile", &ctx)
}

#[get("/profile/edit", rank = 2)]
fn edit_profile_no_auth() -> Flash<Redirect> {
    Flash::warning(
        Redirect::to(uri!(login)),
        "You must login to edit your profile",
    )
}

#[post("/profile/edit", data = "<form>")]
async fn edit_profile_api(
    mut form: Form<Contextual<'_, forms::EditProfileFrom<'_>>>,
    db_state: &State<DBState>,
    mut user: models::User,
) -> Result<Redirect, Template> {
    let pool = &db_state.pool;

    match form.value {
        Some(ref mut new_profile) => {
            let bio = new_profile.bio;
            let occupation = new_profile.occupation;
            let mut new_avatar_path = user.avatar_path.clone();

            if let Some(avatar_name) = new_profile.avatar.raw_name() {
                let allowed_formats = ["png", "jpg", "jpeg"];

                if let Some((_, ext_name)) = avatar_name
                    .dangerous_unsafe_unsanitized_raw()
                    .as_str()
                    .rsplit_once('.')
                {
                    if allowed_formats.contains(&ext_name.to_lowercase().as_str()) {
                        let avatar_uuid = Uuid::new_v4();
                        new_avatar_path = Some(format!(
                            "/site_media/profile_avatars/{}.{}",
                            avatar_uuid, ext_name
                        ));

                        // Remove existing avatar if exists
                        if let Some(old_avatar_path) = &user.avatar_path {
                            let fs_path = old_avatar_path.replacen('/', "", 1);

                            if std::path::Path::new(&fs_path).exists() {
                                tokio::spawn(async move {
                                    match tokio::fs::remove_file(fs_path).await {
                                        Ok(_) => {}
                                        Err(e) => println!("Couldn't delete old avatar: {}", e),
                                    }
                                });
                            }
                        }

                        new_profile
                            .avatar
                            .persist_to(new_avatar_path.as_ref().unwrap().replacen('/', "", 1))
                            .await
                            .unwrap();
                    } else {
                        let formats_str = allowed_formats.join(", ");
                        let error = Error::validation(format!(
                            "Invalid file format: {}. Must be {}.",
                            ext_name.to_uppercase(),
                            formats_str.to_uppercase()
                        ))
                        .with_name("avatar");
                        form.context.push_error(error);

                        let profile_image_url = user.display_avatar_path();
                        let ctx = contexts::EditProfileContext {
                            user,
                            avatar_image_url: profile_image_url,
                            form: &form.context,
                            flash: None,
                        };
                        return Err(Template::render("edit_profile", &ctx));
                    }
                }
            }

            user.edit_profile(pool, bio, occupation, new_avatar_path.clone())
                .await;
            Ok(Redirect::to(uri!(profile(user_id = user.id))))
        }
        None => {
            let profile_image_url = user.display_avatar_path();
            let ctx = contexts::EditProfileContext {
                user,
                avatar_image_url: profile_image_url,
                form: &form.context,
                flash: None,
            };
            Err(Template::render("edit_profile", &ctx))
        }
    }
}

#[get("/register")]
fn register() -> Template {
    let ctx = contexts::RegisterContext {
        form: &Context::default(),
    };
    Template::render("register", &ctx)
}

#[post("/register", data = "<form>")]
async fn register_api(
    mut form: Form<Contextual<'_, forms::RegisterForm<'_>>>,
    db_state: &State<DBState>,
    cookie_jar: &CookieJar<'_>,
) -> Result<Flash<Redirect>, Template> {
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
                // Insert new user into database
                let user_id = models::User::create(pool, username, email, password).await;

                let auth_cookie = Cookie::new("current_user", user_id.to_string());
                cookie_jar.add_private(auth_cookie);

                if let Some(post_login_cookie) = cookie_jar.get("post_login_uri") {
                    let uri = post_login_cookie.value().to_owned();
                    cookie_jar.remove(Cookie::named("post_login_uri"));
                    Ok(Flash::success(
                        Redirect::to(uri),
                        "New account created. Welcome to Snippet!",
                    ))
                } else {
                    Ok(Flash::success(
                        Redirect::to(uri!(index)),
                        "New account created. Welcome to Snippet!",
                    ))
                }
            } else {
                let ctx = contexts::RegisterContext {
                    form: &form.context,
                };
                Err(Template::render("register", &ctx))
            }
        }
        None => {
            let ctx = contexts::RegisterContext {
                form: &form.context,
            };
            Err(Template::render("register", &ctx))
        }
    }
}

#[get("/login")]
fn login(flash: Option<FlashMessage<'_>>) -> Template {
    let ctx = contexts::LoginContext {
        form: &Context::default(),
        flash: flash.map(|f| f.into_inner()),
    };
    Template::render("login", &ctx)
}

#[post("/login", data = "<form>")]
async fn login_api(
    mut form: Form<Contextual<'_, forms::LoginForm<'_>>>,
    db_state: &State<DBState>,
    cookie_jar: &CookieJar<'_>,
) -> Result<Flash<Redirect>, Template> {
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

                    if let Some(post_login_cookie) = cookie_jar.get("post_login_uri") {
                        let uri = post_login_cookie.value().to_owned();
                        cookie_jar.remove(Cookie::named("post_login_uri"));
                        Ok(Flash::success(Redirect::to(uri), "Logged in successfully!"))
                    } else {
                        Ok(Flash::success(
                            Redirect::to(uri!(index)),
                            "Logged in successfully!",
                        ))
                    }
                }
                Err((name, error)) => {
                    let e = Error::validation(error).with_name(name);
                    form.context.push_error(e);

                    let ctx = contexts::LoginContext {
                        form: &form.context,
                        flash: None,
                    };
                    Err(Template::render("login", &ctx))
                }
            }
        }
        None => {
            let ctx = contexts::LoginContext {
                form: &form.context,
                flash: None,
            };
            Err(Template::render("login", &ctx))
        }
    }
}

#[get("/logout")]
fn logout(cookie_jar: &CookieJar<'_>) -> Flash<Redirect> {
    cookie_jar.remove_private(Cookie::named("current_user"));
    Flash::warning(
        Redirect::to(uri!(index)),
        "You have been logged out! Log back in to enjoy all the features of Snippet.",
    )
}

#[launch]
async fn rocket() -> _ {
    dotenv::dotenv().ok();

    // Ensure all the required directories are present
    let required_dirs = ["site_media/profile_avatars"];
    for dir in required_dirs {
        if !std::path::Path::new(dir).exists() {
            match tokio::fs::create_dir_all(dir).await {
                Ok(_) => println!("Created {} directory", dir),
                Err(e) => eprintln!("Unable to create directory {}:\n{}", dir, e),
            }
        }
    }

    let db_url = env!("DATABASE_URL");
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(db_url)
        .await
        .unwrap();

    rocket::build()
        .attach(Template::fairing())
        .manage(DBState { pool })
        .manage(channel::<models::Message>(1024).0)
        .mount(
            "/",
            routes![
                index,
                add_snippet,
                add_snippet_no_auth,
                add_snippet_api,
                snippet_detail,
                add_comment_api,
                snippet_run,
                channels_list,
                channels_list_no_auth,
                channel_messages,
                message_send_api,
                message_events,
                add_channel,
                add_channel_no_auth,
                add_channel_api,
                profile,
                edit_profile,
                edit_profile_no_auth,
                edit_profile_api,
                register,
                register_api,
                login,
                login_api,
                logout,
            ],
        )
        .mount("/static", FileServer::from("static"))
        .mount("/site_media", FileServer::from("site_media"))
}
