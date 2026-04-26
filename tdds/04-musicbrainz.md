# 04 — MusicBrainz, AcoustID, Chromaprint

> Everything music-metadata-lookup-shaped. v2 reuses the existing `musicbrainz` crate from `soulseek-rs/` (`/Users/kpfromer/programming/rust/soulseek-rs/musicbrainz/src/`) as-is and adds a thin local-cache layer on top.

## Building blocks (already exist)

From the `musicbrainz` crate in soulseek-rs:

| Module | Function | Purpose |
|---|---|---|
| `chromaprint.rs:6` | `compute_fingerprint(path)` | Shells out to `fpcalc`, returns `(fingerprint, duration_secs)`. |
| `acoustid.rs:45` | `lookup_fingerprint(fp, duration, api_key)` | Calls AcoustID v2 API; rate-limited 1 req/s via `governor`. |
| `musicbrainz.rs:7` | `fetch_recording`, `fetch_release` | MB API client w/ exponential backoff. |
| `lib.rs:42` | `lookup_track(path, api_key) -> TrackMetadata` | End-to-end pipeline: fingerprint → acoustid → mb. |

V2 imports this crate. **No reimplementation.**

## What v2 adds: local MB cache

Per the discussion: store a local copy, **punt** automatic refresh-from-MB.

### Cache tables

```
mb_cache_recording
  recording_id        TEXT PRIMARY KEY  -- MBID
  payload_json        TEXT NOT NULL     -- raw MB response, stored verbatim
  fetched_at          INTEGER NOT NULL

mb_cache_release
  release_id          TEXT PRIMARY KEY
  payload_json        TEXT NOT NULL
  fetched_at          INTEGER NOT NULL

mb_cache_release_group
  release_group_id    TEXT PRIMARY KEY
  payload_json        TEXT NOT NULL
  fetched_at          INTEGER NOT NULL

mb_cache_artist
  artist_id           TEXT PRIMARY KEY
  payload_json        TEXT NOT NULL
  fetched_at          INTEGER NOT NULL

acoustid_cache
  fingerprint_sha256  TEXT PRIMARY KEY  -- sha256 of fingerprint string (fingerprint itself can be huge)
  duration_secs       INTEGER NOT NULL
  payload_json        TEXT NOT NULL
  fetched_at          INTEGER NOT NULL
```

### Cache semantics

- **Read-through**: `mm-musicbrainz` (a thin wrapper crate, or a module inside `mm-import`) checks cache first; on miss, calls upstream, writes cache, returns.
- **Never expires automatically** in v2. The `fetched_at` is recorded for observability (and for the future "refresh from MB" punt).
- **Invalidation**: only via a manual UI action ("forget MB cache for this release") which deletes the row. Punted from v2 UI; only DB-level for now.
- **Storage cost**: MB JSON is small (hundreds of bytes to a few KB per entity). Even 100k entities is well under 1GB. SQLite handles this fine.

Why JSON blobs instead of fully normalized cache columns: the wrapper crate already deserializes into typed structs (`musicbrainz_rs::Recording`, etc.). We don't need to query MB cache by anything other than MBID, so the JSON-blob shape is the simplest correct cache. If we ever need a query like "all releases by this MB artist," it's a small migration.

## Rate limiting

- AcoustID: the upstream crate uses `governor` to enforce 1 req/s. v2 keeps a single `AcoustIdClient` instance shared across import workers — sharing the governor state is what makes the limit actually hold under concurrency.
- MusicBrainz: same — single shared client. MB's stated limit is also 1 req/s for non-authenticated clients with a User-Agent.
- The shared clients live as `Arc<Client>` in `mm-server`'s state, passed into `mm-import` and `mm-matching`.

## User-Agent

MB requires a UA identifying the app and a contact. `mm-config` requires `MM_MUSICBRAINZ_USER_AGENT` to be set; the format we ship as default is `music-manager/0.1.0 ( https://example.com/contact )` and the user must override it.

## Cover Art Archive

Separate API at `coverartarchive.org`. The `cover-art-archive` crate already wraps it. Used only post-import (Stage 7) to fetch front cover for new MB releases. Cache:

```
cover_art_cache
  release_id          TEXT PRIMARY KEY
  payload_json        TEXT NOT NULL  -- the manifest, not the image bytes
  fetched_at          INTEGER NOT NULL
```

Image bytes go to disk under `MM_MUSIC_ROOT/.cover_art/`, tracked in the `cover_art` table (TDD 02).

**Embedded-art fallback**: when CAA returns nothing or the entity is USER_CREATED, the import pipeline (TDD 03 §Stage 7) extracts art from the file's tags via lofty and writes it as `cover_art.source = 'EMBEDDED'`. CAA always wins over embedded when both are available — CAA art is consistent across releases, embedded art varies per file.

## Flow integration with import pipeline

Detail in TDD 03 — at a glance:

```
file → tags? ─yes→ mb_cache hit? ─no→ MB API → cache write → return
              │                  └─yes→ return cached
              │
              └─no→ chromaprint → acoustid_cache hit? ─no→ AcoustID → cache write → ...
```

## Edge cases

- **AcoustID returns multiple recordings.** Pick the highest-score result above 0.85 confidence. Below that, treat as Branch C (no MB data).
- **AcoustID returns recording without release.** Rare but happens. Treat as Branch C.
- **MB recording exists but all its releases are bootlegs/promos.** Pick the first release; user can re-match later via TDD 05.
- **Network failure mid-pipeline.** Treated as a transient failure — `unimportable_file` row with reason `mb_network_failure`, retry next watcher pass.
- **fpcalc not on PATH.** `mm-server` checks at startup and refuses to start with a clear error message.

## Future work

- Refresh-from-MB action: re-fetch any cached entity older than N days, diff against local entities, surface conflicts in a UI (punted).
- Direct MB search by text (artist/title) for the matching UI — currently only supported via the existing `musicbrainz_rs` crate; we'd add a thin `search_recording(query)` method.
