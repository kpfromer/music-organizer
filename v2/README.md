# music-manager v2

Clean rewrite of music-manager (see `tdds/` at the repo root). Lives alongside v1 in `../` until v2 is feature-complete.

## Layout

```
v2/
  Cargo.toml              # workspace
  crates/                 # mm-* service crates (added incrementally)
  frontend/               # pnpm + Vite + React + TS
  schema/                 # declarative SQL schema (Atlas source of truth)
  migrations/             # generated versioned .sql, embedded in mm-server
  atlas.hcl               # dev-only Atlas config
  Dockerfile
  .forgejo/workflows/     # forgejo Actions (lives at repo root, scoped to v2/**)
```

## Phase plan

See [`tdds/implementation/00-overview.md`](../tdds/implementation/00-overview.md). Phase 1 (foundation) lands as PRs `phase-1.0` through `phase-1.12`.

## Conventions

See [`tdds/implementation/conventions.md`](../tdds/implementation/conventions.md). Every PR follows it; CI enforces it.

## Local dev

Filled in over Phase 1. After 1.12:

```bash
cd v2
cargo build
cd frontend && pnpm install && pnpm dev
```
