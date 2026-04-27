# Phase 6 — Matching UI

> The most logic-heavy phase. Candidate algorithm, the keyboard-driven matching detail page, the delayed-commit match queue, USER_CREATED entity creation, MB free-text search, promote/demote. Largest single phase by code volume.

## Goals

### Shared library
- New crate `mm-text-match` extracted from `song-rs/src/ranker.rs`: exports `normalize_str`, `similarity`, `levenshtein`, plus a duration-falloff helper. Both `song-rs` and `mm-matching` depend on it.

### Backend
- `mm-matching` service crate.
- Candidate algorithm (TDD 05 §Scoring) with three sources: AcoustID re-run, local fuzzy DB search, MB free-text search.
- Override rules (MBID-in-tags, ISRC, AcoustID) implemented.
- Match result caching in `match_candidate_cache` (rescan via UI button).
- Mutations: `matchFileToLocalTrack`, `matchFileToMbRecording`, `createUserTrackForFile`, `promoteUserTrackToMb`, `searchMb`.

### Frontend
- `/matching` list view with TanStack Table, filters (score band, format, date), bulk-action.
- `/matching/$fileId` detail view: 3-column layout (file / candidates / manual).
- Match-queue store in `features/matching/match-queue.ts`: delayed commit + multi-undo + auto-flush on navigate.
- Toast stack rendering queued matches with countdown.
- Hotkeys: `j/k/Enter/n/s/r/z/Z/1/2/3` per TDD 05.
- Settings entry: `matchCommitDelayMs`, `matchQueueMaxDepth`.
- USER_CREATED creation form.
- MB search inline panel.
- Promote/demote flows with confirmation modals.

## Acceptance criteria

- [ ] Open `/matching` with N unmatched files in DB → list renders with top candidate per file (or `—`).
- [ ] Press `j` / `k` to navigate; `Enter` to open detail.
- [ ] In detail, candidates show with badges (`AcoustID 0.91`, `Title fuzzy 0.84`, `Duration 1s`).
- [ ] Press `1` to accept top candidate → file greys out, queue shows toast with countdown, focus advances.
- [ ] Press `z` within 3s → undo, file un-greys, toast disappears.
- [ ] Wait > 3s → mutation flushes, file is now linked.
- [ ] Bulk-accept: click bulk-action, set threshold 0.92, run → all matching files committed.
- [ ] Create USER_CREATED form: fill in title/artists (autocomplete works against existing artists)/album → submit → file linked, new entities created with `source = 'USER_CREATED'`.
- [ ] Promote a USER_CREATED track to MB via search → confirmation modal shows diff → accept → track gains MBIDs, source flips to `MUSIC_BRAINZ`.
- [ ] Demote MB-backed track → MBIDs cleared, source = USER_CREATED, files still linked.
- [ ] Match queue auto-flushes on navigation (with confirm modal).
- [ ] Server returns error mid-flush → recovery toast offers retry.
- [ ] AcoustID re-run on detail page (`r` key) refreshes candidates with fingerprint hits.

## TDD references

- TDD 05 (entire doc)
- TDD 09 §Mutations (matching subset)
- TDD 13 §`features/matching/`

## Out of scope

- Edit metadata of MB-backed entities (refused — that's TDD 12; phase 9 has the demote-then-edit path).
- Delete operations (punted).
- Audit log (punted).

## PR breakdown

**6.1 — `mm-text-match` crate** (~350)
Extract `normalize_str`, `similarity`, `levenshtein`, `duration_score` from `song-rs/src/ranker.rs` into a new workspace crate. Update `song-rs` to depend on it; remove inline copies. Tests carry over.

**6.2 — `mm-matching` skeleton + DB-only candidates** (~450)
Crate, `find_candidates(file_id, opts)`, local DB fuzzy search, scoring with new weights (TDD 05).

**6.3 — `mm-matching` AcoustID re-run path** (~400)
Add AcoustID candidate source, override rules (MBID-in-tags, ISRC, AcoustID confidence).

**6.4 — `mm-matching` MB free-text search path** (~350)
`searchMb(query)` calling MB recording-search API, scoring against query.

**6.5 — `match_candidate_cache`** (~250)
Persist scored results; rescan invalidates.

**6.6 — `matchFileToLocalTrack` + `matchFileToMbRecording` mutations** (~450)
Including USER_CREATED duration-update rule for the local case.

**6.7 — `createUserTrackForFile` mutation** (~400)
Creates artist/album/track rows with `source='USER_CREATED'`, links file.

**6.8 — `promoteUserTrackToMb` + `demoteTrackToUserCreated` mutations** (~450)
Diff-based promote, MBID-stripping demote. Cover album-promotion confirmation server-side.

**6.9 — `searchMb` mutation + `candidates` query** (~250)
Wire MB search to GraphQL; expose candidates fetch.

**6.10 — Frontend `/matching` list view** (~450)
TanStack Table, score-band filter, format filter, j/k/Enter list-level hotkeys.

**6.11 — Frontend `/matching/$fileId` 3-column layout** (~400)
File panel + candidates panel + manual panel scaffolding (no actions yet).

**6.12 — Candidate row component + badges** (~350)
Score, reasons-as-badges, accept button.

**6.13 — Match-queue store** (~400)
`features/matching/match-queue.ts`: delayed-commit, multi-undo, auto-flush. Vitest-tested.

**6.14 — Toast stack + hotkeys (z/Z/1/2/3)** (~400)
Visible queued matches with countdown, hotkey wiring, focus management on undo.

**6.15 — USER_CREATED creation form** (~450)
Re-uses `components/forms/`; artist autocomplete; album autocomplete-or-create.

**6.16 — MB search panel + result candidates inline** (~350)
Free-text search box, results render as MbRecording candidates in the same list.

**6.17 — Promote/demote confirmation modals** (~400)
Diff display, accept/cancel, success toast.

**6.18 — Bulk auto-accept action** (~350)
List-view bulk button, threshold dialog, batch mutations.

## Risks / unknowns

- **Score-tuning iteration.** The proposed weights (TDD 05) are a starting point. Expect to tune over the first week of dogfooding. Surface weights as constants in one place so tuning is a one-line change.
- **MB free-text search noise.** The MB search API returns lots of low-relevance results for ambiguous queries. May need to apply our own scoring on the results rather than trusting MB's ranking. Spike before committing to UI behavior.
- **Match queue edge case: file deleted from DB while queued.** Unlikely in v2 (no delete operations) but if reprocessing happens, queue entries could reference a stale file_id. Defensive: queue entries store `fileId` + `candidate snapshot`; flush call validates fileId still exists; if not, drop silently.
- **Tab close with non-empty queue.** Browser `beforeunload` event can prompt; not bulletproof. Acceptable to lose pending matches on hard close — they're un-flushed = un-applied = unchanged state.
- **Promote/demote with shared albums.** A USER_CREATED album might have multiple tracks. Promoting one track to MB shouldn't auto-promote the album unless user explicitly accepts the album mapping too. UI flow: confirmation modal lists "this will also create MB album X — confirm?" with opt-out.

## Smoke test

```bash
# Pre-condition: phases 1-5 done. Library has at least 50 unmatched files (Branch C imports + a few corrupt removes).
# Manually create some by importing untagged files for which AcoustID returns nothing.

# 1. Open /matching, see list.
# 2. Use j/k/Enter to drill into a file.
# 3. Press 1 to accept top candidate; watch toast countdown; press z; verify undo.
# 4. Accept again, wait 3s, verify mutation flushed and file is no longer in unmatched list.
# 5. Use the create-new form on a file with no candidates; create a USER_CREATED track; verify in /library.
# 6. Find a USER_CREATED track in /library; promote it via the matching search panel; confirm modal; accept; verify source flipped.
# 7. Demote an MB-backed track; verify MBIDs cleared.
```
