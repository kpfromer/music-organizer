# 11 — Migrations & Ops

> How schema evolves, how files are laid out on disk, what to back up, what env vars do what.

## Migrations

**Atlas is a developer-time tool only. The Atlas binary is never present in the production Docker image and is never invoked at runtime.** At runtime, the rust binary applies plain `.sql` migration files itself — same approach as v1.

### Files

- `atlas.hcl` — Atlas project config (dev only).
- `schema/schema.sql` — declarative target schema. The source of truth.
- `migrations/` — generated versioned `.sql` files (`<timestamp>_<name>.sql`). Embedded into the binary via `include_dir!` or `rust-embed`. Do not hand-edit after generation.
- `migrations/atlas.sum` — Atlas's checksum file; committed; used by Atlas linting only.

### Authoring workflow (dev only)

1. Edit `schema/schema.sql` to express the desired shape.
2. `atlas migrate diff <name> --env dev` — Atlas computes a diff against the current migrations directory and writes a new versioned `.sql` file under `migrations/`.
3. Inspect the generated SQL. Hand-edit if needed (e.g. column rename that Atlas saw as drop+add).
4. `atlas migrate lint --env dev` — flags destructive changes.
5. Commit `schema/schema.sql` + the new `.sql` file + the updated `atlas.sum` together.

### CI check

CI runs `atlas migrate diff --dry-run` (or equivalent). If the dev schema would generate a non-empty diff against the committed migrations, CI fails with "uncommitted schema changes — run `atlas migrate diff` and commit the result." This catches PRs that edit `schema.sql` without generating the migration file.

### Runtime application

At server startup, `mm-server` calls a small migration applier crate (`mm-migrate`):

1. Open the SQLite DB.
2. Ensure a `_schema_migrations` tracking table exists (`version TEXT PRIMARY KEY, applied_at INTEGER NOT NULL`).
3. For each `.sql` file in the embedded `migrations/` (sorted by filename), if its version isn't in `_schema_migrations`, execute it inside a transaction and record it.
4. If any apply fails: log the migration version + error, exit 1.

Implementation: hand-roll it (~50 lines) or use a small crate like `rusqlite_migration` if it accepts `&[(version, sql)]` slices. **Do not use `sqlx::migrate!`** — we're on SeaORM, and pulling sqlx in just for migrations is overkill.

### Why Atlas (dev-only) vs SeaORM imperative migrations

SeaORM migrations are imperative Rust files calling SeaORM's schema builder. Atlas at dev-time gives us:
- Declarative — `schema.sql` is the source of truth, easy to read, easy to diff.
- Better lint for destructive changes.
- Plain `.sql` output — production runtime sees only sql, no Atlas dependency.

Net cost: Atlas in dev environment + CI image only. Production image stays small.

### Initial migration

V2 ships one initial migration containing the full schema from TDD 02 + the cache tables from TDD 04. We do **not** attempt to migrate v1 data — the schemas are too different, and the user has explicitly OK'd a clean start. (If we want to backfill, we write a one-off Rust import tool that reads v1's SQLite file and walks files through the v2 import pipeline.)

## On-disk layout

Single root: `MM_MUSIC_ROOT`. Default in docker: `/music`.

```
$MM_MUSIC_ROOT/
  $album_artist/$album (year)/
    01-01 - $title.flac
    01-02 - $title.flac
    ...
  _unmatched/
    <original-filename>.mp3
  _inbox/                       # = MM_WATCH_FOLDER (default)
    <files dropped here>
  .uploading/                   # in-progress drop-zone uploads
  .failed/                      # files where the move-or-DB step failed
  .cover_art/
    album-{id}-front.jpg
    artist-{id}.jpg
  .cover_art_cache/             # server-resized variants
    {id}-w300.jpg
  .wishlist_temp/               # in-flight soulseek downloads
    {item_id}/<peer-original-filename>
```

Hidden directories are **all under one root** so a single backup of `MM_MUSIC_ROOT` captures everything except the SQLite DB. No music files live outside `MM_MUSIC_ROOT` — the import pipeline always moves into it.

## Backup

What to back up (single user — manual):

1. `MM_DB_PATH` — the SQLite file. Use `sqlite3 $DB ".backup '/backups/music-manager-$(date).db'"` or copy with WAL flushed (`PRAGMA wal_checkpoint(TRUNCATE)` first).
2. `MM_MUSIC_ROOT` — entire tree.

The DB references files by `relative_path` under `MM_MUSIC_ROOT`, so backing up both together keeps them consistent. Backing up at different times is fine as long as you back up the DB *first* and the music tree *second* — DB references files we already had on disk.

No automated backup tooling in v2.

## Health endpoints

- `GET /healthz` → 200 if process is up. (Dumb liveness.)
- `GET /readyz` → 200 if DB is reachable AND fpcalc binary is on PATH AND Atlas migration state matches expected. 503 otherwise with JSON body listing failed checks.

Used by docker-compose / k8s if anyone runs this in such an env.

## Logging

`tracing-subscriber` configured from `MM_LOG` env-filter. Default `info`. Output: stdout in JSON when `MM_LOG_FORMAT=json`, otherwise human-readable. Container deployment sets JSON.

Log volumes per import: ~5 lines (one per stage that emits at `info`). At `debug`, ~20 lines.

## Metrics

Punt for v2. Future: Prometheus endpoint at `/metrics` with import counts, wishlist queue depth, transcoder spawn count.

## Env vars (full list)

Reproduced from TDD 01 + additions surfaced by other docs:

| Var | Default | Required | Purpose |
|---|---|---|---|
| `MM_DB_PATH` | `./music-manager.db` | — | SQLite path. |
| `MM_MUSIC_ROOT` | — | yes | Root of managed library. |
| `MM_WATCH_FOLDER` | `$MM_MUSIC_ROOT/_inbox` | — | Watch folder for new files. |
| `MM_LISTEN_ADDR` | `127.0.0.1:8080` | — | HTTP bind. |
| `MM_ACOUSTID_API_KEY` | — | yes (for fingerprint matching) | AcoustID app key. |
| `MM_MUSICBRAINZ_USER_AGENT` | — | yes | UA for MB/CAA requests. |
| `MM_SOULSEEK_USERNAME` | — | yes (for wishlist) | Soulseek user. |
| `MM_SOULSEEK_PASSWORD` | — | yes (for wishlist) | Soulseek password. |
| `MM_TRANSCODE_TARGET` | `opus@128k` | — | Default transcode format. |
| `MM_IMPORT_WORKERS` | `2` | — | Concurrent import workers. |
| `MM_LOG` | `info` | — | tracing env-filter. |
| `MM_LOG_FORMAT` | `text` | — | `text` or `json`. |
| `MM_GRAPHQL_PLAYGROUND` | `0` | — | `1` to enable GraphQL playground. |
| `MM_FRONTEND_DIR` | unset (use embedded) | — | If set, serve frontend from disk. |

`mm-config` validates: required vars present, paths exist (or are creatable), AcoustID key matches MB-acceptable shape, etc. Refuses startup with a clear message if anything's missing.

## Failure recovery runbook (single-user, informal)

| Symptom | What to check |
|---|---|
| Server won't start, "migration failed" | Check logs for which migration; restore DB backup; investigate. |
| Server won't start, "fpcalc not found" | Ensure Chromaprint installed in image. |
| Imports stuck at AUDIO_CHECK | Likely fpcalc OK but symphonia decoder issue; check the file. |
| Wishlist all items FAILED with `all_corrupt` | Soulseek peer pool quality issue; manually retry, lower preferred-format strictness. |
| Disk full | Server still serves reads; new imports fail loudly. Free space, restart. |

## Future work

- Automated backups (rclone-based sidecar).
- Prometheus metrics.
- Healthz includes "watch folder watcher alive" check.
- Migration rollback support (Atlas supports it; v2 punts).
- v1 → v2 data import tool.
