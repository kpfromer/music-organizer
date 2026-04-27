# Phase 9 — Metadata Editing

> Free editing of USER_CREATED entities; refusal of edits on MB-backed entities; demote escape hatch. Small phase — most logic reuses the matching UI's "create new" form.

## Goals

- Mutations: `updateUserCreatedTrack`, `updateUserCreatedAlbum`, `updateUserCreatedArtist`, `demoteTrackToUserCreated`.
- Validation: refuse with `code: VALIDATION` + `reason: 'mb_backed_immutable'` if entity is MB-backed (except `demote`).
- Frontend: detail pages for track / album / artist gain an "Edit" button (USER_CREATED only) opening a form. MB-backed entities show "Re-match" + a "Demote" link in an overflow menu.
- Forms reuse the `components/forms/` primitives + the matching UI's create-new form composition.

## Acceptance criteria

- [ ] On a USER_CREATED track detail: Edit button opens form with all fields editable; submit updates row + junction tables atomically.
- [ ] Edit a USER_CREATED track's artists → `track_artist` rows replaced with new positions.
- [ ] Try to edit an MB-backed track via the GraphQL mutation directly → returns VALIDATION error with `mb_backed_immutable` reason.
- [ ] On an MB-backed track detail: no Edit button; Re-match button links to `/matching/$fileId` for one of its files.
- [ ] Demote an MB-backed track via overflow menu → confirmation modal → accepted → track loses MBIDs, source flips to USER_CREATED, files still linked.
- [ ] After demote: Edit button now appears.
- [ ] Same flow for albums and artists.

## TDD references

- TDD 12 (entire doc)
- TDD 09 §Mutations (edit subset)

## Out of scope

- Delete operations (punted).
- Audit log / edit history (punted).
- Per-field MB override (rejected; demote is the escape hatch).

## PR breakdown

**9.1 — `updateUserCreatedTrack` mutation** (~400)
Including USER_CREATED guard, junction-table replacement, validation.

**9.2 — `updateUserCreatedAlbum` + `updateUserCreatedArtist` mutations** (~400)
Same pattern, smaller field sets.

**9.3 — `demoteTrackToUserCreated` (and album/artist analogues if needed)** (~350)
MBID-stripping, source flip; files stay linked.

**9.4 — Frontend edit forms on detail pages** (~450)
Reuses TDD 13 `components/forms/`; track / album / artist detail pages get conditional Edit button.

**9.5 — Frontend Re-match + Demote buttons on MB-backed entities** (~300)
Overflow menu, demote confirmation modal, navigation to `/matching/$fileId` for re-match.

## Risks / unknowns

- **Junction-table replacement on edit.** Replacing `track_artist` wholesale on every save is O(n_artists) — fine for v2 scale (most tracks have ≤ 5 artists). Watch for SeaORM transaction quirks.
- **Demote with shared album.** If you demote a track whose MB-backed album has other tracks, the album stays MB-backed (only the track is demoted). The demoted track's `album_id` remains pointing at the MB album. UI should make it clear: demoted track lives on an MB album, which is fine — that's the intended mixed state per TDD 02.
- **Form shared with matching UI's create-new.** Refactor: extract the form into `components/forms/track-form.tsx` (and similar for album/artist) so both create and edit use it. Default values + submit handler differ.

## Smoke test

```bash
# 1. From phase 6, find a USER_CREATED track. Edit its title; verify update.
# 2. Edit its artists (add one, change positions); verify junction table.
# 3. Find an MB-backed track. Verify no Edit button. Click Demote → confirm → verify source flipped.
# 4. Now Edit button appears; edit it; verify update.
# 5. Try `updateUserCreatedTrack` directly via GraphQL playground on an MB track → expect VALIDATION error.
```
