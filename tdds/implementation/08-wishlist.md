# Phase 8 — Wishlist

> Soulseek-backed background acquisition. State machine, retry/backoff, song-rs integration, post-completion `playlist_pending` resolution. After this phase, the full mass-import-from-spotify-csv → auto-download flow works end-to-end.

## Goals

- `mm-wishlist` worker crate.
- `WishlistWorker` background task started by `mm-server`.
- `Arc<song_rs::Client>` in app state (no Mutex/Option per Send+Sync confirmation).
- State machine: PENDING → SEARCHING → DOWNLOADING → CHECKING → IMPORTING → COMPLETED / FAILED.
- DB writes before each transition (crash-resilient).
- Backoff schedule (1h / 6h / 24h / 3d / 7d, capped at 10 attempts).
- `progress_text` updates live during transient states.
- Format / quality preferences honored per item.
- Post-IMPORTING: resolve any `playlist_pending` rows for the completed track into `playlist_track`.
- File provenance set to `SOULSEEK` on the resulting `file` row.
- UI: `/wishlist` list with status filter, `/wishlist/$id` detail with live progress.
- Manual actions: retry now, cancel, edit.
- "Wishlist this" mutation on the matching detail page (when no good candidate found, optionally create a wishlist item to acquire a better source).

## Acceptance criteria

- [ ] Create a wishlist item with title/artist/album/duration → it transitions PENDING → SEARCHING in <30s.
- [ ] Worker calls `song_rs::Client::search` with correct query; `searching soulseek (N candidates)` shown in `progress_text`.
- [ ] Top result downloads to `.wishlist_temp/<item_id>/`.
- [ ] `audio-check` runs on downloaded file; corrupt files rejected; next candidate tried.
- [ ] Clean file imports via `ImportService::import_path_with_provenance(path, SOULSEEK)`.
- [ ] On COMPLETED: `wishlist_item.resulting_track_id` set; `playlist_pending` rows for that wishlist_item are deleted and replaced with `playlist_track` rows at the right positions.
- [ ] On FAILED: backoff schedule applied; worker picks up retry-eligible items at next loop.
- [ ] Manual "retry now" pokes the worker (via `tokio::sync::Notify`); worker picks up immediately.
- [ ] Cancel a PENDING item → deleted; cascading removes `playlist_pending` rows.
- [ ] Server shutdown mid-DOWNLOADING leaves the item in `FAILED` with reason `crash_during_download` on next startup.
- [ ] After phase 7's CSV import → unmatched items become wishlist items → over time, items resolve and playlists fill in.

## TDD references

- TDD 08 (entire doc, including provenance + Arc<Client> updates)
- TDD 02 §`wishlist_item`, §`file.provenance`
- TDD 07 §Wishlist linkage (the `playlist_pending` → `playlist_track` resolution)
- TDD 09 §Wishlist mutations / queries

## Out of scope

- Parallel workers (>1 concurrent item — punted).
- Cancel mid-download (punted).
- Priority field (punted).

## PR breakdown

**8.1 — `mm-wishlist` crate + state machine (stubbed soulseek)** (~450)
Crate, `WishlistWorker` task scaffold, state-transition functions, backoff math, all DB writes around state changes. Soulseek calls behind a trait stub.

**8.2 — `Notify`-based wake + main loop** (~350)
Worker loop polls + listens for `Notify`; idle 30s sleep otherwise.

**8.3 — `Arc<song_rs::Client>` in app state** (~250)
Eager construct in `mm-server` startup; pass into worker.

**8.4 — Real soulseek search + download** (~450)
SongQuery builder, search call, ranked-result iteration, temp-dir download, per-attempt timeout.

**8.5 — Audio-check + import on completion** (~400)
Run audio-check on download; on clean → call `ImportService::import_path_with_provenance(path, SOULSEEK)`; on corrupt → next candidate.

**8.6 — `playlist_pending` → `playlist_track` resolution** (~400)
Post-COMPLETED hook scans pending rows, replaces with playlist_track at original positions, transactional.

**8.7 — Manual action mutations** (~400)
`createWishlistItem` / `retryWishlistItem` / `cancelWishlistItem` / `editWishlistItem`.

**8.8 — "Wishlist this" mutation from matching UI** (~250)
Mutation + button on matching detail when no good candidates.

**8.9 — Frontend `/wishlist` list** (~450)
Status-filter dropdown, table with TanStack Table, manual-action buttons inline.

**8.10 — Frontend `/wishlist/$id` detail with live progress** (~400)
Progress text + status, retry/cancel/edit buttons, polling per cadence.

**8.11 — Sidebar active-work badge** (~250)
Count of items in transient states; subtle indicator.

## Risks / unknowns

- **Soulseek connection persistence.** Per soulseek-rs `MEMORY.md`, the Client has reconnect/relogin built in. But long-running idle connections might still drop in unexpected ways. Smoke-test by leaving the worker idle overnight.
- **Soulseek search timeouts.** `song-rs::Client::search` takes a Duration; pick a sensible default (60s) and surface as config later if needed.
- **Download stalls.** `song-rs` returns a `(Download, DownloadHandle)`; `DownloadHandle` has `.cancel()` and a `progress_timeout` on the `Client::download` call. Use both — progress timeout = 60s default, hard recv timeout = 10min.
- **Disk space during downloads.** A FLAC can be 50MB. With 1 worker and 10-attempt retries against `.wishlist_temp/`, worst case ~500MB transient. Document; not in scope to enforce.
- **Audio-check on a partial download.** If symphonia hits unexpected EOF on a partial file, our existing soft-error logic should mark it bad. Verify with a deliberately-truncated fixture.

## Smoke test

```bash
# Pre-condition: real Soulseek creds in env, song-rs has been smoke-tested separately.

# 1. Create a wishlist item for a known-easy song (popular track on soulseek).
# 2. Watch /wishlist/$id: progress_text updates; status moves through SEARCHING → DOWNLOADING → CHECKING → IMPORTING → COMPLETED.
# 3. Open /library/tracks: the new track is there with provenance=SOULSEEK on its file.
# 4. Repeat with an obscure song that won't be on soulseek; verify FAILED with reason=no_search_results, next_retry_at populated.
# 5. Mass-import a 30-track CSV from phase 7; let it run for a few hours; verify items resolve over time.
# 6. Restart server mid-DOWNLOADING (kill -9); on restart, verify the item moves to FAILED with the crash reason.
```
