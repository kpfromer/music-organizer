# Phase 2 — Library Browse

> Read-only GraphQL + frontend browse views. Empty DB ⇒ empty UI. Manually inserted rows render correctly. No mutations, no playback.

## Goals

- `async-graphql` wired into `mm-server` at `POST /graphql`.
- Schema covers `Artist`, `Album`, `Track`, `File`, `FileAudioInfo`, `CoverArt` and their queries (TDD 09 §Object types, §Queries — the read parts).
- `mm-library` service crate implements the read paths.
- DataLoaders set up for N+1-prone fields.
- Frontend has graphql-codegen wired; typed query hooks generated.
- Routes: `/library/tracks`, `/library/albums`, `/library/albums/$id`, `/library/artists`, `/library/artists/$id` rendering real (if empty) data.
- Sidebar navigates between sections.
- Search params drive list filters/pagination.

## Acceptance criteria

- [ ] `pnpm graphql-codegen` produces `frontend/src/lib/gql/generated.ts` from a `frontend/schema.graphql` exported from the binary.
- [ ] CI runs `cargo run --bin export-schema > frontend/schema.graphql` and `git diff --exit-code` to verify no drift.
- [ ] Empty DB → all browse views render their empty states.
- [ ] Manually `INSERT` an artist + album + track + file via sqlite3; refresh the UI; the entity is visible in all expected views.
- [ ] Track list paginates (insert 250 fixtures, verify offset/limit works and `hasMore` flips correctly).
- [ ] `/library/albums?search=foo` filters; URL param survives reload.
- [ ] Artist detail shows top tracks + albums.
- [ ] No mutations defined yet (verify by introspection).
- [ ] `query-keys.ts` registry has entries for every query used.

## TDD references

- TDD 09 §Schema overview, §Queries, §Pagination, §DataLoader
- TDD 10 §Routes, §Data fetching, §Layout
- TDD 13 §Directory layout, §Query keys, §Codegen workflow

## Out of scope

- Mutations of any kind.
- Playback (clicking a track does nothing useful yet, or queues to a no-op player).
- Cover art rendering with real images — the `CoverArt.url` resolver returns a URL, but `/api/cover_art/:id` doesn't exist yet → broken images. Use the SVG-initials placeholder always for this phase.
- Search across types (`/search` route is phase 10).
- Sorting beyond default (album-tracks by disc/number, etc. handled in resolvers; no sort-by-column UI).

## PR breakdown

**2.1 — async-graphql bootstrap + schema export** (~350)
Wire async-graphql into mm-server, `POST /graphql`, empty `Query` root, `bin/export-schema.rs`, CI step that exports + diffs.

**2.2 — `mm-library` crate skeleton + Artist queries** (~400)
New crate, Artist GraphQL type + `artist`/`artists` queries with offset/limit pagination + `ArtistConnection`.

**2.3 — Album queries** (~450)
Album type + `album`/`albums` queries (with `artistId` filter) + `AlbumConnection`. Album.artists resolver.

**2.4 — Track + File queries** (~500)
Track + File + FileAudioInfo types, `track`/`tracks` queries with filters, Track.album / Track.artists / Track.files resolvers.

**2.5 — DataLoaders** (~300)
Loaders for Track.album, Track.artists, Album.artists, Track.files. Verify batching with a test that issues N parent reads + asserts 1 child query each.

**2.6 — Frontend codegen pipeline** (~350)
graphql-codegen config, generated `lib/gql/generated.ts`, `lib/gql/client.ts` fetcher, `lib/query-keys.ts` registry skeleton, top-level `QueryClient` + provider.

**2.7 — `/library/tracks` route** (~450)
Route, query hook, table component composing `components/tables/data-table.tsx`, search-param filter, pagination footer, empty state.

**2.8 — `/library/albums` list + detail** (~500)
List route (grid component), detail route, AlbumDetail component with track-list section, cover-art placeholder.

**2.9 — `/library/artists` list + detail** (~450)
List route, detail route with top-tracks + albums sections.

## Risks / unknowns

- **graphql-codegen + TanStack Query integration.** Use `@graphql-codegen/typescript-react-query` plugin. Confirm it generates hooks that accept a `queryKey` override.
- **Schema export step.** Add a `bin/export-schema.rs` that prints `Schema::sdl()` and exits. Wire into CI.
- **DataLoader + SeaORM.** `async-graphql`'s `DataLoader` works with any backend; the loader function does the batched query. Spike: verify the artists-per-track loader actually batches.
- **Pagination shape.** Connection types with `totalCount` mean every list query runs `SELECT COUNT(*)` — for v2's library size this is fine, but flag if it ever becomes slow.

## Smoke test

```bash
# From a fresh DB seeded with fixtures:
sqlite3 $MM_DB_PATH < scripts/dev-fixtures.sql

# Start server + frontend dev mode
cargo run -p mm-server &
cd frontend && pnpm dev

# Visit http://localhost:5173/library/tracks
# Expect: list of seeded tracks with artist + album + duration
# Click an artist → /library/artists/$id, see top tracks + albums
# Search: type in URL `?search=Pearl`, list filters
```
