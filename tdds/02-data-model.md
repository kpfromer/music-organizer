# 02 — Data Model

> SQLite schema. Goals: everything has a stable internal id, MusicBrainz IDs are first-class but optional, file identity is decoupled from track identity, no JSON-string arrays, all timestamps are `INTEGER` (unix-seconds, UTC), every FK is enforced.
>
> Lessons explicitly carried forward from v1's mistakes (see `/Users/kpfromer/programming/personal/music-manager/schema.sql`):
> - v1 stored ISRCs and Spotify artists as JSON strings — v2 uses junction tables.
> - v1 mixed `i64` and `DateTime<Utc>` timestamps across tables — v2 uses unix-seconds (`INTEGER`) everywhere.
> - v1's `tracks.file_path` was unique on tracks — v2 separates `file` (1:N to track? see below) from `track`.
> - v1 had no Source enum — v2 adds one to `artist`, `album`, `track`.
> - v1 had no cover-art table — v2 adds one.
> - v1 had no corruption flag — v2 adds `file.is_corrupt`.

## Conventions

- IDs: `INTEGER PRIMARY KEY AUTOINCREMENT` everywhere except where a string ID is the natural key (none in v2 core schema).
- Timestamps: `created_at`, `updated_at` — `INTEGER NOT NULL` (unix seconds). Set by the application, not by triggers, so SeaORM owns it.
- Soft deletes: **none**. Deletes cascade or are forbidden by FK.
- Source enum: stored as `TEXT` with a `CHECK` constraint. Values: `'MUSIC_BRAINZ'`, `'USER_CREATED'`.
- MusicBrainz IDs: stored as `TEXT` (UUID string), `UNIQUE` per entity-type table, indexed.
- Booleans: `INTEGER` 0/1.

## Entity overview

```
                    ┌─────────────┐
                    │   artist    │
                    └──────┬──────┘
                           │ N:M (track_artist, album_artist)
              ┌────────────┼────────────┐
              │            │            │
       ┌──────▼─────┐  ┌───▼─────┐
       │   track    │  │  album  │
       └────────┬───┘  └────┬────┘
                │           │
                │ track.album_id → album.id (nullable)
                │
        ┌───────▼────────┐
        │      file      │  (1 file ↔ 1 track, but file may have NULL track_id)
        └───────┬────────┘
                │
        ┌───────▼─────────┐
        │ file_audio_info │  (1:1 — duration, codec, bitrate, sample rate, soft errors)
        └─────────────────┘

       ┌──────────────┐         ┌──────────────────┐
       │   playlist   │ 1───N   │ playlist_track   │
       └──────────────┘         └──────────────────┘

       ┌────────────────┐
       │ wishlist_item  │
       └────────────────┘

       ┌──────────────┐
       │ cover_art    │  (linked to album_id and/or artist_id)
       └──────────────┘

       ┌──────────────────┐
       │ unimportable_file│  (failed imports — original_filename, reason, failed_path)
       └──────────────────┘
```

## Tables

### `artist`

| Col | Type | Notes |
|---|---|---|
| `id` | INTEGER PK | |
| `name` | TEXT NOT NULL | Canonical display name. |
| `sort_name` | TEXT NOT NULL | MB-style; for USER_CREATED, app derives from `name`. |
| `musicbrainz_id` | TEXT UNIQUE | Nullable for USER_CREATED. |
| `disambiguation` | TEXT | From MB; nullable. |
| `source` | TEXT NOT NULL CHECK (`source IN ('MUSIC_BRAINZ','USER_CREATED')`) | |
| `created_at`, `updated_at` | INTEGER NOT NULL | |

Indexes: `UNIQUE(musicbrainz_id)` (where not null), `INDEX(name COLLATE NOCASE)` for search.

> **Disambiguation note.** Two artists with the same `name` and no MBID are not automatically merged. Manual dedupe lives in matching UI (future). v2 accepts that USER_CREATED artists may collide on name; the app warns when a user creates a name that already exists.

### `album`

| Col | Type | Notes |
|---|---|---|
| `id` | INTEGER PK | |
| `title` | TEXT NOT NULL | |
| `musicbrainz_release_id` | TEXT UNIQUE | The specific release MBID. |
| `musicbrainz_release_group_id` | TEXT | Indexed but not unique (multiple releases share a group). |
| `release_year` | INTEGER | Nullable. |
| `release_date` | TEXT | ISO date string when known; nullable. |
| `barcode` | TEXT | Nullable. |
| `source` | TEXT NOT NULL CHECK | |
| `created_at`, `updated_at` | INTEGER NOT NULL | |

Indexes: `UNIQUE(musicbrainz_release_id)`, `INDEX(musicbrainz_release_group_id)`, `INDEX(title COLLATE NOCASE)`.

### `album_artist`

Composite-PK junction.

| Col | Type | Notes |
|---|---|---|
| `album_id` | INTEGER NOT NULL → album.id ON DELETE CASCADE | |
| `artist_id` | INTEGER NOT NULL → artist.id ON DELETE CASCADE | |
| `position` | INTEGER NOT NULL | 0-indexed; for "Various Artists" or multi-artist albums. |
| `is_primary` | INTEGER NOT NULL | 1 for primary album artist; convention: position=0 ↔ is_primary=1. |

PK: `(album_id, artist_id)`. Index: `INDEX(album_id, position)`.

### `track`

| Col | Type | Notes |
|---|---|---|
| `id` | INTEGER PK | |
| `title` | TEXT NOT NULL | |
| `album_id` | INTEGER → album.id ON DELETE SET NULL | Nullable: a track can exist without an album (singles, unmatched). |
| `track_number` | INTEGER | Nullable. |
| `disc_number` | INTEGER | Nullable; default 1 when album has discs. |
| `duration_ms` | INTEGER | From MB metadata; the *authoritative* track duration. May differ slightly from file duration. |
| `musicbrainz_recording_id` | TEXT UNIQUE | |
| `isrc` | TEXT | Single primary ISRC. Additional ISRCs in `track_isrc`. |
| `source` | TEXT NOT NULL CHECK | |
| `created_at`, `updated_at` | INTEGER NOT NULL | |

Indexes: `UNIQUE(musicbrainz_recording_id)`, `INDEX(album_id, disc_number, track_number)`, `INDEX(title COLLATE NOCASE)`.

### `track_isrc`

| Col | Type | Notes |
|---|---|---|
| `track_id` | INTEGER NOT NULL → track.id ON DELETE CASCADE | |
| `isrc` | TEXT NOT NULL | |

PK: `(track_id, isrc)`. Index: `INDEX(isrc)`.

### `track_artist`

Same shape as `album_artist`.

| Col | Type | Notes |
|---|---|---|
| `track_id` | INTEGER NOT NULL → track.id ON DELETE CASCADE | |
| `artist_id` | INTEGER NOT NULL → artist.id ON DELETE CASCADE | |
| `position` | INTEGER NOT NULL | |
| `is_primary` | INTEGER NOT NULL | |

PK: `(track_id, artist_id)`. Index: `INDEX(track_id, position)`.

### `file`

The on-disk reality. **A `track` may have many `file`s** (different rips, different formats). The matching UI is what links them.

| Col | Type | Notes |
|---|---|---|
| `id` | INTEGER PK | |
| `track_id` | INTEGER → track.id ON DELETE SET NULL | Nullable: file is unmatched until linked. |
| `relative_path` | TEXT NOT NULL UNIQUE | Path under `MM_MUSIC_ROOT`. Always forward-slash, never absolute. |
| `original_filename` | TEXT NOT NULL | Filename as we received it (per Q6 — no full original path). |
| `format` | TEXT NOT NULL | `mp3`, `flac`, `m4a`, `ogg`, `opus`, `wav`. |
| `size_bytes` | INTEGER NOT NULL | |
| `is_corrupt` | INTEGER NOT NULL DEFAULT 0 | Set by audio-check at import. |
| `corruption_reason` | TEXT | Human-readable; e.g. "decode error count 12 > threshold 10". |
| `imported_at` | INTEGER NOT NULL | |
| `provenance` | TEXT NOT NULL CHECK (`provenance IN ('WATCH_FOLDER','UPLOAD','SOULSEEK')`) | How this file got into the system — see TDD 08 §File provenance. |
| `created_at`, `updated_at` | INTEGER NOT NULL | |

Indexes: `UNIQUE(relative_path)`, `INDEX(track_id)`, `INDEX(is_corrupt)`, `INDEX(provenance)`.

> **No content hash.** v1 used `sha256` as the file's identity, which broke whenever we wrote tags to a file (tag write changes file bytes → hash changes). v2 uses `file.id` written into the file as a `MUSIC_MANAGER_FILE_ID` custom tag. The `id` is the canonical identity; the tag survives tag rewrites because tag writes don't change the tag we wrote. See TDD 03 §Stage 1 for re-link-on-reimport behavior.

> **Why nullable `track_id`:** Per import pipeline (TDD 03), a file may be imported with a successful corruption check but a *failed* MB lookup. We still record the file (so it's in the library, playable, surfaced in the matching UI) — it just has no Track yet.

### `file_audio_info`

1:1 with `file`. Split out so the import pipeline can update it cheaply post-decode.

| Col | Type | Notes |
|---|---|---|
| `file_id` | INTEGER PK → file.id ON DELETE CASCADE | |
| `duration_ms` | INTEGER NOT NULL | Decoded actual duration. |
| `sample_rate_hz` | INTEGER NOT NULL | |
| `channels` | INTEGER NOT NULL | |
| `bit_rate_kbps` | INTEGER | Nullable for lossless. |
| `bit_depth` | INTEGER | Nullable for lossy. |
| `codec` | TEXT NOT NULL | `mp3`, `flac`, `aac`, `vorbis`, `opus`, `pcm`. |
| `decode_soft_errors` | INTEGER NOT NULL DEFAULT 0 | From audio-check. |

### `cover_art`

| Col | Type | Notes |
|---|---|---|
| `id` | INTEGER PK | |
| `album_id` | INTEGER → album.id ON DELETE CASCADE | Nullable. |
| `artist_id` | INTEGER → artist.id ON DELETE CASCADE | Nullable. |
| `kind` | TEXT NOT NULL CHECK (`kind IN ('FRONT','BACK','ARTIST','OTHER')`) | |
| `relative_path` | TEXT NOT NULL UNIQUE | Under `MM_MUSIC_ROOT/.cover_art/`. |
| `mime_type` | TEXT NOT NULL | |
| `width`, `height` | INTEGER | Nullable. |
| `source` | TEXT NOT NULL CHECK (`source IN ('CAA','EMBEDDED','USER_UPLOAD')`) | CAA = Cover Art Archive. |
| `created_at` | INTEGER NOT NULL | |

Constraint: exactly one of `album_id`/`artist_id` must be non-null (CHECK).

### `playlist`

| Col | Type | Notes |
|---|---|---|
| `id` | INTEGER PK | |
| `name` | TEXT NOT NULL | |
| `description` | TEXT | |
| `created_at`, `updated_at` | INTEGER NOT NULL | |

### `playlist_track`

| Col | Type | Notes |
|---|---|---|
| `playlist_id` | INTEGER NOT NULL → playlist.id ON DELETE CASCADE | |
| `track_id` | INTEGER NOT NULL → track.id ON DELETE CASCADE | |
| `position` | INTEGER NOT NULL | |
| `added_at` | INTEGER NOT NULL | |

PK: `(playlist_id, track_id)`. UNIQUE: `(playlist_id, position)`. Index: `INDEX(playlist_id, position)`.

> Reordering is "delete + reinsert with new positions in a transaction." Position is dense (no gaps); v2 doesn't try to be clever with sparse positions.

### `wishlist_item`

Inspired by v1 (`src/entities/wishlist_item.rs`) but de-spotified.

| Col | Type | Notes |
|---|---|---|
| `id` | INTEGER PK | |
| `query_title` | TEXT NOT NULL | What we ask soulseek for. |
| `query_artist` | TEXT NOT NULL | |
| `query_album` | TEXT | Optional. |
| `target_duration_ms` | INTEGER | Used to prefer matches with similar length. |
| `target_isrc` | TEXT | If known, helps disambiguate. |
| `target_musicbrainz_recording_id` | TEXT | If known. |
| `preferred_formats` | TEXT NOT NULL | Comma-separated priority list, e.g. `flac,m4a,mp3`. |
| `min_bitrate_kbps` | INTEGER | Optional; for lossy. |
| `status` | TEXT NOT NULL CHECK (`status IN ('PENDING','SEARCHING','DOWNLOADING','CHECKING','IMPORTING','COMPLETED','FAILED')`) | |
| `attempts_count` | INTEGER NOT NULL DEFAULT 0 | |
| `last_attempt_at` | INTEGER | |
| `next_retry_at` | INTEGER | Set on FAILED with backoff; worker queries `WHERE status='PENDING' OR (status='FAILED' AND next_retry_at <= now)`. |
| `error_reason` | TEXT | |
| `resulting_track_id` | INTEGER → track.id ON DELETE SET NULL | Set on COMPLETED. |
| `created_at`, `updated_at` | INTEGER NOT NULL | |

Indexes: `INDEX(status)`, `INDEX(next_retry_at)`, `INDEX(resulting_track_id)`.

### `unimportable_file`

For files we couldn't import at all (e.g. unreadable, format unsupported, move failed). Distinct from a successfully-imported corrupt file (those go in `file` with `is_corrupt=1`).

| Col | Type | Notes |
|---|---|---|
| `id` | INTEGER PK | |
| `original_filename` | TEXT NOT NULL | |
| `original_size_bytes` | INTEGER | |
| `reason` | TEXT NOT NULL | |
| `failed_path` | TEXT NOT NULL | Where we moved the file (under `.failed/`) so the user can inspect. |
| `created_at` | INTEGER NOT NULL | |

This is a pure error log — failed imports are physically moved out of `MM_WATCH_FOLDER` to `.failed/` immediately, so they can't re-trigger watcher events on the same path. No dedup key needed.

### `mb_cache_*` (deferred to TDD 04)

A small set of MB cache tables to avoid hitting the API repeatedly for the same MBID. Schema lives in TDD 04 since it's MB-shaped, not core domain.

## Source enum semantics

| Field | When `MUSIC_BRAINZ` | When `USER_CREATED` |
|---|---|---|
| `artist.musicbrainz_id` | Set | NULL |
| `album.musicbrainz_release_id` | Set | NULL |
| `track.musicbrainz_recording_id` | Set | NULL |

Mixed entities are allowed and expected: a USER_CREATED track can be on a MUSIC_BRAINZ album (e.g. a hidden bonus track), and vice versa. The matching UI is responsible for "promoting" a USER_CREATED entity to MUSIC_BRAINZ when the user identifies a match.

## Why `track` has its own `duration_ms` separate from `file_audio_info.duration_ms`

- `track.duration_ms` is the **canonical** duration:
  - When `source = MUSIC_BRAINZ`: from MB. Never updated by file operations.
  - When `source = USER_CREATED`: from the **most-recently-tagged** file's decoded duration. If the user manually links a second file to a USER_CREATED track later, the track's duration updates to the new file's duration. Reasoning: USER_CREATED tracks have no canonical source; the latest user-confirmed link is the best signal.
- `file_audio_info.duration_ms` is the **observed** decoded duration of this specific file. Always populated.
- Big delta between the two (for MUSIC_BRAINZ tracks only) is one signal for corruption (per TDD 03's check) — fold in alongside symphonia soft errors.

## Migrations philosophy (Atlas)

- Atlas declarative schema in `atlas.hcl`.
- The schema is the source of truth; Atlas computes diffs.
- Versioned migration files generated via `atlas migrate diff`.
- CI runs `atlas migrate lint` to catch destructive changes.
- v2 ships with a single initial migration; we accept that the v1 DB is incompatible (per "v2 is a clean rewrite").

Detail in TDD 11.

## Open questions / future-proofing

- **Genre/tag taxonomy.** Punted from v2; future tables would be `tag` + `track_tag` + `album_tag`.
- **Artist relationships** (member-of, alias-of) — not in v2.
- **Multi-disc release media table** — v2 stores `disc_number` on `track` directly; if we later want disc titles or media-format-per-disc, add a `media` table.
- **Listen history** — punted; future `listen` table FK to track + timestamp.
