use color_eyre::eyre::Result;
use sea_orm::prelude::Expr;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use tracing::instrument;

use crate::entities::wishlist_item::WishlistStatus;
use crate::{
    database::Database,
    entities::{self, spotify_to_local_matcher_tasks::MatchTaskStatus},
};

/// Cleans up spotify to local tracks tasks that were not completed.
#[instrument(skip(db))]
async fn cleanup_spotify_to_local_matcher_tasks(db: &Database) -> Result<()> {
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

/// Resets wishlist items that were not completed. This will allow them to be retried.
#[instrument(skip(db))]
async fn reset_wishlist_items(db: &Database) -> Result<()> {
    let items = entities::wishlist_item::Entity::update_many()
        .col_expr(
            entities::wishlist_item::Column::Status,
            Expr::value(WishlistStatus::Failed),
        )
        .filter(entities::wishlist_item::Column::Status.is_not_in(vec![WishlistStatus::Completed]))
        .exec(&db.conn)
        .await?;
    tracing::info!("Reset {} wishlist items", items.rows_affected);
    Ok(())
}

#[instrument(skip(db))]
pub async fn cleanup_tasks(db: &Database) -> Result<()> {
    cleanup_spotify_to_local_matcher_tasks(db).await?;
    reset_wishlist_items(db).await?;
    Ok(())
}
