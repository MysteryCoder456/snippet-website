use rocket::form::Context;
use serde::Serialize;

use crate::models;

#[derive(Serialize)]
pub struct IndexContext {
    pub user: Option<models::User>,
    pub code_snippets: Vec<models::CodeSnippet>,
    pub flash: Option<(String, String)>,
}

#[derive(Serialize)]
pub struct RegisterContext<'a, 'b> {
    pub form: &'a Context<'b>,
}

#[derive(Serialize)]
pub struct LoginContext<'a, 'b> {
    pub form: &'a Context<'b>,
    pub flash: Option<(String, String)>,
}

#[derive(Serialize)]
pub struct AddSnippetContext<'a, 'b> {
    pub user: models::User,
    pub form: &'a Context<'b>,
    pub flash: Option<(String, String)>,
}

#[derive(Serialize)]
pub struct SnippetDetailContext<'a, 'b> {
    pub user: Option<models::User>,
    pub snippet: models::CodeSnippet,
    pub liked: Option<bool>,
    pub like_count: usize,
    pub comments: Vec<models::Comment>,
    pub form: Option<&'a Context<'b>>,
    pub flash: Option<(String, String)>,
}

#[derive(Serialize)]
pub struct ProfileContext {
    pub user: Option<models::User>,
    pub requested_user: models::User,
    pub avatar_image_url: String,
    pub first_snippet: Option<models::CodeSnippet>,
    pub latest_snippet: Option<models::CodeSnippet>,
}

#[derive(Serialize)]
pub struct EditProfileContext<'a, 'b> {
    pub user: models::User,
    pub avatar_image_url: String,
    pub form: &'a Context<'b>,
    pub flash: Option<(String, String)>,
}

#[derive(Serialize)]
pub struct ChannelsListContext {
    pub user: models::User,
    pub channels: Vec<models::Channel>,
    pub flash: Option<(String, String)>,
}

#[derive(Serialize)]
pub struct AddChannelContext<'a, 'b> {
    pub user: models::User,
    pub form: &'a Context<'b>,
    pub flash: Option<(String, String)>,
}

#[derive(Serialize)]
pub struct ChannelMessagesContext<'a, 'b> {
    pub user: models::User,
    pub channel: models::Channel,
    pub messages: Vec<models::Message>,
    pub form: &'a Context<'b>,
    pub flash: Option<(String, String)>,
}
