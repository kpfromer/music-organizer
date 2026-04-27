# Implementation Plan — Overview

> Phased plan to build music-manager v2 from the specs in `tdds/`. Each phase is one mergeable checkpoint where the system is in a usable, testable state. Some phases are one PR, others are split into 2–3.
>
> **This is a plan, not a contract.** Reality always rearranges things. Update phase docs as you go; don't pretend the original plan was right.

## Before any phase begins

**Read [`conventions.md`](./conventions.md) first.** It's the contract every PR follows: rust + TypeScript style, error handling, architectural rules, DB conventions, the full CI check matrix, branch protection, and how to propose changes to itself. Phase 1's first PR is *not* PR 1.1 — it's setting up the CI checks defined there.

## Phases

1. **[Foundation](./01-foundation.md)** — workspace, DB schema, migrations, server skeleton, healthz.
2. **[Library browse](./02-library-browse.md)** — read-only GraphQL + frontend layout + browse views (empty data is OK).
3. **[Import pipeline — Branches A & C](./03-import-pipeline-base.md)** — watch folder + tag-read + audio-check + storage + DB upsert. No AcoustID yet.
4. **[Fingerprinting & drop zone](./04-fingerprinting-and-uploads.md)** — Branch B (AcoustID + MB), `/api/upload`, file provenance.
5. **[Playback & cover art](./05-playback-cover-art.md)** — `/api/files/*/stream`, transcoder, cover-art pipeline (CAA + embedded fallback), full now-playing player.
6. **[Matching UI](./06-matching-ui.md)** — candidate algorithm (`mm-text-match` extraction), match-queue (delayed commit + undo), USER_CREATED creation, promote/demote.
7. **[Playlists](./07-playlists.md)** — CRUD + reorder, CSV mass-import, `playlist_pending` resolution.
8. **[Wishlist](./08-wishlist.md)** — soulseek-backed background worker, state machine, retry/backoff.
9. **[Metadata editing](./09-metadata-editing.md)** — USER_CREATED edit mutations, demote escape hatch, edit forms.
10. **[Search & polish](./10-search-and-polish.md)** — global search, settings page, empty/loading/error polish.

## Dependency graph

```
1 ─┬─► 2 ─► 3 ─► 4 ─► 5 ─► 6 ─► 7 ─► 8
   │              │         │
   │              └─► 9 ────┘
   └────────────────────► 10  (depends only on 2 + 9)
```

Phase 9 (metadata editing) only blocks phase 10 (the settings page) and could land before 6/7/8 — it's just CRUD on existing tables. Sequencing as above prioritizes seeing real data flowing end-to-end (1→5) before investing in editing UI.

## Conventions

### Per-phase doc shape

Each phase doc has:
- **Goals** — what's true at the end.
- **Acceptance criteria** — checklist a reviewer can run through.
- **TDD references** — which spec sections this phase implements.
- **Out of scope** — what's deferred to later phases (so reviewers don't ask for it).
- **PR breakdown** — if multiple PRs, the split.
- **Risks / unknowns** — things to validate before / during.

### PR sizing

**Aim for ≤ 500 LoC diff per PR for net-new logic** (excluding generated code, lockfiles, fixtures). The point of the cap is reviewability: a 500-line PR of *new code* needs careful reading; a 500-line PR of new code mixed with bootstrapping is hard to review well.

**Exceptions where a larger PR is fine** (as long as the diff stays *contained* in scope — one concern, not five):

- **Scaffolding / generated code** — workspace skeleton, empty crate folders, graphql-codegen output, schema export.
- **shadcn primitive imports** — `pnpm dlx shadcn add` runs that pull in many components at once.
- **Deletions / system removals** — ripping out an old module en masse is easier to review as one PR than smeared across five.
- **Schema migrations + their entity updates** — a migration that touches 8 tables paired with the entity updates that match is one logical change; splitting hurts reviewability.
- **Bulk fixtures** — adding 50 test files for the import pipeline.

The judgment call: would splitting this PR make it harder or easier to review? If splitting makes reviewers chase context across PRs, keep it together. If splitting lets each piece stand alone and be evaluated, split.

Frontend + backend changes for the same *feature slice* stay in one PR so reviewers see the full slice. Phase scaffolding lands in its own PRs ahead of the feature slices.

The PR breakdowns in each phase below are a *suggested* split — the actual implementation may merge or further-split based on these rules. Don't treat them as a contract.

### Commit / branch / PR style

Per the user's jj workflow (see `MEMORY.md` in soulseek-rs). One bookmark per PR, push via `jj git push -c <bookmark>`. PR title = phase name + sub-PR if applicable (`Phase 3.2: Watch folder integration`). PR body links to the relevant phase doc.

### What "done" means for a phase

1. All acceptance criteria check.
2. `cargo test --workspace` + `pnpm test` green.
3. `cargo clippy -- -D warnings` + `cargo fmt --check` green.
4. `atlas migrate diff --dry-run` produces no diff (schema and migrations are in sync).
5. Manual smoke test path documented in the phase doc has been run.

### Atlas + migrations

Whenever a phase introduces schema changes:
1. Edit `schema/schema.sql` to the new target.
2. `atlas migrate diff <name>` to generate the `.sql` file under `migrations/`.
3. Inspect, hand-edit if needed, commit both files + `atlas.sum`.
4. Restart the dev server — `mm-migrate` applies the new file, records it.

The runtime applier never sees Atlas. Atlas only generates files that ship as embedded resources.

### Test strategy per phase

- **Foundation**: smoke test that the binary starts and migrations apply against a fresh tempdir DB.
- **Service crates**: unit tests next to code. Integration tests in `<crate>/tests/` with sqlite tempfile + wiremock for any HTTP.
- **Frontend**: Vitest for store/queue/match-queue logic. RTL only for the matching detail page (most logic). Other UI manually verified.
- **End-to-end**: not in v2.

## Out of scope for the entire plan

These are listed as punts in TDD 00 and won't appear in any phase:
- Auth, multi-user.
- Spotify/Plex/YouTube live integration.
- Offline / PWA.
- Gapless / crossfade / replay-gain.
- Mobile / native clients.
- Listen history, scrobbling, smart playlists.
- v1 → v2 data migration (clean start).
- Subscriptions / WebSocket transport (v2 polls).
- Delete operations on tracks/files/albums/artists/playlists/wishlist (per gap-3 punt).
- Bulk admin operations.

If a reviewer asks "why isn't X here?", first check this list and TDD 00's punts section.

## Estimated sequencing

No calendar dates — single-developer side-project, no deadlines. Rough relative effort:

| Phase | Relative size |
|---|---|
| 1 — Foundation | L (lots of scaffolding, but mostly wiring) |
| 2 — Library browse | M |
| 3 — Import pipeline base | L |
| 4 — Fingerprinting & uploads | M |
| 5 — Playback & cover art | L |
| 6 — Matching UI | XL (the most logic-heavy phase) |
| 7 — Playlists | M |
| 8 — Wishlist | L |
| 9 — Metadata editing | S |
| 10 — Search & polish | S |
