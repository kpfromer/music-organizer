# 12 — Metadata Editing

> Rules for editing track / album / artist metadata after import. Scope: editing only. Delete / bulk-cleanup / orphan-pruning are punted from v2.

## Hard rule: MusicBrainz-backed entities are read-only

If `track.source = 'MUSIC_BRAINZ'` (or album / artist), the user **cannot directly edit** title, artists, album, track number, disc number, or any MB-derived field via the UI. The corresponding GraphQL mutations refuse with `code: VALIDATION` and `extensions.reason: 'mb_backed_immutable'`.

To "fix" an MB-backed track, the user has two paths, both via the matching UI (TDD 05):

1. **Re-match** the track's file(s) to a different MB recording. This goes through the same MB upsert as Branch B import — chooses a new (or existing) MB-backed track, links the file(s), leaves the old track in place. If the old track is now orphaned (no files), it persists (per the punted cleanup discussion — that's fine).
2. **Demote to USER_CREATED** (escape hatch). Mutation `demoteTrackToUserCreated(trackId)` strips the `musicbrainz_recording_id`, sets `source = 'USER_CREATED'`, and unlinks the album/artists if they were also MB-only joined. From there, regular USER_CREATED editing works.

Demotion is not auto-reversible — to come back to MB, use the matching UI's "promote to MB" flow.

Why this rule: MB is the source of truth. v2 doesn't carry a per-field override layer (would require LEFT JOINs on every read). Edits-on-MB-data drift over time, get forgotten, and confuse future re-syncs (which we may add as the punted "refresh from MB" feature).

The same rule applies to `album.source = 'MUSIC_BRAINZ'` and `artist.source = 'MUSIC_BRAINZ'`.

## USER_CREATED entities are freely editable

Mutations:

- `updateUserCreatedTrack(id, input: UpdateTrackInput!)` — title, track_number, disc_number, duration_ms, isrc, album_id (must be USER_CREATED or null), artist_ids (with positions). Refuses if `track.source != 'USER_CREATED'`.
- `updateUserCreatedAlbum(id, input: UpdateAlbumInput!)` — title, release_year, release_date, barcode, artist_ids. Refuses if MB-backed.
- `updateUserCreatedArtist(id, input: UpdateArtistInput!)` — name, sort_name, disambiguation. Refuses if MB-backed.

All three reuse the same form components from the matching UI's "Create new" flow (TDD 05 §Manual). Submission is a single transaction; junction tables (`track_artist`, `album_artist`) are replaced wholesale on each save.

### Promote USER_CREATED → MUSIC_BRAINZ

Already specified in TDD 05 §"Promote USER_CREATED → MUSIC_BRAINZ". Editing → promoting is a one-way trip (you lose your custom edits, replaced by MB data, with a confirmation modal showing the diff).

## UI surface

- **Track / album / artist detail pages** show an "Edit" button only when `source = 'USER_CREATED'`. For MB-backed entities, button is replaced with "Re-match" (linking to TDD 05) and a smaller "Demote" link in an overflow menu.
- **Forms** are the same shadcn + TanStack Form components used in matching UI.
- **Audit trail**: not in v2. (Future: `entity_edit_log` table.)

## What v2 does *not* support (punted, per user)

- Deleting tracks, files, albums, artists, playlists, wishlist items.
- Bulk operations (mass-delete, mass-cancel, mass-cleanup).
- Empty-album / empty-artist auto-pruning.
- Cleanup of `unimportable_file` rows.
- Per-field override of MB-backed entities (the `track_override` alternative considered and rejected for v2).

These all become "future work." If the DB ever needs cleanup before such tooling exists, it's a manual SQL exercise — fine for single-user.

## Test plan

- Unit: each mutation refuses correctly when entity is MB-backed.
- Integration: edit USER_CREATED track, assert junction tables replaced and `updated_at` advanced.
- Integration: demote MB track, assert `source = 'USER_CREATED'`, `musicbrainz_recording_id IS NULL`, files still linked.
