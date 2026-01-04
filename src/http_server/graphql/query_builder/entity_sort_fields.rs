use async_graphql::{Enum, InputObject};

use super::{SortOrder, SortableField};
use crate::entities;

// ============================================================================
// Entity-Specific Sort Field Enums
// ============================================================================

#[derive(Enum, Copy, Clone, Debug, Eq, PartialEq)]
#[graphql(name = "TrackSortField")]
pub enum TrackSortField {
    Id,
    Title,
    TrackNumber,
    Duration,
    CreatedAt,
    UpdatedAt,
}

#[derive(Enum, Copy, Clone, Debug, Eq, PartialEq)]
#[graphql(name = "PlaylistSortField")]
pub enum PlaylistSortField {
    Id,
    Name,
    CreatedAt,
    UpdatedAt,
}

// ============================================================================
// Entity-Specific Sort Input Types
// ============================================================================

// Entity-specific sort input types (async_graphql doesn't support generic InputObjects)
#[allow(dead_code)] // Will be used when integrated into GraphQL resolvers
#[derive(InputObject, Clone, Debug)]
pub struct TrackSortInput {
    pub field: TrackSortField,
    pub order: SortOrder,
}

#[allow(dead_code)] // Will be used when integrated into GraphQL resolvers
#[derive(InputObject, Clone, Debug)]
pub struct PlaylistSortInput {
    pub field: PlaylistSortField,
    pub order: SortOrder,
}

// Internal generic sort input for use in helper functions
#[allow(dead_code)] // Used internally by helper functions
pub struct SortInput<F> {
    pub field: F,
    pub order: SortOrder,
}

impl From<TrackSortInput> for SortInput<TrackSortField> {
    fn from(input: TrackSortInput) -> Self {
        Self {
            field: input.field,
            order: input.order,
        }
    }
}

impl From<PlaylistSortInput> for SortInput<PlaylistSortField> {
    fn from(input: PlaylistSortInput) -> Self {
        Self {
            field: input.field,
            order: input.order,
        }
    }
}

// ============================================================================
// SortableField Implementations
// ============================================================================

impl SortableField for TrackSortField {
    type Column = entities::track::Column;

    fn to_column(self) -> Self::Column {
        match self {
            TrackSortField::Id => entities::track::Column::Id,
            TrackSortField::Title => entities::track::Column::Title,
            TrackSortField::TrackNumber => entities::track::Column::TrackNumber,
            TrackSortField::Duration => entities::track::Column::Duration,
            TrackSortField::CreatedAt => entities::track::Column::CreatedAt,
            TrackSortField::UpdatedAt => entities::track::Column::UpdatedAt,
        }
    }

    fn default_sort() -> (Self, SortOrder) {
        (TrackSortField::CreatedAt, SortOrder::Desc)
    }
}

impl SortableField for PlaylistSortField {
    type Column = entities::playlist::Column;

    fn to_column(self) -> Self::Column {
        match self {
            PlaylistSortField::Id => entities::playlist::Column::Id,
            PlaylistSortField::Name => entities::playlist::Column::Name,
            PlaylistSortField::CreatedAt => entities::playlist::Column::CreatedAt,
            PlaylistSortField::UpdatedAt => entities::playlist::Column::UpdatedAt,
        }
    }

    fn default_sort() -> (Self, SortOrder) {
        (PlaylistSortField::CreatedAt, SortOrder::Desc)
    }
}
