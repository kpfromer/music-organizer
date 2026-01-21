use color_eyre::eyre::Result;
use sea_orm::prelude::Expr;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use tracing::instrument;

use crate::{
    database::Database,
    entities::{self, spotify_to_local_matcher_tasks::MatchTaskStatus},
};

/// Cleans up spotify to local tracks tasks that where not completed.
#[instrument(skip(db))]
pub async fn cleanup_tasks(db: &Database) -> Result<()> {
    let tasks = entities::spotify_to_local_matcher_tasks::Entity::update_many()
        .col_expr(
            entities::spotify_to_local_matcher_tasks::Column::Status,
            Expr::value(MatchTaskStatus::Killed),
        )
        .filter(
            entities::spotify_to_local_matcher_tasks::Column::Status
                .is_in(vec![MatchTaskStatus::Pending, MatchTaskStatus::InProgress]),
        )
        .exec(&db.conn)
        .await?;
    tracing::info!("Set {} tasks to killed", tasks.rows_affected);
    Ok(())
}
