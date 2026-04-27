# Phase 7 — Playlists

> Playlist CRUD, reordering, CSV mass-import, `playlist_pending` resolution. Wishlist creation in this phase only as a placeholder — actual download work happens in phase 8.

## Goals

- `mm-playlist` service crate.
- `playlist`, `playlist_track`, `playlist_pending` tables (already in TDD 02 / 07; no migration changes if Phase 1 included them).
- Mutations: `createPlaylist`, `renamePlaylist`, `deletePlaylist`, `addTracksToPlaylist`, `removeTracksFromPlaylist`, `reorderPlaylist`, `importPlaylist`.
- CSV parser (`Title`, `Artist`, `Album`, `Duration` required; `ISRC`, `Year`, `Track Number` optional).
- For each row: try local match (DB-only candidates with score ≥ 0.8) → if matched, add to `playlist_track`; else create `wishlist_item` + `playlist_pending` row.
- Frontend: `/playlists` list, `/playlists/$id` detail with reorder, `/playlists/import` mass-import flow with preview table + confirm.
- Sidebar shows playlists.
- Now-playing integration: "Play playlist" button.

## Acceptance criteria

- [ ] Create a playlist; rename it; delete it.
- [ ] Add tracks to a playlist (single + multi-select); they appear in order.
- [ ] Reorder via drag-and-drop; new positions persist.
- [ ] Remove tracks from a playlist.
- [ ] Click "Play" on a playlist → enqueues all tracks, starts playing first.
- [ ] Mass-import: paste a CSV with valid columns → preview shows matched rows + unmatched rows + parse errors.
- [ ] Confirm import → matched tracks added, unmatched create wishlist items + `playlist_pending` rows.
- [ ] Playlist detail shows "5 of 30 pending" indicator.
- [ ] CSV with missing required column → preview rejects entire file with row-pointed error.

## TDD references

- TDD 07 (entire doc — already revised to CSV-only with required columns)
- TDD 09 §Mutations (playlist subset), §Object types `Playlist`, `PlaylistTrack`, `PlaylistPending`

## Out of scope

- Wishlist worker (phase 8); imported pending items will sit at `status=PENDING` indefinitely until phase 8.
- Playlist folders, smart playlists, sharing (punted).
- Auto-resolution of `playlist_pending` → `playlist_track` on wishlist completion (lives in `mm-wishlist` and lands in phase 8).

## PR breakdown

**7.1 — `mm-playlist` skeleton + CRUD mutations** (~450)
Crate, `createPlaylist`/`renamePlaylist`/`deletePlaylist`/playlist-by-id query.

**7.2 — Track add/remove/reorder mutations** (~450)
`addTracksToPlaylist`/`removeTracksFromPlaylist`/`reorderPlaylist`. Reorder is delete-and-reinsert in transaction.

**7.3 — CSV parser** (~400)
`parse_csv(bytes) -> Result<Vec<ImportRow>, ParseError>`. Header normalization, duration-format normalization (`ms`/`s`/`M:SS`). Tests with Exportify/TuneMyMusic samples.

**7.4 — Mass-import logic** (~400)
For each row, call `mm-matching::find_candidates(db_only=true)`; high-confidence → `playlist_track`; else → `wishlist_item` + `playlist_pending` row.

**7.5 — `importPlaylist` mutation + preview** (~350)
Two-step flow: parse + preview returns `PlaylistImportPreview`; confirm executes.

**7.6 — Frontend `/playlists` list** (~400)
List view with TanStack Table, create-playlist dialog, sidebar live-updating.

**7.7 — Frontend `/playlists/$id` detail (read)** (~400)
Detail layout, track table, pending count badge, play-all button hooked to player.

**7.8 — Frontend reorder via dnd-kit** (~450)
`@dnd-kit/sortable` wrapper around the track table, optimistic update + reorder mutation.

**7.9 — Frontend `/playlists/import`** (~450)
File-upload + textarea, preview table (matched / unmatched / errors), confirm button.

## Risks / unknowns

- **DnD library choice.** TanStack Table doesn't ship with DnD; common pairings are `dnd-kit/sortable` (modern, recommended) or `react-beautiful-dnd` (older, more docs). Pick `@dnd-kit/sortable`.
- **Reorder atomicity.** TDD 07 says "delete + reinsert in transaction." Confirm SeaORM lets you batch deletes + inserts cleanly. Worst case, raw SQL via `Statement::from_sql_and_values`.
- **CSV variants.** Real exports have BOM, CRLF, quoted fields with embedded commas, empty rows. Use the `csv` crate with permissive defaults; test with Exportify and TuneMyMusic samples in fixtures.
- **Local-match threshold.** TDD 07 says ≥ 0.8 for auto-link during import. May need tuning. Surface as a constant.

## Smoke test

```bash
# 1. Create a playlist "Test", add 10 tracks from /library/tracks via multi-select context menu.
# 2. Drag track 7 to position 2; reload; verify order.
# 3. Click play; queue advances; works through all 10.
# 4. Go to /playlists/import; paste a CSV with 20 rows, half match local DB.
# 5. Verify preview, confirm.
# 6. Playlist now shows 10 tracks + "10 pending" indicator.
# 7. Delete the playlist; verify cascade removed playlist_track + playlist_pending rows.
```
