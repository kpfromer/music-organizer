use async_graphql::{Enum, InputObject};
use sea_orm::{ColumnTrait, Condition, EntityTrait, QueryFilter, QueryOrder, QuerySelect};

#[allow(unused_imports)] // Used in tests
use crate::entities;

pub mod entity_sort_fields;
pub use entity_sort_fields::*;

// ============================================================================
// GraphQL Input Types
// ============================================================================

#[derive(Enum, Copy, Clone, Debug, Eq, PartialEq)]
pub enum SortOrder {
    Asc,
    Desc,
}

#[allow(dead_code)] // Will be used when integrated into GraphQL resolvers
#[derive(InputObject, Clone, Debug)]
pub struct PaginationInput {
    pub page: Option<i32>,
    pub page_size: Option<i32>,
}

#[allow(dead_code)] // Will be used when integrated into GraphQL resolvers
#[derive(InputObject, Clone, Debug)]
pub struct TextSearchInput {
    pub search: Option<String>,
}

// ============================================================================
// Trait for Type-Safe Sort Fields
// ============================================================================

#[allow(dead_code)] // Used by implementations and helper functions
pub trait SortableField: Sized + Copy + Clone {
    type Column: ColumnTrait;

    /// Convert the enum variant to the corresponding Sea-ORM column
    fn to_column(self) -> Self::Column;

    /// Get the default sort field and order
    fn default_sort() -> (Self, SortOrder);
}

// ============================================================================
// Generic Helper Functions
// ============================================================================

/// Apply pagination to a query with defaults and bounds checking.
/// Returns the modified query along with the calculated page and page_size.
#[allow(dead_code)] // Will be used when integrated into GraphQL resolvers
pub fn apply_pagination<T: EntityTrait>(
    query: sea_orm::Select<T>,
    page: Option<i32>,
    page_size: Option<i32>,
) -> (sea_orm::Select<T>, usize, usize) {
    let page = page.unwrap_or(1).max(1) as usize;
    let page_size = page_size.unwrap_or(25).clamp(1, 100) as usize;
    let offset = (page.saturating_sub(1)) * page_size;

    (
        query.limit(page_size as u64).offset(offset as u64),
        page,
        page_size,
    )
}

/// Apply sorting to a query using type-safe sort field enums.
/// If no sort inputs are provided, applies the default sort.
#[allow(dead_code)] // Will be used when integrated into GraphQL resolvers
pub fn apply_sort<T, F>(
    mut query: sea_orm::Select<T>,
    sort_inputs: &[SortInput<F>],
    default_sort: Option<(F, SortOrder)>,
) -> Result<sea_orm::Select<T>, color_eyre::Report>
where
    T: EntityTrait,
    F: SortableField<Column = <T as sea_orm::EntityTrait>::Column>,
{
    if sort_inputs.is_empty() {
        if let Some((field, order)) = default_sort {
            let column = field.to_column();
            query = match order {
                SortOrder::Asc => query.order_by_asc(column),
                SortOrder::Desc => query.order_by_desc(column),
            };
        }
        return Ok(query);
    }

    // Apply multiple sorts (first has highest priority)
    for sort in sort_inputs {
        let column = sort.field.to_column();
        query = match sort.order {
            SortOrder::Asc => query.order_by_asc(column),
            SortOrder::Desc => query.order_by_desc(column),
        };
    }

    Ok(query)
}

/// Apply case-insensitive substring search to a single column using SQLite's LIKE operator.
#[allow(dead_code)] // Will be used when integrated into GraphQL resolvers
pub fn apply_text_search<T, C>(
    query: sea_orm::Select<T>,
    column: C,
    search_term: &str,
) -> sea_orm::Select<T>
where
    T: EntityTrait,
    C: ColumnTrait,
{
    if search_term.is_empty() {
        return query;
    }

    // SQLite's LIKE is case-insensitive for ASCII characters by default
    // Use %pattern% for substring matching
    let pattern = format!("%{}%", search_term);
    query.filter(column.like(&pattern))
}

/// Apply case-insensitive substring search across multiple columns (OR condition).
/// Matches if the search term appears in any of the provided columns.
#[allow(dead_code)] // Will be used when integrated into GraphQL resolvers
pub fn apply_multi_column_text_search<T>(
    query: sea_orm::Select<T>,
    columns: Vec<impl ColumnTrait>,
    search_term: &str,
) -> sea_orm::Select<T>
where
    T: EntityTrait,
{
    if search_term.is_empty() || columns.is_empty() {
        return query;
    }

    let pattern = format!("%{}%", search_term);
    let mut condition = Condition::any();

    for column in columns {
        condition = condition.add(column.like(&pattern));
    }

    query.filter(condition)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // Pagination Tests
    // ========================================================================

    #[test]
    fn test_pagination_defaults() {
        let query = entities::track::Entity::find();
        let (_, page, page_size) = apply_pagination(query, None, None);
        assert_eq!(page, 1);
        assert_eq!(page_size, 25);
    }

    #[test]
    fn test_pagination_custom_values() {
        let query = entities::track::Entity::find();
        let (_, page, page_size) = apply_pagination(query, Some(3), Some(50));
        assert_eq!(page, 3);
        assert_eq!(page_size, 50);
    }

    #[test]
    fn test_pagination_page_bounds() {
        let query = entities::track::Entity::find();
        // Page 0 should become 1
        let (_, page, _) = apply_pagination(query.clone(), Some(0), None);
        assert_eq!(page, 1);

        // Negative page should become 1
        let (_, page, _) = apply_pagination(query.clone(), Some(-5), None);
        assert_eq!(page, 1);
    }

    #[test]
    fn test_pagination_page_size_bounds() {
        let query = entities::track::Entity::find();
        // Page size 0 should become 1
        let (_, _, page_size) = apply_pagination(query.clone(), None, Some(0));
        assert_eq!(page_size, 1);

        // Page size > 100 should be clamped to 100
        let (_, _, page_size) = apply_pagination(query.clone(), None, Some(200));
        assert_eq!(page_size, 100);

        // Negative page size should become 1
        let (_, _, page_size) = apply_pagination(query, None, Some(-10));
        assert_eq!(page_size, 1);
    }

    #[test]
    fn test_pagination_offset_calculation() {
        let query = entities::track::Entity::find();
        let (_query, page, page_size) = apply_pagination(query, Some(3), Some(25));
        // Page 3 with page_size 25 should have offset of 50
        // We can't directly test offset, but we can verify the pagination values
        assert_eq!(page, 3);
        assert_eq!(page_size, 25);
    }

    // ========================================================================
    // Sort Application Tests
    // ========================================================================

    #[test]
    fn test_apply_sort_with_default() {
        let query = entities::track::Entity::find();
        let result = apply_sort::<entities::track::Entity, TrackSortField>(
            query,
            &[],
            Some(TrackSortField::default_sort()),
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_apply_sort_with_single_field() {
        let query = entities::track::Entity::find();
        let sort_inputs = vec![SortInput {
            field: TrackSortField::Title,
            order: SortOrder::Asc,
        }];
        let result = apply_sort(query, &sort_inputs, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_apply_sort_with_multiple_fields() {
        let query = entities::track::Entity::find();
        let sort_inputs = vec![
            SortInput {
                field: TrackSortField::Title,
                order: SortOrder::Asc,
            },
            SortInput {
                field: TrackSortField::CreatedAt,
                order: SortOrder::Desc,
            },
        ];
        let result = apply_sort(query, &sort_inputs, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_sort_input_conversion() {
        let track_input = TrackSortInput {
            field: TrackSortField::Title,
            order: SortOrder::Asc,
        };
        let sort_input: SortInput<TrackSortField> = track_input.into();
        assert_eq!(sort_input.field, TrackSortField::Title);
        assert_eq!(sort_input.order, SortOrder::Asc);

        let playlist_input = PlaylistSortInput {
            field: PlaylistSortField::Name,
            order: SortOrder::Desc,
        };
        let sort_input: SortInput<PlaylistSortField> = playlist_input.into();
        assert_eq!(sort_input.field, PlaylistSortField::Name);
        assert_eq!(sort_input.order, SortOrder::Desc);
    }

    #[test]
    fn test_apply_sort_empty_without_default() {
        let query = entities::track::Entity::find();
        let result = apply_sort::<entities::track::Entity, TrackSortField>(query, &[], None);
        assert!(result.is_ok());
    }

    // ========================================================================
    // Text Search Tests
    // ========================================================================

    #[test]
    fn test_apply_text_search_empty_term() {
        let query = entities::track::Entity::find();
        let _result = apply_text_search(query, entities::track::Column::Title, "");
        // Should return query unchanged (no filter applied for empty term)
        // We can't easily compare Select types, so just verify it doesn't panic
    }

    #[test]
    fn test_apply_text_search_simple_term() {
        let query = entities::track::Entity::find();
        let _result = apply_text_search(query, entities::track::Column::Title, "test");
        // Query should be modified (we can't easily test the SQL without a DB)
        // But we can verify it doesn't panic and returns a query
    }

    #[test]
    fn test_apply_text_search_special_characters() {
        let query = entities::track::Entity::find();
        // Test with special characters that might need escaping
        let _result = apply_text_search(query, entities::track::Column::Title, "test%_");
        // Should handle special characters
    }

    #[test]
    fn test_apply_multi_column_text_search_empty_term() {
        let query = entities::playlist::Entity::find();
        let columns = vec![
            entities::playlist::Column::Name,
            entities::playlist::Column::Description,
        ];
        let _result = apply_multi_column_text_search(query, columns, "");
        // Should return query unchanged (no filter applied for empty term)
        // We can't easily compare Select types, so just verify it doesn't panic
    }

    #[test]
    fn test_apply_multi_column_text_search_single_column() {
        let query = entities::playlist::Entity::find();
        let columns = vec![entities::playlist::Column::Name];
        let _result = apply_multi_column_text_search(query, columns, "test");
        // Query building succeeded
    }

    #[test]
    fn test_apply_multi_column_text_search_multiple_columns() {
        let query = entities::playlist::Entity::find();
        let columns = vec![
            entities::playlist::Column::Name,
            entities::playlist::Column::Description,
        ];
        let _result = apply_multi_column_text_search(query, columns, "test");
        // Query building succeeded
    }

    #[test]
    fn test_apply_multi_column_text_search_empty_columns() {
        let query = entities::playlist::Entity::find();
        let columns: Vec<entities::playlist::Column> = vec![];
        let _result = apply_multi_column_text_search(query, columns, "test");
        // Should return query unchanged when no columns provided
        // We can't easily compare Select types, so just verify it doesn't panic
    }

    // ========================================================================
    // Query Building Verification Tests
    // ========================================================================

    #[test]
    fn test_query_building_chain() {
        // Test that we can chain multiple query building operations
        let mut query = entities::track::Entity::find();

        // Apply search
        query = apply_text_search(query, entities::track::Column::Title, "test");

        // Apply sort
        let sort_inputs = vec![SortInput {
            field: TrackSortField::Title,
            order: SortOrder::Asc,
        }];
        query = apply_sort(query, &sort_inputs, None).unwrap();

        // Apply pagination
        let (_query, page, page_size) = apply_pagination(query, Some(1), Some(25));

        // Verify pagination values
        assert_eq!(page, 1);
        assert_eq!(page_size, 25);
    }

    #[test]
    fn test_multi_column_search_condition_building() {
        let query = entities::playlist::Entity::find();
        let columns = vec![
            entities::playlist::Column::Name,
            entities::playlist::Column::Description,
        ];
        let _result = apply_multi_column_text_search(query, columns, "test");
        // Verify query is built (condition should be applied)
    }
}
