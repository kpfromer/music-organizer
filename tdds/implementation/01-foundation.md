# Phase 1 — Foundation

> Workspace skeleton, DB schema, migrations, runtime applier, server that starts and serves health endpoints. No music-specific logic yet.

## Goals

By end of phase:
- Cargo workspace exists with all crate folders from TDD 01 (most empty).
- pnpm + Vite frontend boots; renders a minimal layout shell (sidebar + main area placeholder + now-playing bar placeholder).
- Atlas dev workflow set up; `schema/schema.sql` contains the full v2 schema (TDD 02 + cache tables from TDD 04).
- Initial migration generated under `migrations/`.
- `mm-migrate` runtime applier embeds migrations and applies them at startup.
- `mm-server` binary starts, applies migrations against a fresh DB, serves `GET /healthz` (200) and `GET /readyz` (200 once DB + fpcalc + ffmpeg checks pass).
- Multi-stage Dockerfile builds and runs.
- CI pipeline runs cargo test + clippy + fmt + pnpm test + pnpm build + atlas lint.

## Acceptance criteria

- [ ] `cargo build --workspace` clean, no warnings.
- [ ] `cargo test --workspace` passes (mostly empty tests but the harness exists).
- [ ] `cargo clippy --workspace -- -D warnings` clean.
- [ ] `cargo fmt --check` clean.
- [ ] `pnpm install && pnpm build` produces a static bundle.
- [ ] `pnpm test` passes (Vitest configured with one trivial test).
- [ ] `atlas migrate apply --env dev` against an empty SQLite file creates all v2 tables.
- [ ] `atlas migrate diff --env dev --dry-run` produces empty output (schema and migrations in sync).
- [ ] Running the binary against a fresh tempdir applies the embedded migration and creates `_schema_migrations` row.
- [ ] `curl localhost:8080/healthz` → 200.
- [ ] `curl localhost:8080/readyz` → 200 (when fpcalc + ffmpeg present) or 503 with JSON listing failed checks.
- [ ] `docker build .` produces an image; `docker run` with required env vars starts and passes /readyz.

## TDD references

- TDD 00 §Tech stack
- TDD 01 §Workspace layout, §Process model, §Configuration, §Deployment
- TDD 02 (entire schema)
- TDD 04 §Cache tables
- TDD 11 §Migrations, §Health endpoints, §Env vars

## Out of scope

- Any GraphQL schema (next phase).
- Any music import logic.
- Any frontend routes beyond `/` showing the layout shell.
- Embedded frontend bundle in the binary (use `MM_FRONTEND_DIR=...` for now; embed step lands in a later phase or as a dedicated infra PR).
- Auth, sessions, anything user-facing beyond rendering a layout.

## PR breakdown

**1.0 — CI baseline + lint configs** (scaffolding-shaped, size unbounded)
Set up the full CI matrix from [`conventions.md`](./conventions.md) §8 against an empty repo *before* code lands. `rustfmt.toml`, `clippy.toml`, `deny.toml`, root `tsconfig.json` baseline, `frontend/biome.json`, `lefthook.yml`. CI runs the full check set against trivially-passing targets so the matrix is wired before code starts arriving.

**1.1 — Workspace skeleton** (~250)
Cargo workspace `Cargo.toml`, all crate folders with empty `lib.rs`, root `README`, basic `.gitignore`.

**1.2 — `mm-config`** (~300)
Env-var loader + validation + tests. Single `Config::from_env()`.

**1.3 — `mm-entities` core** (~450)
SeaORM entities for `artist`, `album`, `album_artist`, `track`, `track_artist`, `track_isrc`. Relations.

**1.4 — `mm-entities` files + cover art** (~400)
`file`, `file_audio_info`, `cover_art`. Relations.

**1.5 — `mm-entities` playlists + wishlist** (~400)
`playlist`, `playlist_track`, `playlist_pending`, `wishlist_item`, `unimportable_file`, `import_progress`.

**1.6 — `mm-entities` MB cache + match cache** (~300)
`mb_cache_recording/release/release_group/artist`, `acoustid_cache`, `cover_art_cache`, `match_candidate_cache`.

**1.7 — Atlas dev setup + initial migration** (~400, mostly SQL)
`atlas.hcl`, `schema/schema.sql` (full v2 schema), generated initial `migrations/<ts>_initial.sql`, `atlas.sum`.

**1.8 — `mm-migrate` runtime applier** (~350)
Embed migrations via `include_dir!`, `_schema_migrations` tracking table, transactional apply, tests against tempdir DB.

**1.9 — `mm-server` skeleton** (~400)
Axum bootstrap, healthz/readyz handlers, calls `mm-migrate` at startup, `tracing-subscriber` init, graceful shutdown via `CancellationToken`.

**1.10 — Frontend bootstrap** (~450)
pnpm workspace, Vite, React+TS, Tailwind, shadcn init (Button/Card), `lib/utils.ts`, root `App.tsx`, TanStack Router with `__root.tsx` and `index.tsx`.

**1.11 — Frontend layout shell** (~350)
Sidebar component (links only, no data), now-playing-bar placeholder, top-bar placeholder, `MM_FRONTEND_DIR` wiring on backend, end-to-end "fresh DB → server up → frontend renders shell".

**1.12 — Dockerfile + CI** (~400)
Multi-stage Dockerfile (node → rust → distroless, with fpcalc + ffmpeg), forgejo CI workflow (cargo test/clippy/fmt + pnpm test/build + atlas lint), `.env.example`.

## Risks / unknowns

- **Atlas SQLite support quirks.** Atlas works with SQLite but check FK + CHECK constraint generation. Spike: generate the migration, eyeball the SQL.
- **`include_dir!` vs `rust-embed` for embedded migrations.** Both work; pick `include_dir!` for slightly less ceremony. Confirm SQL files are read at compile time, not runtime.
- **Distroless + dynamic binary deps.** distroless/cc has glibc but maybe not all of libstdc++. If `ffmpeg` static-fails, fall back to `debian:bookworm-slim`.
- **CI atlas action.** If forgejo's existing CI patterns from soulseek-rs don't have an atlas runner, install via curl from GitHub releases in a CI step.

## Smoke test

```bash
# 1. From repo root:
atlas migrate apply --env dev --url sqlite:///tmp/mm-test.db
sqlite3 /tmp/mm-test.db ".schema artist"   # expect column list

# 2. Run server against same DB:
MM_DB_PATH=/tmp/mm-test.db \
MM_MUSIC_ROOT=/tmp/mm-music \
MM_WATCH_FOLDER=/tmp/mm-music/_inbox \
MM_ACOUSTID_API_KEY=fake \
MM_MUSICBRAINZ_USER_AGENT="test/0.1" \
MM_SOULSEEK_USERNAME=u MM_SOULSEEK_PASSWORD=p \
cargo run -p mm-server

# 3. From another shell:
curl -i http://localhost:8080/healthz   # expect 200
curl -i http://localhost:8080/readyz    # expect 200 or 503 with JSON

# 4. Frontend dev:
cd frontend && pnpm dev
# Visit http://localhost:5173, see layout shell with empty content area
```
