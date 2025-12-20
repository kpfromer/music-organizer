use std::sync::Arc;

use async_graphql::http::GraphiQLSource;
use async_graphql::{EmptyMutation, EmptySubscription, Object, Schema};
use axum::response::{Html, IntoResponse};

use crate::http_server::state::AppState;
use color_eyre::Result;

pub struct Query;

#[Object]
impl Query {
    async fn howdy(&self) -> &'static str {
        "partner"
    }

    async fn error_example(&self) -> Result<&'static str> {
        Err(color_eyre::eyre::eyre!(
            "This is a test error from the graphql schema"
        ))
    }
}

pub async fn graphql() -> impl IntoResponse {
    Html(GraphiQLSource::build().endpoint("/graphql").finish())
}

pub fn create_schema(app_state: Arc<AppState>) -> Schema<Query, EmptyMutation, EmptySubscription> {
    Schema::build(Query, EmptyMutation, EmptySubscription)
        .data(app_state)
        .finish()
}
