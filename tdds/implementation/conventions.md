# Conventions & PR Checks

> The contract every PR follows. Conventions = rules I write code to. Checks = what CI runs to verify them. Both live here so there's one source of truth.
>
> When a rule and reality conflict, **fix the rule** (PR to this doc) rather than working around it silently.

---

## 1. Rust conventions

### Style & lints

- **Edition**: 2021 (move to 2024 when stable across all our deps).
- **`rustfmt.toml`**: workspace-wide, default settings + `imports_granularity = "Module"` + `group_imports = "StdExternalCrate"`.
- **Clippy**: workspace-wide `clippy.toml`. Run as `cargo clippy --workspace --all-targets -- -D warnings`. No allow-by-default exceptions without a code comment justifying it.
- **No `unwrap()` / `expect()` outside tests.** Use `?` with typed errors.
- **No `panic!()` / `unimplemented!()` / `todo!()` in merged code.** Pre-merge cleanup catches these.
- **No `println!` / `eprintln!`.** Use `tracing::{info, warn, error, debug, trace}`.
- **No `dbg!` in merged code.**

### Error handling

(Reaffirms TDD 01 §Error handling convention.)

- **Every crate defines its own typed error enum via `thiserror`.** Naming: `<CrateName>Error` (e.g. `mm_import::ImportError`).
- **No `anyhow`, no `color_eyre`, no `Box<dyn Error>` in returned error types.** Internal helpers may use `Box<dyn Error + Send + Sync>` only when erasing across third-party error types where `#[from]` doesn't fit — and only inside the crate.
- **Error variants are specific.** `ImportError::CorruptionCheckFailed { reason: String }` good. `ImportError::Other(String)` bad.
- **Use `#[from]` for "natural" conversions** between this crate's error and an upstream crate's error. Use `#[error("…")]` with a meaningful message; the message ends without punctuation.
- **At the GraphQL boundary** (only in `mm-server`): map typed errors onto `async_graphql::Error` with `extensions.code` per TDD 09. Don't stringify.

### Async / tracing

- **Tokio multi-thread runtime.** Spawn long-lived tasks via `tokio::spawn` with a captured `CancellationToken`.
- **Annotate service methods with `#[tracing::instrument(skip(self))]`** when they're worth tracing (most public methods on services). Skip the noisy ones.
- **Spans capture domain identifiers** (`file_id`, `track_id`) but never secrets (passwords, tokens).
- **Cancellation**: every long-running task accepts a `CancellationToken` from the caller; fast-path for shutdown.

### Module organization

- Prefer `<name>.rs` over `<name>/mod.rs`.
- `pub(crate)` over `pub` for items not intended for cross-crate consumption. `pub` only when crossing crate boundaries.
- Re-exports in `lib.rs` are explicit (`pub use foo::Bar`); no glob re-exports.
- One service struct per crate. If a crate grows two service structs that don't share state, split the crate.

### Public API documentation

- Every `pub` item in a service crate has a `///` doc comment. One sentence is fine.
- `pub` items in `mm-server` (resolvers, axum handlers) don't need doc comments — they're never consumed as a library.

### Concurrency primitives

- `parking_lot::Mutex` over `std::sync::Mutex` for non-async mutexes (faster, no poisoning).
- `tokio::sync::Mutex` only when holding across await points.
- `Arc<RwLock<...>>` only when reads vastly dominate writes; otherwise `Arc<Mutex<...>>`.
- **No `unsafe` without an `// Safety:` comment** explaining the invariant.

### Testing

- **Unit tests**: `#[cfg(test)] mod tests` next to code.
- **Integration tests**: `<crate>/tests/<feature>.rs`.
- **DB integration**: SQLite tempfile via `tempfile` crate; apply migrations via `mm-migrate` from the workspace; never share state between tests.
- **HTTP integration**: `wiremock` for upstream APIs (AcoustID, MusicBrainz, CAA).
- **Fixtures**: `<crate>/tests/fixtures/`. Real audio files preferred over mocks for the import pipeline.
- **Test names**: `test_<what>_<condition>_<expected>`, e.g. `test_import_branch_b_writes_mbids_to_tags`.
- **No `tokio::test(start_paused = true)` magic** unless time matters in the test — it makes failures harder to debug.

---

## 2. TypeScript / frontend conventions

(Reaffirms TDD 13. Adding lints + tooling specifics here.)

- **`tsconfig.json`**: `strict: true`, `noUncheckedIndexedAccess: true`, `noImplicitAny: true`, `exactOptionalPropertyTypes: true`.
- **No `any`.** `unknown` + narrowing.
- **No default exports** except where TanStack Router requires them in `routes/*`.
- **No barrel files** (`index.ts` re-exporting). Import directly.
- **Biome** for linting + formatting (single tool, replaces ESLint + Prettier). Config in `frontend/biome.json`.
  - Formatter: 2-space indent, 100-char line width, single quotes, trailing commas.
  - Recommended rules on. Custom rule overrides:
    - `noConsoleLog: error` (allow `console.warn`/`error` in error boundaries; nowhere else).
    - `noRestrictedImports`: forbid raw `fetch` outside `lib/gql/client.ts` and the audio streaming layer.
    - `useExhaustiveDependencies: error` (replaces `react-hooks/exhaustive-deps`).
  - Run via `pnpm biome check` (lint) and `pnpm biome format --write` (format).
- **Component naming**: `PascalCase` exports, kebab-case file names per TDD 13.
- **Hook naming**: `useCamelCase`, kebab-case files.

### React Query keys

- All keys via `lib/query-keys.ts` registry. **No inline keys** in component code — enforced by lint rule (`no-restricted-syntax` for `useQuery({ queryKey: [...] })` literal arrays at call sites — exception for tests).

### State management

- **Server state**: TanStack Query exclusively.
- **Client state**: Zustand for non-trivial cross-component state (player, match-queue, upload queue). React `useState`/`useReducer` for component-local. **No Redux, no MobX, no Context for state** (Context is fine for DI like the player audio element).
- **URL state** (filters, pagination, search): TanStack Router search params, not React state.

---

## 3. Architecture conventions

(Reaffirms TDD 01 §Rules the layout enforces. Restated as enforceable rules.)

- **Service crates never depend on `async-graphql` or `axum`.** CI guards via dependency-graph check.
- **`mm-entities` never depends on service crates.**
- **`mm-server` resolvers are ≤ 10 lines.** If a resolver grows, the logic moves to a service.
- **One `Config::from_env()` entry point** in `mm-config`. No other code reads `std::env`. Enforced via `cargo deny` ban or a dependency check.
- **Single `tracing-subscriber` init point** in `mm-server::main`.

---

## 4. Database conventions

- All timestamps: `INTEGER NOT NULL` (unix seconds, UTC).
- All FKs explicit. SQLite FK enforcement: `PRAGMA foreign_keys = ON` in `mm-migrate` startup.
- All boolean-like fields: `INTEGER` 0/1.
- Enums: `TEXT NOT NULL CHECK (col IN ('A','B','C'))`. Values match Rust enum names exactly (SCREAMING_SNAKE_CASE).
- Junction tables: composite PK + index on the higher-cardinality side.
- Migrations: dev-time Atlas, runtime apply via `mm-migrate`. Per TDD 11.
- `created_at` / `updated_at` set by application code, not triggers. SeaORM `ActiveModel::insert/update` populates them.

---

## 5. Logging conventions

- **Levels**:
  - `error` — unexpected; something is wrong with the system. Logs include enough context to investigate without reproducing.
  - `warn` — handled but unusual. e.g. AcoustID returned no result for a fingerprint we expected to match.
  - `info` — major lifecycle events (import started/finished, server started, wishlist item state transition).
  - `debug` — per-step detail useful while developing.
  - `trace` — fine-grained, normally off.
- **Default `MM_LOG=info`**.
- **Never log secrets**: passwords, API keys, soulseek credentials. The `Config` struct's `Debug` impl redacts them.
- **Structured fields preferred over interpolation**: `info!(file_id, "import complete")` over `info!("import complete for {file_id}")`.

---

## 6. Documentation requirements per PR

- **TDD updates land in the same PR as the code that implements the change**, not a follow-up. If a PR's behavior diverges from a TDD, either update the TDD or update the code.
- **`migrations/<ts>_<name>.sql`** committed alongside the matching `schema/schema.sql` change and the entity update.
- **No new top-level `.md` files outside `tdds/` and `README.md`** without a discussion. Documentation that lives away from code rots.
- **Code comments**: rare, only for non-obvious *why*. Per the global rule.

---

## 7. Commit & PR conventions

- **jj workflow** per soulseek-rs `MEMORY.md`. One bookmark per PR.
- **PR title**: `Phase X.Y — <terse description>` (e.g. `Phase 3.5 — mm-import skeleton + Stage 1`).
- **PR body**: links to relevant phase doc, copies the acceptance criteria as a checklist with the boxes ticked, lists the smoke test path. Optional: notes on tradeoffs or risks.
- **No force-pushes to `main`.** Local jj rewriting before push is fine and expected.
- **Squash-merge** PRs into `main` so each PR is one main-branch commit.
- **Don't skip hooks** (`--no-verify`). If a hook fails, fix the issue.

---

## 8. CI checks (the per-PR matrix)

Every PR runs every check. PRs cannot merge until all green.

### Rust (workspace-wide)

| Check | Command | Notes |
|---|---|---|
| Build | `cargo build --workspace --all-targets` | Includes test binaries. |
| Tests | `cargo test --workspace --all-targets` | All unit + integration tests. |
| Clippy | `cargo clippy --workspace --all-targets -- -D warnings` | Warnings = errors. |
| Format | `cargo fmt --check --all` | |
| Doc-build | `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps` | Catches broken intra-doc links. |
| Audit | `cargo audit` | RUSTSEC advisories. Soft-fail in dev, hard-fail in CI. |
| Deny | `cargo deny check` | License + duplicate-version policy. Config in `deny.toml`. |
| Unused deps | `cargo machete` (or `cargo shear`, already used in soulseek-rs) | Surface in PR. |

### Frontend

| Check | Command | Notes |
|---|---|---|
| Install | `pnpm install --frozen-lockfile` | Lockfile drift fails. |
| Typecheck | `pnpm typecheck` (= `tsc --noEmit`) | |
| Lint + format | `pnpm biome ci frontend` | Biome's `ci` mode runs both checks; fails on any issue. |
| Tests | `pnpm test` (vitest) | |
| Build | `pnpm build` (vite) | Production bundle must build. |
| Codegen drift | `pnpm graphql-codegen --check` and `git diff --exit-code` | Generated artifacts and `schema.graphql` must be committed. |

### Schema

| Check | Command | Notes |
|---|---|---|
| Atlas in sync | `atlas migrate diff --dry-run --env dev` | Empty diff required. |
| Atlas lint | `atlas migrate lint --env dev` | Destructive changes flagged. |
| `atlas.sum` valid | `atlas migrate hash --env dev && git diff --exit-code` | Sum file matches migrations. |

### Integration check

| Check | Command | Notes |
|---|---|---|
| Schema export | `cargo run -p mm-server --bin export-schema > /tmp/schema.graphql && diff /tmp/schema.graphql frontend/schema.graphql` | Rust schema matches committed frontend schema. |
| Service-crate boundary | `scripts/check-deps.sh` (custom) | Asserts that no service crate's `Cargo.toml` lists `async-graphql` or `axum`. |
| Env-var single source | `scripts/check-env-source.sh` | Asserts that only `mm-config` calls `std::env::var`. |
| Markdown links | `lychee tdds/` (or similar) | Catches broken links between TDDs. |

### Acceptance criteria check

A PR's "Acceptance criteria" checklist (in body) must be fully checked. Reviewer (or self-review for solo work) verifies items match the PR's actual scope. **Not CI-enforceable** — habit-enforceable.

---

## 9. Pre-commit hooks (recommended, not required)

`lefthook` config in repo root (`lefthook.yml`):

```yaml
pre-commit:
  parallel: true
  commands:
    fmt-rust:
      glob: "*.rs"
      run: cargo fmt -- {staged_files}
      stage_fixed: true
    fmt-ts:
      glob: "*.{ts,tsx,js,jsx,json}"
      run: cd frontend && pnpm biome format --write {staged_files}
      stage_fixed: true
    lint-ts:
      glob: "*.{ts,tsx}"
      run: cd frontend && pnpm biome lint {staged_files}
    secrets:
      run: scripts/check-secrets.sh {staged_files}
```

`scripts/check-secrets.sh`: greps for obvious credential patterns (AKIA…, BEGIN PRIVATE KEY, MM_SOULSEEK_PASSWORD=, etc.) and aborts if found.

---

## 10. Branch protection (forgejo / GitHub)

`main` branch:
- All CI checks must pass before merge.
- Linear history (squash-merge only).
- Force-push disabled.
- Direct commits disabled (PR required, even for solo work — keeps the audit trail clean).
- Stale check runs blocked from merge (after 24h, must re-run).

---

## 11. Performance baselines

Not enforced as CI checks in v2 (overkill for single-user). Documented as targets to watch for regressions:

| Operation | Target |
|---|---|
| `tracks` query (1 page of 100 from a 30k-track DB) | < 50ms p95 |
| `unmatchedFiles` (200 entries) | < 100ms p95 |
| Import pipeline single file (Branch A, no network) | < 2s |
| Stream-endpoint TTFB | < 50ms |
| Cover-art `?w=300` cache miss | < 200ms |
| Cover-art `?w=300` cache hit | < 10ms |

If a PR makes any of these meaningfully worse, that's a flag to investigate. No automated enforcement.

---

## 12. Adding new conventions

Process: open a PR that edits this doc, with a one-paragraph rationale at the top of the diff. Self-review for solo work; for any non-trivial rule change, sleep on it before merging — conventions are easy to add, hard to remove.

If a convention is regularly being violated in practice, the convention is wrong, not the code. Update.
