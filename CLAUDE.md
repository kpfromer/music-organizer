# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Music Organizer is a fullstack music management application with a Rust backend (Axum + GraphQL) and a React/TypeScript frontend (Bun runtime). It integrates with Spotify, YouTube, Soulseek, Plex, MusicBrainz, AcoustID, and Ollama.

## Commands

All commands use [Just](https://just.systems/) as the task runner. Sub-recipes are in `backend.just`, `frontend.just`, `migrations.just`, and `docker.just`.

### Development
- `just watch` - Run backend (cargo watch) and frontend (Bun HMR) concurrently
- `just backend-watch` - Backend only with auto-reload
- `just frontend-dev` - Frontend dev server + GraphQL codegen watcher

### Build
- `just build` - Build both frontend and backend (release)
- `just frontend-build` - Frontend production build via Rspack
- `just backend-build` - Backend debug build
- `just backend-build-release` - Backend release build

### Check / Lint / Format
- `just check` (alias `just c`) - Run all checks: format, lint, tests (parallel)
- `just lint` - Lint both frontend (Biome) and backend (Clippy)
- `just format` - Format both frontend (Biome) and backend (cargo fmt)
- `just fix-frontend` - Auto-fix Biome issues

### Testing
- `just test` - Run backend tests (`cargo test`)
- Single test: `cargo test <test_name>`
- Frontend type check: `just frontend-check-typescript` (runs `tsc --noEmit`)

### Database Migrations (Atlas + SQLite)
- `just dev-migrate-sync` - Apply `schema.sql` directly to dev database
- `just create-migration <name>` - Generate a versioned migration from schema.sql changes
- `just diff-migration` - Check for unapplied schema differences
- Schema is defined declaratively in `schema.sql`; versioned migrations live in `migrations/`

### GraphQL Codegen
- `just frontend-codegen` - One-shot codegen (requires backend running on :3000)
- `just generate-graphql-types` - Starts backend, runs codegen, stops backend

## Architecture

### Backend (Rust)
- **Framework**: Axum with Tower middleware
- **API**: GraphQL via async-graphql (schema in `src/http_server/graphql/`)
- **REST routes**: Audio streaming, image serving, downloads (`src/http_server/http_routes/`)
- **ORM**: Sea-ORM with SQLite (`src/database.rs`, entities in `src/entities/`)
- **Services**: Modular per integration (`src/services/spotify/`, `src/services/youtube/`, `src/services/background/`)
- **CLI**: Clap subcommands (Import, Download, Watch, Serve, Config) in `src/main.rs`
- **Shared state**: `AppState` passed via Axum's state extractor
- **Error handling**: `color-eyre`
- **Tracing**: OpenTelemetry with optional Jaeger

### Frontend (TypeScript/React)
- **Runtime**: Bun (use `bun` instead of node/npm/npx)
- **Bundler**: Rspack (Webpack-compatible, fast)
- **Entry point**: `frontend/src/index.ts` (Bun.serve), `frontend/src/App.tsx` (React router)
- **State**: Zustand stores (`frontend/src/stores/`)
- **Data fetching**: TanStack React Query with GraphQL codegen types (`frontend/src/graphql/`)
- **Forms**: TanStack Form + Zod validation (see Form Patterns below)
- **UI components**: Radix UI + Tailwind CSS 4; Shadcn-style components in `frontend/src/components/ui/`
- **Linting/formatting**: Biome (not ESLint/Prettier)

### Data Flow
Browser -> Bun dev server (:3001) -> Axum backend (:3000) -> SQLite via Sea-ORM + external APIs

## Form Patterns (TanStack Form + Zod)

1. Define a Zod schema first, derive types with `z.infer<typeof schema>`
2. Use `useAppForm` from `@/components/form/form-hooks` with `validators: { onChange: schema }`
3. Use `form.AppField`, `FormFieldContainer`, `FormTextField`, `form.FormSubmitButton`
4. Avoid `useState` for form state, raw `<Input>` elements, or manual validation
5. Full examples and anti-patterns are documented in `.cursorrules`

## CI Checks (GitHub Actions)
- `ci.yml`: cargo fmt --check, clippy (-D warnings), cargo test, cargo audit
- `frontend-lint.yml`: Biome check
- `atlas-check.yml`: Schema/migration diff validation
- `cargo-shear.yml`: Unused dependency detection
