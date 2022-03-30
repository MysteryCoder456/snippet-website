use indexmap::{IndexMap, IndexSet};
use rocket::form::{name::{NameBuf, Name}, Errors};
use serde::Serialize;

use crate::models;

#[derive(Serialize)]
pub struct IndexContext {
    pub user: Option<models::User>,
    pub code_snippets: Vec<models::CodeSnippet>,
}

#[derive(Default, Serialize)]
pub struct AddSnippetContext<'v> {
    pub user: Option<models::User>,
    errors: IndexMap<NameBuf<'v>, Errors<'v>>,
    values: IndexMap<&'v Name, Vec<&'v str>>,
    data_fields: IndexSet<&'v Name>,
    form_errors: Errors<'v>,
}
