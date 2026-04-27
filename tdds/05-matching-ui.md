# 05 — Matching UI

> The home for orphan files (Branch C from TDD 03 — `file.track_id IS NULL`). Goal: link each unmatched file to a structured `track` (and through that, an `album` and `artist`s), creating MB-backed entities when possible and USER_CREATED ones when not.

## Where it lives

A single page at `/matching`, which is essentially a queue of unmatched files. Sub-routes:

- `/matching` — list of unmatched files (paginated, filterable).
- `/matching/$fileId` — detail view for one file.

## Data the matcher works with

For each unmatched `file`, we have:
- `original_filename`, `relative_path`
- `format`, `size_bytes`
- `file_audio_info.duration_ms`, `bit_rate_kbps`, `codec`
- Existing tags read from the file (re-read on demand; not cached in DB to avoid drift)

## Candidate algorithm

`mm-matching::find_candidates(file_id) -> Vec<Candidate>` returns ranked candidates. A `Candidate` is one of:

```rust
enum Candidate {
    LocalTrack { track_id: i64, score: f32, reasons: Vec<MatchReason> },
    MbRecording { recording_id: String, score: f32, reasons: Vec<MatchReason> },
}
```

### Sources of candidates

1. **Re-run AcoustID on demand.** The candidate algorithm always tries this first — many files that failed Branch B at import time will succeed on retry (network blips, upstream changes). Returns up to 5 MB recordings as `MbRecording` candidates.
2. **Local DB tag-similarity search.** Parse the file's tags (or filename via `song-rs::parser::parse_soulseek_filename`). Search `track` for fuzzy matches on `(title, artist, duration_ms)` using:
   - Title: trigram or Levenshtein similarity ≥ 0.8.
   - Any artist: exact (case-insensitive) match contributes; fuzzy ≥ 0.85 also contributes.
   - Duration: within 3 seconds boosts score; otherwise neutral.
   Returns top 10 as `LocalTrack` candidates.
3. **MB direct search.** Free-text `recording` search against MB API using the file's title + primary artist. Rate-limited like everything else. Up to 5 results as `MbRecording`. Skipped if the file has no parseable tags/filename.

### Scoring

Adapted from `song-rs/src/ranker.rs::compare_tracks` (`/Users/kpfromer/programming/rust/soulseek-rs/song-rs/src/ranker.rs:73`) — that code is tuned for soulseek search (filename is the dominant signal, peer-reported duration is noisy, format quality matters). Local-DB context flips this:
- We have the file's decoded duration — precise.
- DB tracks have MB-canonical durations — also precise.
- File tags can be misspelled; titles/artists are *less* reliable than duration.
- Format quality is irrelevant — we're matching identity, not picking a download.

So we reweight, while reusing the existing normalization + similarity + duration-falloff machinery.

**Shared crate**: extract `normalize_str` (NFC + lowercase + strip `(…)`/`[…]` + collapse whitespace) and `similarity` (1 − Levenshtein/max_len) from `song-rs/src/ranker.rs` into a new workspace crate `mm-text-match`. Both `mm-matching` (this doc) and `song-rs` depend on it. Single source of truth for normalization rules.

**Local-DB weights** (sum to 1.0):

| Component | Weight | Curve |
|---|---|---|
| Duration | 0.50 | 1.0 within ±2s, linear falloff to 0.0 at ±30s (same shape as ranker.rs, shifted thresholds — files are more precise than soulseek metadata). |
| Title | 0.30 | `similarity(normalize_str(file_title), normalize_str(track_title))`. |
| Artist | 0.20 | Max similarity across the track's artists; 0.5 if the file's artist tag is missing. |

**Override rules** (set score to 1.0 regardless of weighted sum):
- File's tags contain a `MusicBrainz Recording Id` matching the candidate track's `musicbrainz_recording_id`. (This means tags were added post-import — we'd have auto-matched on Branch A otherwise.)
- File's `ISRC` tag exactly matches a candidate track's `isrc` or any row in `track_isrc`.
- AcoustID lookup (rerun on demand) returns this candidate's MB recording with confidence > 0.85.

**Thresholds**:
- Surface threshold: **0.65**. Below this, candidate is dropped before display.
- Auto-accept threshold (for the bulk action "auto-accept top candidate where score ≥ X"): default **0.92**, user-adjustable in the bulk dialog.

Candidates are sorted descending by score and returned. UI shows top 10 with reasons as small badges.

**Worth noting**: when applied to MB-recording candidates from the AcoustID re-run path, score is dominated by the override rule (instant 1.0). The duration/title/artist scoring matters most for *local-DB* candidates and the *MB free-text search* path where there's no fingerprint match.

### Caching

Per-file results cached in `match_candidate_cache(file_id, generated_at, payload_json)` and refreshed on demand from the UI. Default TTL: never auto-refresh; user clicks "rescan" if they want fresh candidates.

## UI flow

### List view (`/matching`)

A TanStack Table of unmatched files:
- Columns: filename, format, duration, top candidate (or "—"), top candidate score, actions.
- Filters: by score band (high ≥0.7, medium 0.4–0.7, low <0.4, none), by format, by date imported (search params).
- Bulk action: "Auto-accept top candidate where score ≥ X" — opens a confirm modal with a count and proceeds to accept all.

### Detail view (`/matching/$fileId`)

Three columns:

1. **The file** — left rail. Shows tags, audio info, an inline player, the original filename.
2. **Candidates** — middle. Ranked list. Each candidate row shows:
   - For `LocalTrack`: track title, artists, album, year, duration; a "Match this" button.
   - For `MbRecording`: title, artist credits, release options dropdown (since one recording can be on many releases), duration; "Match this" button.
   - Reasons displayed as small badges (`AcoustID 0.87`, `Title fuzzy 0.91`, `Duration 2s`).
3. **Manual** — right rail. Three actions:
   - **Search MB** — free-text search box hitting MB recording search; results render as candidates inline.
   - **Search local** — search existing tracks; results render as candidates inline.
   - **Create new (USER_CREATED)** — form with title, artists (with autocomplete from existing artists), album (autocomplete or "create new"), track #, disc #. Submitting creates fresh `track`, `album`, `artist` rows with `source = USER_CREATED` and links the file.

### "Match this" action semantics

- **Local track candidate**: just sets `file.track_id = candidate.track_id`. No entity creation.
- **MB recording candidate**:
  1. If the chosen release isn't in `album` yet, fetch + cache + create `album`, its `album_artist` rows, and any new `artist` rows.
  2. Create the `track` if missing (matching by `musicbrainz_recording_id`), with its `track_artist` rows.
  3. Set `file.track_id`.
  4. Tag-write the MBIDs back to the file (same as import Branch B).
- **Create new**: as described.

All of these run in a single DB transaction.

### Promote USER_CREATED → MUSIC_BRAINZ

If a user later identifies a USER_CREATED track via "Search MB" and accepts an MB candidate, we don't have two rows — we *promote* the existing row:
- Update `track.musicbrainz_recording_id`, `track.source = 'MUSIC_BRAINZ'`.
- Replace `track_artist` rows.
- If the album was USER_CREATED, similarly promote it (or move the track to a new MB-backed album, leaving the USER_CREATED one if it has other tracks — UI choice).

This is a non-trivial flow; v2 implements it but with a confirmation modal showing the diff.

## Hotkeys

(Per "tanstack hotkeys" — tentatively, see TDD 00 fallback note.)

In list view (`/matching`):
- `j` / `k` — next / previous file.
- `Enter` — open focused file's detail.
- `1` / `2` / `3` — quick-accept top candidate / 2nd candidate / 3rd candidate (delayed-commit, see below).

In detail view (`/matching/$fileId`):
- `j` / `k` — next / previous candidate within the file.
- `Enter` — accept focused candidate (delayed-commit).
- `n` — focus "Create new" form.
- `s` — focus search box.
- `r` — rescan candidates.

Global within both views:
- `z` — undo most-recent queued match.
- `Z` (shift+z) — undo *all* queued matches.

## Delayed-commit + multi-undo queue

The matching UI is keyboard-driven and fast — which means it's also *easy to mis-tap* and slam a wrong match through. To make rapid tagging safe, accept actions are **optimistic in the UI but delayed in the network**:

### Behavior

1. User presses `Enter` (or `1`/`2`/`3`) to accept a candidate.
2. UI immediately:
   - Greys out the file in the list (it's "matched as far as you're concerned").
   - Advances focus to the next file.
   - Adds an entry to the **match queue** (client-side state, see `features/matching/match-queue.ts` per TDD 13).
3. A toast stack in the bottom-right shows queued matches with countdown:
   ```
   ✓ Pearl Jam – Black            [undo Z]   2.4s
   ✓ Pearl Jam – Yellow Ledbetter [undo Z]   1.8s
   ```
4. After `commit_delay_ms` (default 3000), the actual GraphQL mutation flushes. Toast disappears.
5. `z` pops the most recent queued entry: undoes the optimistic UI state, the file un-greys and refocuses, no mutation ever fires.
6. `Z` flushes the entire queue back to un-matched state.

### Settings (localStorage, exposed in `/settings`)

- `matchCommitDelayMs`: 3000 (range 0–10000). Set 0 to disable the buffer entirely.
- `matchQueueMaxDepth`: 5. Older queued entries auto-flush when a 6th is added.

### Edge cases

- **Page navigation while queue non-empty**: flush all immediately. Confirmation modal first if `commitDelayMs > 0` so the user knows they're committing.
- **Server returns an error after flush**: surface a recovery toast — "Couldn't match X to Y (server error). [retry] [edit]". The optimistic UI for that file rolls back.
- **Two candidates accepted in rapid succession against the same file**: the second wins. Queue entries are keyed by `fileId`; pushing a new entry for the same file replaces the previous one (which was about to commit but hadn't).

### Carve-out: complex actions skip the queue

These commit immediately with no delay:
- "Create new" USER_CREATED form submission (it's a multi-field form, the user is already engaged; no risk of mis-tap).
- "Promote USER_CREATED to MB" (already has a confirmation modal showing the diff).
- Bulk "auto-accept top candidate" action (user already chose the threshold in the dialog).

Quick-keybind acceptance of an already-displayed candidate is the only flow that uses the delayed-commit queue. That's where the speed/risk tradeoff actually lives.

### Why client-side and not server-side undo

Server-side undo would require an audit log table + cleanup logic + UI to surface old log entries. For a 3-second hesitation window, that's overkill. Once a match flushes, it's committed; further changes go through the regular re-match flow (which is itself just another match, queueable, undoable).

## Future work

- Bulk-edit "all files in this folder are this album" wizard (powerful but fiddly; punted).
- AcousticBrainz / genre tagging — punted with the rest of the genre work.
- Confidence-based auto-match without UI for very-high-confidence matches at import time (right now Branch B requires AcoustID success at the >0.85 default; we could go higher and auto-match in the matching view's terms, but it adds complexity for little gain).
