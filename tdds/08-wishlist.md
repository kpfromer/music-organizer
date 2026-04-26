# 08 — Wishlist

> Background worker that turns `wishlist_item` rows into imported tracks via Soulseek. Built on `song-rs` (`/Users/kpfromer/programming/rust/soulseek-rs/song-rs/`), which sits on `soulseek-rs-lib` and already has search + ranked download primitives.

## Worker lifecycle

`mm-wishlist::WishlistWorker` is a single tokio task spawned at server startup. It:

1. Lazily connects the soulseek client (`song-rs::Client`) on first work item — keeps Soulseek connections warm via the underlying actor system's heartbeat.
2. Loops:
   - Query `wishlist_item WHERE status = 'PENDING' OR (status = 'FAILED' AND next_retry_at <= now()) ORDER BY created_at LIMIT 1`.
   - If none: sleep 30s (or until poked via `tokio::sync::Notify`).
   - If one: process it (state machine below). Loop again.
3. On shutdown: drain current item to a clean state (`PENDING` if interrupted mid-work, never leave it stuck in a transient state), disconnect soulseek.

Concurrency: **1 item at a time** in v2. Rationale: Soulseek peer slots are limited; sequential keeps load polite. Future: small parallelism (≤3) configurable.

## State machine

```
PENDING ──► SEARCHING ──► DOWNLOADING ──► CHECKING ──► IMPORTING ──► COMPLETED
   ▲           │              │              │             │
   │           ▼              ▼              ▼             ▼
   └────── FAILED (with next_retry_at backoff)
```

Transitions are written to DB *before* the action, so a crash mid-action leaves a useful breadcrumb (e.g. `DOWNLOADING` with no file → next worker run sees this is not `PENDING`/`FAILED`, treats as crashed, transitions back to `FAILED` with reason `crash_during_download` and a short backoff).

### SEARCHING

Build a `song-rs::SongQuery` from the wishlist item:

```rust
SongQuery {
  title: query_title,
  artist: query_artist,
  album: query_album,
  duration_ms: target_duration_ms,
  preferred_formats: parse(preferred_formats),  // e.g. ["flac", "m4a", "mp3"]
  min_bitrate_kbps,
}
```

`song-rs::Client::search(query)` returns ranked `SongResult`s. Take the top N (default 10).

If empty: → `FAILED`, reason `no_search_results`, backoff 1h.

### DOWNLOADING

Iterate the ranked results, attempt download of each until one succeeds or list is exhausted:

- `song-rs::Client::download(result)` to a temp dir under `MM_MUSIC_ROOT/.wishlist_temp/<item_id>/`.
- Per-download timeout (10 min default; configurable).
- Cancellation via `DownloadHandle::cancel()` if the worker is shutting down.

If all fail: → `FAILED`, reason `all_downloads_failed`, backoff 6h.

### CHECKING

Run `audio_check::check_file_with_options` on the downloaded file. Per Q13 — full decode-to-end + duration check.

- If `is_corrupt = true` (soft errors over threshold OR duration mismatch with `target_duration_ms`): discard this file, return to DOWNLOADING with the next ranked result. If we exhaust the list without a clean file: → `FAILED`, reason `all_corrupt`, backoff 6h.
- If clean: continue.

### IMPORTING

Call `mm-import::ImportService::import_path(temp_path)` — same code path as the watch-folder import. The temp file is moved into its proper location by `mm-storage`.

On success, `import_path` returns the `file_id` and `track_id` (Some). We update:

```sql
UPDATE wishlist_item
SET status = 'COMPLETED',
    resulting_track_id = ?,
    updated_at = now()
WHERE id = ?
```

On failure: → `FAILED`, reason from import, backoff 24h.

### Backoff schedule

Exponential with caps:

| Attempts | Next retry |
|---|---|
| 1 | +1h |
| 2 | +6h |
| 3 | +24h |
| 4 | +3d |
| 5+ | +7d, capped |

Stored as `next_retry_at = now() + backoff(attempts_count)`.

After 10 attempts: stay in `FAILED` but stop auto-retrying (`next_retry_at` set to far future). UI shows "Max retries reached — manual retry only."

## Prioritization

In v2, FIFO by `created_at`. Punted: priority field on wishlist items, urgent-mode etc.

`playlist_pending` items (TDD 07) are *not* prioritized higher in v2 — they share the queue. Future tweak.

## Format / quality preferences

Stored per-item (`preferred_formats`, `min_bitrate_kbps`) so different playlists imported with different prefs play nicely. UI presents global defaults at wishlist-creation time but each item can be overridden in its detail view.

## Manual actions (UI)

- **Retry now** (any state) — sets `status = PENDING`, `next_retry_at = NULL`, pokes the worker via `Notify`.
- **Cancel** (PENDING/FAILED) — deletes the item; cascade removes any `playlist_pending` rows (which leaves a hole in the playlist — UI shows "removed from wishlist").
- **Edit** — change query fields (title/artist/album/format prefs); resets `attempts_count` to 0.

Active item (SEARCHING/DOWNLOADING/etc.) cannot be cancelled mid-flight in v2 — punt cancel-mid-download. UI shows it greyed.

## Soulseek client lifecycle inside the binary

- `song-rs::Client` is created once and stored in app state behind `Arc<Mutex<Option<Client>>>`.
- First wishlist work triggers `connect()`. Stays connected.
- Server shutdown disconnects.
- Connection failures: `song-rs::Client::connect()` already has reconnect / relogin (per soulseek-rs `MEMORY.md` "Reconnect / Relogin"). Wishlist worker just retries on its own backoff.

## Progress UI

Polled GraphQL `wishlistItems(status?, limit, offset)`. The detail page polls every 2s when an item is in a transient state (SEARCHING / DOWNLOADING / CHECKING / IMPORTING) for live progress. We add a `progress_text` field on wishlist_item updated by the worker:

| State | Example text |
|---|---|
| SEARCHING | `searching soulseek (12 candidates)` |
| DOWNLOADING | `downloading from peer abc123 (45%)` |
| CHECKING | `decoding file (3:21 / 4:02)` |
| IMPORTING | `moving file and writing tags` |

Stored as a column on `wishlist_item`; updated frequently (cheap on SQLite WAL).

## Test plan

- Unit: state transitions, backoff math, score-to-format mapping.
- Integration: spin up a fake `song-rs` client (trait extracted in `mm-wishlist` so we can swap), drive an item PENDING → COMPLETED with a fixture file. Verify DB rows and `playlist_pending` resolution.
- Manual smoke test: real Soulseek creds in dev, wishlist a known-easy track, verify end-to-end.

## Future work

- Priority field, urgent mode.
- Parallel workers (≤3).
- Cancel mid-download.
- Track-level alerts ("notify me when X is acquired").
- Auto-clean `.wishlist_temp/` on startup.
