# 05 — Matching UI

> The home for orphan files (Branch C from TDD 03 — `file.track_id IS NULL`). Goal: link each unmatched file to a structured `track` (and through that, an `album` and `artist`s), creating MB-backed entities when possible and USER_CREATED ones when not.

## Where it lives

A single page at `/matching`, which is essentially a queue of unmatched files. Sub-routes:

- `/matching` — list of unmatched files (paginated, filterable).
- `/matching/$fileId` — detail view for one file.

## Data the matcher works with

For each unmatched `file`, we have:
- `original_filename`, `relative_path`
- `sha256`, `format`, `size_bytes`
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

Each `MatchReason` carries a weight; final `score` is a weighted sum normalized to [0, 1].

| Reason | Weight |
|---|---|
| `AcoustIdHighConfidence` (>0.9) | 1.0 |
| `AcoustIdModerate` (0.7–0.9) | 0.7 |
| `TitleExact` | 0.5 |
| `TitleFuzzy(s)` | 0.5 * s |
| `ArtistExact` | 0.3 |
| `ArtistFuzzy(s)` | 0.3 * s |
| `DurationWithin3s` | 0.2 |
| `IsrcMatch` | 0.9 |
| `MbRecordingIdInTags` | 1.0 (auto-import would've caught it; here means tags were added post-import) |

Candidates are sorted descending by score and returned. UI shows top 10 with reasons as tags.

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
- `j` / `k` — next / previous candidate.
- `Enter` — accept focused candidate.
- `n` — focus "Create new" form.
- `s` — focus search box.
- `r` — rescan candidates.

## Future work

- Bulk-edit "all files in this folder are this album" wizard (powerful but fiddly; punted).
- AcousticBrainz / genre tagging — punted with the rest of the genre work.
- Confidence-based auto-match without UI for very-high-confidence matches at import time (right now Branch B requires AcoustID success at the >0.85 default; we could go higher and auto-match in the matching view's terms, but it adds complexity for little gain).
