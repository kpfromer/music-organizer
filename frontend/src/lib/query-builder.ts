import type { SortingState } from "@tanstack/react-table";
// Note: These types will be available after running `bun run codegen`
import {
  type PaginationInput,
  SortOrder,
  type TextSearchInput,
  TrackSortField,
  type TrackSortInput,
} from "@/graphql/graphql";

/**
 * Map React Table column IDs to TrackSortField enum values
 */
const TRACK_SORT_FIELD_MAP: Record<string, TrackSortField> = {
  id: TrackSortField.Id,
  title: TrackSortField.Title,
  trackNumber: TrackSortField.TrackNumber,
  duration: TrackSortField.Duration,
  createdAt: TrackSortField.CreatedAt,
  updatedAt: TrackSortField.UpdatedAt,
} as const;

/**
 * Convert React Table sorting state to GraphQL TrackSortInput array
 */
export function buildTrackSortInput(
  sorting: SortingState,
): TrackSortInput[] | undefined {
  if (sorting.length === 0) {
    return undefined;
  }

  return sorting.map((sort) => {
    const field = TRACK_SORT_FIELD_MAP[sort.id];
    if (!field) {
      // Fallback to CreatedAt if field not found
      return {
        field: TrackSortField.CreatedAt,
        order: sort.desc ? SortOrder.Desc : SortOrder.Asc,
      };
    }

    return {
      field,
      order: sort.desc ? SortOrder.Desc : SortOrder.Asc,
    };
  });
}

/**
 * Build PaginationInput from page and pageSize
 */
export function buildPaginationInput(
  page: number,
  pageSize: number,
): PaginationInput {
  return {
    page,
    pageSize,
  };
}

/**
 * Build TextSearchInput from search string
 */
export function buildTextSearchInput(
  search: string | undefined,
): TextSearchInput | undefined {
  if (!search || search.trim() === "") {
    return undefined;
  }

  return {
    search: search.trim(),
  };
}

/**
 * Get the default sort for tracks (CreatedAt Desc)
 */
export function getDefaultTrackSort(): TrackSortInput[] {
  return [
    {
      field: TrackSortField.CreatedAt,
      order: SortOrder.Desc,
    },
  ];
}
