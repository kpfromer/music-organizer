# 01 — Architecture

> How the v2 system is split into crates, processes, and runtime tasks. The guiding rule: **the GraphQL/HTTP layer is thin — it only translates between the wire and service calls. All real logic lives in well-tested rust libraries.**

## Workspace layout

V2 is a fresh Cargo workspace. It depends on the existing `soulseek-rs/` workspace via path or git deps for the music-pipeline libraries that already exist and are tested there.

```
music-manager/                     # this repo
  Cargo.toml                       # workspace root
  crates/
    mm-entities/                   # SeaORM entity definitions, nothing else
    mm-config/                     # Env-var loading + ClientSettings struct
    mm-storage/                    # File-on-disk layout, move logic, sanitization
    mm-import/                     # Import pipeline (orchestrates mb + audio-check + storage + DB)
    mm-library/                    # Browse/search services: artists, albums, tracks, files
    mm-playlist/                   # Playlist CRUD + mass import parser
    mm-wishlist/                   # Wishlist worker + retry/backoff (uses song-rs)
    mm-matching/                   # Candidate ranking for manual matching UI
    mm-playback/                   # Stream endpoint helpers, transcoding pipeline
    mm-server/                     # binary: axum + async-graphql + frontend-embed
  frontend/                        # pnpm workspace
    package.json
    src/
    vite.config.ts
  migrations/                      # Atlas .hcl or .sql files
  atlas.hcl                        # Atlas config
  Dockerfile
  tdds/                            # specs (this folder)

# External, depended-on via path/git from soulseek-rs:
soulseek-rs/
  soulseek-rs-lib/
  song-rs/
  musicbrainz/
  audio-check/
  cover-art-archive/
```

### Crate responsibilities (one-liners)

| Crate | Owns | Knows about |
|---|---|---|
| `mm-entities` | SeaORM `Entity`, `Model`, `ActiveModel`, `Relation` for every table. | DB only. No business logic. |
| `mm-config` | `Config` struct loaded from env, validated at startup. | env vars, paths. |
| `mm-storage` | "Where does this file go on disk?" path templates, sanitization, atomic move w/ duplicate suffix. | filesystem, no DB. |
| `mm-import` | Orchestrates: tag read → fingerprint → MB lookup → corruption check → storage move → DB upsert. | `mm-entities`, `mm-storage`, `musicbrainz`, `audio-check`, `lofty`. |
| `mm-library` | Read-only services: list/search artists, albums, tracks, files. Pagination. | `mm-entities`. |
| `mm-playlist` | Playlist CRUD, ordering, mass-import parser (CSV / Spotify export JSON). | `mm-entities`, `mm-wishlist` (to enqueue). |
| `mm-wishlist` | Background acquisition loop. State machine: pending → searching → downloading → checking → importing → completed/failed. Retry w/ backoff. | `mm-entities`, `song-rs`, `mm-import`, `audio-check`. |
| `mm-matching` | Given an unmatched `File`, produce ranked Track candidates from local DB + optional MB search. | `mm-entities`, `musicbrainz`. |
| `mm-playback` | Compute `Content-Range` responses, decide transcode-or-passthrough per request, manage transcoder processes. | `mm-entities` (to resolve track→file), `symphonia`/`ffmpeg`. |
| `mm-server` | The binary. Axum routes, GraphQL schema, watch-folder task spawn, wishlist task spawn, frontend bundle serving. | All `mm-*` crates. |

### Error handling convention

**All crates use `thiserror` for error types.** `anyhow` and `color_eyre` are not used anywhere in the workspace — their dynamic-error / context-stack model is an anti-pattern for libraries and a leak of detail at API boundaries. Each crate defines its own typed error enum (e.g. `mm_import::ImportError`, `mm_wishlist::WishlistError`) and conversions via `#[from]`. The binary (`mm-server`) is the only crate that may collapse errors at the very edge — and it does so by mapping typed errors onto `async_graphql::Error` with `extensions.code` (TDD 09), not by stringifying.

This matches the convention already used in `soulseek-rs/` (see `src/error.rs`).

### Rules the layout enforces

- **`mm-server` never contains business logic.** A resolver should look like: `Ok(import_service.import_file(input).await?.into())`. If a resolver grows past ~10 lines, the logic moves into a service crate.
- **Service crates never depend on `async-graphql` or `axum`.** They take plain Rust structs in, return plain Rust structs (or `Result`) out.
- **`mm-entities` never depends on service crates.** No `impl Model { fn import() }` — services act on entities, not the reverse.
- **Tests live next to the code they test.** Every service crate has unit tests; integration tests for the import pipeline live in `mm-import/tests/`.

## Process model

Single binary, single OS process. Inside it, several long-lived tokio tasks:

```
mm-server (process)
├── axum HTTP server task (graphql + REST + static)
├── watch-folder task        (notify-rs → import queue)
├── import worker task(s)    (drains import queue, runs pipeline)
├── wishlist worker task     (loop: pending items → soulseek → import)
└── soulseek client          (song-rs/soulseek-rs-lib actor system, started on demand)
```

The wishlist worker and import workers communicate via tokio mpsc channels. Progress is mirrored to the DB so the UI sees it via polling without needing a live channel into the worker.

### Why all in-process

- One thing to deploy, one thing to monitor.
- Workers can hand each other typed Rust values instead of round-tripping through the DB.
- Cancellation is unified via `CancellationToken`s on shutdown.

### Shutdown

Server installs a `CancellationToken`; SIGTERM/SIGINT triggers it. Workers drain in-flight work (with a deadline, default 30s) then exit. The HTTP server stops accepting new requests immediately and finishes inflight ones.

## Configuration

All config comes from env vars, loaded at startup into a `Config` struct (validated with `serde` + `figment` or hand-rolled). No config file in v2.

| Env var | Default | Purpose |
|---|---|---|
| `MM_DB_PATH` | `./music-manager.db` | SQLite file path. |
| `MM_MUSIC_ROOT` | (required) | Root directory where managed files live. |
| `MM_WATCH_FOLDER` | (required) | Drop folder watched for new files. May equal `MM_MUSIC_ROOT/_inbox`. |
| `MM_LISTEN_ADDR` | `127.0.0.1:8080` | HTTP bind. |
| `MM_ACOUSTID_API_KEY` | (required for fingerprint matching) | AcoustID app key. |
| `MM_MUSICBRAINZ_USER_AGENT` | `music-manager/0.1 (you@example.com)` | MB requires identifying UA. |
| `MM_SOULSEEK_USERNAME` / `MM_SOULSEEK_PASSWORD` | (required for wishlist) | Soulseek credentials. |
| `MM_TRANSCODE_TARGET` | `opus@128k` | Format used when browser can't play original. |
| `MM_LOG` | `info` | `tracing-subscriber` env-filter. |
| `MM_FRONTEND_DIR` | (unset → use embedded) | If set, serve frontend from disk (dev). |

`mm-config` exposes a single `Config::from_env()` that returns `Result<Config>` and is the only thing in the codebase that reads env vars.

## Deployment

Single Dockerfile, multi-stage:

1. `node:20` stage — `pnpm install && pnpm build` → static bundle.
2. `rust:1.84` stage — copies the bundle into the rust source tree, `cargo build --release`.
3. `gcr.io/distroless/cc` final stage — copies the binary, the bundle (or relies on embed), and the `fpcalc` binary (Chromaprint), plus `ffmpeg` for transcoding. **No Atlas binary** — migrations are applied by the rust binary against embedded `.sql` files (see TDD 11).

User runs:

```
docker run \
  -e MM_MUSIC_ROOT=/music \
  -e MM_WATCH_FOLDER=/music/_inbox \
  -e MM_DB_PATH=/data/music-manager.db \
  -e MM_ACOUSTID_API_KEY=... \
  -e MM_MUSICBRAINZ_USER_AGENT=... \
  -e MM_SOULSEEK_USERNAME=... -e MM_SOULSEEK_PASSWORD=... \
  -v /host/music:/music \
  -v /host/data:/data \
  -p 8080:8080 \
  ghcr.io/kpfromer/music-manager:latest
```

Caddy (outside the container) handles TLS and exposes it on the user's internal LAN.

## Logging / tracing

`tracing` everywhere. `mm-server` initializes `tracing-subscriber` with env-filter. Service crates emit structured spans (`#[tracing::instrument]`). No `println!`. No custom logger.

## Testing strategy

- **Unit tests** live next to code in every service crate.
- **Integration tests** in `mm-import/tests/` use a real SQLite tempfile DB + fixture files (sample mp3/flac), but **mock the MB/AcoustID HTTP client** (the `musicbrainz` crate from soulseek-rs should expose a trait or `wiremock`-friendly seam — verify when implementing).
- **Frontend** uses Vitest for unit tests; no e2e in v2 (punt).
- CI runs `cargo test --workspace` + `pnpm test` + `cargo clippy -- -D warnings` + `cargo fmt --check`.
