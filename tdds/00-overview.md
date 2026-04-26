# 00 — Overview

> Status: draft. Single-user, single-machine music manager + player. Successor to v1 at this same repo path; v2 is a clean rewrite, not a migration.

## Goals

1. **Own the music library end-to-end on one box.** Files on disk + a SQLite database describing what's there, with MusicBrainz-grade metadata wherever possible.
2. **Make importing reliable.** Every file that lands in the watch folder or drop zone gets fingerprinted, checked for corruption, tagged with MusicBrainz IDs when we can, and either filed correctly or surfaced for manual matching — never silently dropped.
3. **Play music in a browser** with a Spotify-like UI: sidebar nav, now-playing bar, browse by artist/album/playlist, queue + shuffle.
4. **Wishlist → soulseek** auto-download flow that respects user format/quality preferences and verifies files before importing.
5. **Mass-import playlists** (e.g. exported from Spotify) into the wishlist.
6. **Manual matching UI** for files we couldn't auto-match to MusicBrainz, so the long tail can be cleaned up by hand.
7. **Thin web app, fat libraries.** The `music-manager` crate is GraphQL resolvers + DB access only; everything else (import pipeline, MB client, audio check, soulseek, cover art, ranking) lives in well-tested workspace libs that already exist in `soulseek-rs/`.

## Non-goals (v2)

- Multi-user, accounts, auth — single user behind internal Caddy.
- Live Spotify/Plex/YouTube sync, OAuth, account linking — all dropped from v1.
- Offline / PWA support — strictly online.
- Gapless playback, crossfade, replay-gain normalization.
- Mobile app, native clients.
- Federation, sharing, social features.
- Server-side rendering — frontend is fully client-side, served as static assets by the rust binary.

## Punts (deferred, not rejected)

These are features we want eventually but aren't in v2 scope. Spec sections may reference them as "future work."

- **Refresh-from-MB** for already-imported tracks (re-fetch metadata for stale entities).
- **MP3-player SD-card sync CLI** — long-tail, probably its own binary in the same workspace, talks to the v2 server via GraphQL.
- **Gapless playback / crossfade.**
- **Audio analysis features** — BPM, key detection, replay-gain, waveforms.
- **Smart playlists** (rule-based, auto-updating).
- **Listen history / scrobbling.**
- **Multi-library / multi-root** (v2 has exactly one music root).
- **Auth / multi-user.**
- **Backup/restore tooling** (beyond "back up the SQLite file and the music folder yourself").

## Tech stack (locked)

| Layer | Choice | Notes |
|---|---|---|
| Language (server) | Rust 1.84+ (edition 2024 if practical, else 2021) | |
| Async runtime | Tokio multi-thread | Wishlist worker, file watcher, soulseek client all live here. |
| HTTP server | Axum 0.8 | Already used in v1, well-understood. |
| API | `async-graphql` (GraphQL over HTTP POST) + a few REST endpoints for streaming | See `09-graphql-api.md`. |
| Database | SQLite (single file) | WAL mode, `synchronous=NORMAL`. |
| ORM | SeaORM | Carry over from v1; keep entity definitions in their own crate. |
| Migrations | **Atlas** (declarative HCL/SQL schema) | Replaces v1's hand-written SeaORM migrations. |
| Frontend | React 18 + TypeScript | Strictly client-side. |
| Build (frontend) | **pnpm** + Vite | (v1 used Bun; switching to pnpm per spec.) |
| Routing | TanStack Router | Search params over local state where practical. |
| Data fetching | TanStack Query + a typed GraphQL client (graphql-codegen → typed hooks) | |
| Tables | TanStack Table | |
| Forms | TanStack Form + shadcn form components | |
| Hotkeys | TanStack Hotkeys | (Note: as of writing this is a newer package; if it's not stable enough we fall back to `react-hotkeys-hook`.) |
| Components | shadcn/ui (Tailwind) | |
| Audio playback | Native HTMLAudioElement, MediaSession API for OS controls | |
| Deployment | Single docker image, env-var config | Frontend bundle embedded in binary via `rust-embed` or served from a static dir alongside the binary. |
| Reverse proxy | Caddy (user-managed, outside this project) | |

## Workspace layout (locked at high level — details in `01-architecture.md`)

V2 lives in a fresh repo (or fresh top-level dir of this one), pulling reusable libraries from the existing `soulseek-rs/` workspace as path/git deps:

- From `soulseek-rs/`: `soulseek-rs-lib`, `song-rs`, `musicbrainz`, `audio-check`, `cover-art-archive`.
- New crates in this repo: `mm-entities`, `mm-import`, `mm-library`, `mm-wishlist`, `mm-server` (the binary), and a few smaller ones — see `01-architecture.md`.

## Glossary

- **Track** — a structured, MusicBrainz-aware music entity. Has artists, an album, an MBID (when source = `MUSIC_BRAINZ`).
- **File** — a single file on disk. Has a path, format, duration, corruption flag. Always exists; may or may not be linked to a Track. Identity is `file.id`, written into the file as a `MUSIC_MANAGER_FILE_ID` custom tag.
- **Source enum** (`USER_CREATED` | `MUSIC_BRAINZ`) — applies to Track, Album, Artist. Indicates whether the entity was hand-created vs. created from MB data.
- **Match** — the act of linking a File to a Track. Auto-matched (via fingerprint/MB) or manually matched (via the matching UI).
- **Wishlist item** — a desired track we don't have yet; the wishlist worker will try to acquire it via soulseek.
- **Watch folder** — a directory polled (or notified via inotify/fsevents) for new files; anything that lands there enters the import pipeline.
- **Drop zone** — a UI region in the web app that accepts uploads, which then enter the same import pipeline.

## Document map

- `00-overview.md` — this file.
- `01-architecture.md` — workspace crates, server boundaries, deployment, config.
- `02-data-model.md` — DB schema, source enum, relationships, indexes, migrations philosophy.
- `03-import-pipeline.md` — watch + drop zone → fingerprint → MB → corruption check → file move → DB upsert.
- `04-musicbrainz.md` — MB/AcoustID/Chromaprint client usage, local cache strategy, rate limiting.
- `05-matching-ui.md` — candidate algorithm, manual tagging flows.
- `06-playback.md` — streaming endpoints, transcoding, queue/shuffle/now-playing semantics.
- `07-playlists.md` — playlist data model, mass import format.
- `08-wishlist.md` — soulseek-backed background acquisition, prioritization, retry/backoff.
- `09-graphql-api.md` — schema sketch, REST exceptions, error shape, polling cadences.
- `10-frontend.md` — routes, layout, state model, conventions.
- `11-migrations-ops.md` — Atlas (dev-time only) workflow, runtime migration applier, file layout on disk, backup, env vars.
- `12-metadata-editing.md` — edit rules: MB-backed = read-only, USER_CREATED = freely editable, demote escape hatch.
- `13-frontend-conventions.md` — directory layout, where each component/hook/type lives, query keys, naming, codegen.
