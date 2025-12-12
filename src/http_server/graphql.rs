use std::sync::Arc;

use async_graphql::http::GraphiQLSource;
use async_graphql::{Context, EmptyMutation, EmptySubscription, Object, Schema};
use axum::response::{Html, IntoResponse};
use color_eyre::eyre::{Result, WrapErr};
use sea_orm::EntityTrait;

use crate::entities;
use crate::http_server::state::AppState;

pub struct Query;

#[Object]
impl Query {
    async fn howdy(&self) -> &'static str {
        "partner"
    }

    async fn unimportable_files(
        &self,
        ctx: &Context<'_>,
    ) -> Result<Vec<entities::unimportable_file::Model>> {
        let app_state = ctx.data::<Arc<AppState>>().unwrap();
        let db = &app_state.db;

        let r = entities::unimportable_file::Entity::find()
            .all(&db.conn)
            .await
            .wrap_err("Failed to get unimportable files");

        // TODO: remove this once we have a proper error handling
        log::error!("Failed to get unimportable files");
        log::error!("{r:?}");
        r
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
