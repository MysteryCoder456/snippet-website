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
}

#[derive(Serialize)]
pub struct SnippetDetailContext {
    pub user: Option<models::User>,
    pub snippet: models::CodeSnippet,
    pub flash: Option<(String, String)>,
}
