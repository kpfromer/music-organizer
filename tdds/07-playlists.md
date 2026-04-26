# 07 — Playlists

> Local playlists only. v2 has no Spotify sync. Mass-import accepts file uploads of common playlist formats and produces wishlist items + playlist tracks for whatever's already in the library.

## Data model

See TDD 02 — `playlist` (id, name, description, timestamps) and `playlist_track` (playlist_id, track_id, position, added_at).

Position is dense (no gaps). Reordering is "delete all, reinsert in new order" inside one transaction; for the library size we expect (hundreds of items per playlist max), this is fine.

## CRUD operations (GraphQL)

- `createPlaylist(name, description)` → playlist
- `renamePlaylist(id, name, description?)` → playlist
- `deletePlaylist(id)` → bool
- `addTracksToPlaylist(id, trackIds[], position?)` → playlist (appends if position null)
- `removeTracksFromPlaylist(id, trackIds[])` → playlist
- `reorderPlaylist(id, trackIds[])` → playlist (full new order)
- `playlist(id)` → playlist with paginated tracks
- `playlists(offset, limit, search?)` → paged list

## Mass-import

UI route: `/playlists/import`.

### Supported input format: CSV only

v2 supports exactly one mass-import format: **CSV**. M3U, JSON, and plain-text are punted — they're all lossy in different ways and CSV covers the actual use case (Spotify exports via Exportify / TuneMyMusic / similar).

**Required columns** (header names case-insensitive, common synonyms accepted):

| Column | Synonyms | Required | Why |
|---|---|---|---|
| Title | Track Name, Song Name | yes | |
| Artist | Artist Name(s), Artists | yes | |
| Album | Album Name, Album | **yes** | Strong disambiguation signal — required per user spec. |
| Duration | Duration (ms), Length, Track Duration | **yes** | The single best signal for matching identity (per discussion). Accepts `ms`, `s`, or `M:SS` and normalizes to ms. |

**Optional columns**: `ISRC`, `Year`, `Track Number`. Anything else is ignored.

If a row is missing any required column, the import fails the whole file with a precise error pointing at the offending row. We do not silently skip — partial imports of playlists are worse than refusing.

`mm-playlist::parse_csv(bytes)` returns `Result<Vec<ImportRow>, ParseError>`:

```rust
struct ImportRow {
  title: String,
  artist: String,
  album: String,        // required
  duration_ms: i64,     // required
  isrc: Option<String>,
  year: Option<i32>,
  track_number: Option<i32>,
  source_line: usize,   // for error reporting
}
```

### Import flow

For each row:

1. **Try match against existing tracks.** Reuse `mm-matching::find_candidates` minus the AcoustID/MB-API hops (DB-only, since we have no audio file). If best candidate score ≥ 0.8 → link directly to the playlist.
2. **No good local match.** Create a `wishlist_item` with the row's metadata. The UI shows the playlist with placeholder rows ("waiting for download") that resolve once the wishlist worker completes them.

### Result UI

After parsing, show a preview table:

| Source row | Status | Action |
|---|---|---|
| "Black — Pearl Jam" | Matched (Pearl Jam – Black, score 0.94) | un-match |
| "Yellow Ledbetter" | No match — will wishlist | edit / skip |
| "garbage row" | Couldn't parse | skip |

User confirms; we create the playlist + add matched tracks + enqueue wishlist items.

### Wishlist linkage

`playlist_track` always points to a `track`. So unmatched rows can't be in the playlist directly. We add a side table:

```
playlist_pending
  id             INTEGER PK
  playlist_id    INTEGER NOT NULL → playlist.id ON DELETE CASCADE
  position       INTEGER NOT NULL
  wishlist_item_id INTEGER NOT NULL → wishlist_item.id ON DELETE CASCADE
  created_at     INTEGER NOT NULL
```

When the wishlist worker completes an item with `resulting_track_id` set, a trigger (or a post-success hook in `mm-wishlist`) converts the matching `playlist_pending` row(s) to real `playlist_track` rows. The position carries over. If the import was numbered 1..N and pending items resolve out of order, positions are still correct.

UI shows "5 of 30 tracks pending download" on the playlist detail page until pending count reaches zero.

## Future work

- Smart playlists (rule-based) — punt.
- Sharing / export — punt.
- Collaborative — N/A single user.
- Playlist folders — punt.
- Cover art / custom art — punt.
