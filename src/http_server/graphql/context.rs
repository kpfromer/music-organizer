use std::sync::Arc;

use async_graphql::Context;

use crate::http_server::{graphql_error::GraphqlError, state::AppState};

pub fn get_app_state<'a>(ctx: &Context<'a>) -> Result<&'a Arc<AppState>, GraphqlError> {
    ctx.data::<Arc<AppState>>()
        .map_err(|_| GraphqlError::FailedToGetAppState)
}
