# 03 — Import Pipeline

> Every file that enters the system flows through this pipeline. There is exactly one pipeline; both the watch-folder watcher and the drop-zone HTTP handler push files into the same import queue.
>
> Heavy reliance on existing soulseek-rs workspace crates:
> - `audio-check` (`/Users/kpfromer/programming/rust/soulseek-rs/audio-check/src/lib.rs:74`) — corruption check + decode duration.
> - `musicbrainz` (`/Users/kpfromer/programming/rust/soulseek-rs/musicbrainz/src/lib.rs:42`) — `lookup_track()` already orchestrates Chromaprint → AcoustID → MB.
> - `cover-art-archive` — fetches cover art from CAA.

## Inputs

Two producers, one consumer.

### Producer 1: watch folder

`mm-server` spawns a `notify`-rs watcher on `MM_WATCH_FOLDER` at startup. Falls back to a 5s polling watcher inside Docker (where inotify can be flaky on bind mounts). On a `Create` or `Modify` event for a supported extension, the file path is enqueued into the import channel.

Debounce: a file must be stable (size unchanged) for 2 seconds before enqueue, to avoid grabbing partially-written files.

Supported extensions (from v1, expanded slightly): `mp3`, `flac`, `m4a`, `aac`, `ogg`, `opus`, `wav`.

### Producer 2: drop zone

A multipart-upload REST endpoint (`POST /api/upload`) writes the upload to a temp file inside `MM_WATCH_FOLDER/.uploading/<uuid>`, then `rename`s it into `MM_WATCH_FOLDER/<original-filename>` once complete. From there the watcher picks it up — *exactly the same code path*.

The endpoint returns immediately after rename (HTTP 202) with a JSON body `{ "queued": true, "filename": "..." }`. Progress polling is via the standard "list recent imports" GraphQL query.

### Why one pipeline, not two

The drop zone could write straight into the import queue, but routing through the watch folder means:
- Identical observability (one log stream, one progress table).
- A file that fails import is on disk in the watch folder, easy to inspect/retry by re-saving.
- Less code to maintain.

## Pipeline stages

```
file path
   │
   ├─ stage 1: pre-flight     (extension check, peek MUSIC_MANAGER_FILE_ID tag, re-link if known)
   │
   ├─ stage 2: read tags      (lofty)
   │
   ├─ stage 3: identify       (try existing tags → MBID, else fingerprint via musicbrainz crate)
   │
   ├─ stage 4: corruption check  (audio-check::check_file)
   │
   ├─ stage 5: file move      (mm-storage)
   │
   ├─ stage 6: db upsert      (artists, album, track, file, file_audio_info)
   │
   └─ stage 7: cover art      (cover-art-archive, async best-effort)
```

Each stage has a clear failure mode (below). Stages 1, 2, 5, 6 must succeed for the file to be considered imported. Stages 3, 4, 7 can fail and the file still gets imported in degraded form.

### Stage 1 — pre-flight

1. Extension check (whitelist above). Fail → log, leave file in place, do **not** add to `unimportable_file` (might be a typo / unrelated file).
2. Read tags (peek for `MUSIC_MANAGER_FILE_ID` only — full tag-read is Stage 2). Three cases:
   - **No tag** → new file, continue to Stage 2 normally.
   - **Tag present, `id` exists in `file` table** → this is a known file that's been moved/copied back into the watch folder. **Re-link path**: update `file.relative_path` to its new location after Stage 5's move; do not run Stage 2/3/4 (we already vetted this file). Skip to Stage 5 with the existing row, then Stage 6's only action is `UPDATE file SET relative_path = ?, updated_at = ?`. Done.
   - **Tag present, `id` does NOT exist** → tag is stale (different DB / wiped install). Treat as no-tag: continue normally; we'll write a fresh ID at the end.
3. Duplicate-content (same audio bytes as an existing file): **not detected**. Per the user's reading-A confirmation, two files with the same content are imported as separate `file` rows. Future "show me likely duplicates" UI can compare audio fingerprints across the library; not in v2.

### Stage 2 — read tags

Use `lofty` to read existing tags. We extract:
- Title, artist, album, album-artist, track-number, disc-number, year.
- **MusicBrainz tags** if present: `MusicBrainz Track Id`, `MusicBrainz Album Id`, `MusicBrainz Album Artist Id`, etc. (Vorbis comments / ID3 TXXX frames / MP4 atoms — `lofty` normalizes most of this.)
- ISRC.

If MB tags are present *and* look valid (UUID-shaped), we skip Stage 3's fingerprinting and go straight to "fetch by MBID" using the `musicbrainz` crate.

### Stage 3 — identify (MusicBrainz)

Branch A — **MBIDs in tags** (per user's spec step 1):
- Call `musicbrainz` crate's recording fetch for the recording MBID.
- If the recording has releases, prefer the release whose MBID matches the file's `MusicBrainz Album Id` tag if present, else use the first.
- Cache the recording + release in `mb_cache_*` (TDD 04).
- → Track is `source=MUSIC_BRAINZ`.

Branch B — **No MBIDs in tags** (per user's spec step 2):
- Call `musicbrainz::lookup_track(path, acoustid_api_key)` (lib.rs:42–160). This invokes `fpcalc` for Chromaprint, hits AcoustID, picks best result by score, fetches MB recording + release.
- On success: write the resolved MBIDs back into the file's tags (per user's spec — "If found, add tags to song file"). Use `lofty` for tag write. Tag-write happens *after* the move (Stage 5), so we tag the moved file in its final location. Synchronous (per Q2 confirmation). The new `file.id` is also written as `MUSIC_MANAGER_FILE_ID` in the same tag-write pass.
- Cache results.
- → Track is `source=MUSIC_BRAINZ`.

Branch C — **MB lookup failed** (per user's spec step 3):
- File is imported as USER_CREATED, with `track_id = NULL` on the `file` row.
- The track entity is *not* created at import time — there's no MB data to base it on, and we don't want to create a half-baked Track from raw tags. The matching UI (TDD 05) is what creates the Track and links the file.
- → No Track yet. File exists, browseable in "unmatched files" view.

### Stage 4 — corruption check

`audio_check::check_file_with_options(path, CheckOptions { max_soft_errors: 10, .. })` — uses symphonia to decode the entire file.

Two corruption signals:
1. `soft_errors > max_soft_errors` (configurable, default 10). From `audio-check` directly.
2. `|track.duration_ms - file_audio_info.duration_ms| > max(2000ms, 1% of track duration)` — only checked when Stage 3 produced a track (we have an MB-canonical duration to compare against).

Either signal sets `file.is_corrupt = 1` with a `corruption_reason` describing which fired. The file is **still imported** (per user's spec — "if the file imported is corrupted we should still handle it normally but make sure the DB file model has a flag"). UI surfaces it as a warning so the user can re-rip / re-source.

### Stage 5 — file move

`mm-storage` computes the destination path:

- If track is MB-resolved:
  `{album_artist}/{album_title} ({year})/{disc:02}-{track:02} - {title}.{ext}`
- If track is unmatched (Branch C):
  `_unmatched/{original_filename}`
- Sanitization: strip `/ \ : * ? " < > |`, collapse whitespace, truncate each path segment to 200 chars, normalize Unicode (NFC).
- Duplicate handling: if the target path exists, append ` (2)`, ` (3)` until free. (Two different rips of the same track legitimately collide on path under the default template.)

Move strategy:
1. Try `std::fs::rename` (atomic, same-filesystem).
2. On `EXDEV`, fall back to copy + verify size + delete.

After a successful move, if Branch B succeeded, write MB tags back to the file at its new path.

### Stage 6 — DB upsert

In a single transaction:

1. **Artists** — for each track artist and album artist, `INSERT ... ON CONFLICT(musicbrainz_id) DO NOTHING` keyed on MBID; if no MBID, `INSERT ... ON CONFLICT(name COLLATE NOCASE)` is **not** used (we don't auto-merge USER_CREATED artists by name). Instead, a USER_CREATED artist is only ever created from the matching UI, never the import pipeline. *Implication:* in Branch A/B every artist has an MBID; in Branch C no artists are created.
2. **Album** — same pattern; upsert by `musicbrainz_release_id`.
3. **Album-artist links** — insert with positions.
4. **Track** — upsert by `musicbrainz_recording_id`.
5. **Track-artist links** — insert with positions.
6. **track_isrc** — insert any ISRCs found.
7. **File** — insert with `track_id` (Branch A/B) or NULL (Branch C). Capture the new `file.id` for tag-write.
8. **file_audio_info** — insert with values from Stage 4.
9. **Tag write-back** — for Branch B, write resolved MBIDs to the file's tags. For *all* branches, write `MUSIC_MANAGER_FILE_ID = file.id` as a custom tag (Vorbis comment / ID3 TXXX / MP4 atom via lofty). Synchronous (per Q2 confirmation). Failure here is logged but doesn't roll back — the file is still imported and findable.

For USER_CREATED tracks linked to multiple files (relevant only on re-link via the matching UI in TDD 05, not import): when a file is linked, also `UPDATE track SET duration_ms = file_audio_info.duration_ms WHERE id = ? AND source = 'USER_CREATED'` — most-recently-tagged wins (per Q8 confirmation).

If Stage 6 fails (e.g. SQLite busy, FK violation), the move from Stage 5 is rolled back: move the file back to `MM_WATCH_FOLDER/.failed/<original>` and add a row to `unimportable_file`.

### Stage 7 — cover art (best-effort, async)

After Stage 6 commits, spawn a tokio task:
1. **For new MB albums**, fetch front cover from Cover Art Archive via the `cover-art-archive` crate.
2. **If CAA fails OR album is USER_CREATED OR file imported as Branch C**, fall back to **embedded art**: use lofty to extract the first picture from the file's tags. If found, write to disk as `album-{id}-front.{ext}` (or `_unmatched-{file_id}.{ext}` for Branch C) and insert a `cover_art` row with `source = 'EMBEDDED'`.
3. If both fail: no art for this entity. UI shows the deterministic-color initials placeholder (TDD 10).

Failures here only log; they don't affect import success.

**Re-extracting embedded art for re-link (Stage 1 case 2)** is not done — the file is the same one we already saw, art is already extracted.

## Concurrency

`mm-import` exposes an `ImportService` backed by a true **multi-producer / multi-consumer** channel (`async-channel`, since tokio's stock mpsc is single-consumer). Producers: the watch-folder watcher task and the `/api/upload` handler. Consumers: N worker tasks (default 2; configurable via `MM_IMPORT_WORKERS`).

Each work item is processed exactly once: `async-channel`'s `recv()` is competitive across consumers — only one worker gets each item. The watch-folder watcher does its own in-memory dedup (a `HashSet<PathBuf>` of in-flight paths) so the same file path can't be enqueued twice from rapid filesystem events.

Why limit consumers to 2: AcoustID rate-limits at 1 req/sec per client — the `musicbrainz` crate already serializes via `governor`. MB fetches are similarly rate-limited. CPU-bound chromaprint via `fpcalc` benefits from a little parallelism but more than ~3 just blocks on the rate limiter.

This pattern naturally handles the "initial library scan" case (gap 1): drop a folder of 30k files into `MM_WATCH_FOLDER` and the watcher will enqueue them as it discovers them; workers chew through at ~1/sec (AcoustID-limited) without stepping on each other.

## Progress tracking (for UI)

A row in `import_progress` (table not yet in TDD 02 — adding here as it's pipeline-specific):

| Col | Type | Notes |
|---|---|---|
| `id` | INTEGER PK | |
| `original_filename` | TEXT NOT NULL | |
| `stage` | TEXT NOT NULL | `QUEUED`, `PREFLIGHT`, `TAGS`, `IDENTIFY`, `AUDIO_CHECK`, `MOVE`, `DB`, `TAG_WRITE`, `COVER_ART`, `DONE`, `FAILED`. |
| `branch` | TEXT | `A_MB_TAGS`, `B_FINGERPRINT`, `C_UNMATCHED` after stage 3. |
| `error_message` | TEXT | |
| `file_id` | INTEGER → file.id | Set on success. |
| `created_at`, `updated_at` | INTEGER NOT NULL | |

UI polls a "recent imports" GraphQL query (last N rows + any in non-terminal stage) every 1–2s. Older rows pruned by a daily task (keep 7 days).

## Failure modes summary

| Stage | Failure | Action |
|---|---|---|
| 1 (extension) | Unknown ext | Skip silently. |
| 1 (tag peek) | I/O error | Log, leave in watch folder, retry next event. |
| 1 (re-link) | Known file ID, path collision in DB | Update existing row's `relative_path`, no new row. |
| 2 (tags) | Tag read fails | Treat as no-tags, proceed to Branch B. |
| 3 (MB) | All branches fail | Branch C — file imported, no track. |
| 4 (audio) | Corrupt | Mark `is_corrupt=1`, continue. |
| 4 (decode panic) | Catastrophic | Caught, treat file as `is_corrupt=1` with reason "decode panic". |
| 5 (move) | Disk full / perms | Add to `unimportable_file`, leave in watch folder. |
| 6 (DB) | Tx fails | Roll back move, add to `unimportable_file`. |
| 7 (cover) | Any | Log only. |

## Re-import / re-process

UI action "re-process this file": re-runs stages 3–7 on an existing `file` row without re-moving the file. Useful when:
- AcoustID/MB previously failed but might succeed now.
- The user has just edited tags via the matching UI.

This is a separate code path (`ImportService::reprocess(file_id)`) but reuses stages 3, 4, 7 plus a narrower DB upsert that doesn't touch the file table itself.

## Test plan (for the implementation phase)

- Unit: each stage's logic with mocked deps.
- Integration in `mm-import/tests/`:
  - Fixture: a tagged-with-MBIDs mp3 (Branch A). Asserts artist/album/track rows + correct file path.
  - Fixture: an untagged mp3 with a known fingerprint (Branch B), with `wiremock` standing in for AcoustID + MB.
  - Fixture: a noise file (Branch C, audio-check fails). Asserts file imported, no track, `is_corrupt=1`.
  - Fixture: a file with our `MUSIC_MANAGER_FILE_ID` tag pointing to an existing row, dropped into watch folder. Asserts re-link (existing row's `relative_path` updated, no new row).
  - Fixture: a file with our `MUSIC_MANAGER_FILE_ID` tag pointing to a non-existent id. Asserts treated as fresh import.
  - Fixture: a file with MB-tagged duration that grossly mismatches its decoded duration. Asserts `is_corrupt=1` with the duration reason.
