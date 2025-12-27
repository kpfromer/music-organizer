use std::sync::Arc;

use async_graphql::http::GraphiQLSource;
use async_graphql::{EmptySubscription, Object, Schema};
use axum::response::{Html, IntoResponse};

use crate::http_server::graphql_error::GraphqlResult;
use crate::http_server::state::AppState;

pub mod soulseek_mutations;

use soulseek_mutations::Mutation;

pub struct Query;

#[Object]
impl Query {
    async fn howdy(&self) -> &'static str {
        "partner"
    }

    async fn error_example(&self) -> GraphqlResult<&'static str> {
        Err(color_eyre::eyre::eyre!("This is a test error from the graphql schema").into())
    }
}

pub async fn graphql() -> impl IntoResponse {
    Html(GraphiQLSource::build().endpoint("/graphql").finish())
}

pub fn create_schema(app_state: Arc<AppState>) -> Schema<Query, Mutation, EmptySubscription> {
    Schema::build(Query, Mutation, EmptySubscription)
        .data(app_state)
        .finish()
}
