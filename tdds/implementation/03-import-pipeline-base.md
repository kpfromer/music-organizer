# Phase 3 — Import Pipeline (Branches A & C)

> Watch folder + tag-read + audio-check + storage + DB upsert. **No AcoustID/fingerprinting in this phase** — files with MB tags (Branch A) import normally; files without tags (Branch C) land as USER_CREATED with no track. Branch B (fingerprint lookup) is phase 4.

## Goals

- `mm-storage` crate: path templates, sanitization, atomic move, duplicate-suffix handling.
- `audio-check` integrated via the existing crate from soulseek-rs.
- Tag reading via `lofty`; recognition of MusicBrainz IDs in tags.
- `mm-import` orchestrates stages 1, 2, 4 (corruption check), 5 (move), 6 (DB upsert), 9 (tag write-back of `MUSIC_MANAGER_FILE_ID`).
- Stage 3 in this phase only handles **Branch A** (MB tags present → fetch from MB) and falls through to **Branch C** (no tags → no track) when MB tags absent. No fingerprinting.
- Watch folder task with `notify`-rs + 2s debounce + extension whitelist.
- `async-channel` MPMC import queue with N workers (default 2).
- `import_progress` table populated; recent-imports query exposes it.
- `/imports` route renders a live list of import progress with auto-polling (per TDD 09 polling cadence).
- `unimportable_file` log written on failures; failed files moved to `.failed/`.
- Re-link path in stage 1 works (file with our tag returns to watch folder → row's path updated, no new row).

## Acceptance criteria

- [ ] Drop a FLAC with MusicBrainz tags into `MM_WATCH_FOLDER` → it's moved to its proper path under `MM_MUSIC_ROOT`, `track`/`album`/`artist` rows created with `source = 'MUSIC_BRAINZ'`, `file` row linked.
- [ ] Drop an untagged MP3 → file moves to `_unmatched/<original>`, `file` row created with `track_id = NULL`, no track/album/artist rows.
- [ ] Drop a corrupt file → `is_corrupt = 1`, file still imported (visible as warning in UI).
- [ ] Drop the same file twice (different physical files, same content) → both imported as separate rows (per reading-A confirmation).
- [ ] Drop a file with our `MUSIC_MANAGER_FILE_ID` tag matching an existing row → existing row's `relative_path` updated, no new row.
- [ ] Drop a file with our tag pointing to a non-existent id → treated as fresh import.
- [ ] Failed import (e.g. read-only target dir) → file moved to `.failed/`, `unimportable_file` row written.
- [ ] `/imports` shows the in-progress import advancing through stages; settles to `DONE` or `FAILED`.
- [ ] Spamming 100 files into the watch folder doesn't lose any (MPMC channel + per-path in-flight set works).
- [ ] After Branch A or B import, the file at its new location actually has the `MUSIC_MANAGER_FILE_ID` tag readable by lofty.

## TDD references

- TDD 03 (entire doc, minus Branch B specifics)
- TDD 02 §`file`, §`file_audio_info`, §`unimportable_file`, §`import_progress`
- TDD 04 §Cache tables (only the MB cache used in Branch A)
- TDD 11 §On-disk layout

## Out of scope

- AcoustID / Chromaprint / fingerprinting (phase 4).
- Drop zone HTTP upload (phase 4).
- File provenance column population for `UPLOAD` / `SOULSEEK` (phase 4 / 8 respectively); for now everything is `WATCH_FOLDER`.
- Cover art fetching (phase 5).
- Re-process file action (phase 4).
- Frontend match queue / matching UI (phase 6).

## PR breakdown

**3.1 — `mm-storage` path templater** (~350)
Path template (`{album_artist}/{album} ({year})/{disc:02}-{track:02} - {title}.{ext}`), sanitization (Unicode NFC, illegal chars, length cap). Pure function + unit tests.

**3.2 — `mm-storage` atomic move** (~300)
`std::fs::rename` w/ `EXDEV` fallback to copy+verify+delete, duplicate-suffix handling, tests with tempdirs.

**3.3 — `audio-check` integration wrapper** (~250)
Thin module exposing `audio_check::check_file_with_options` with project defaults.

**3.4 — `lofty` tag-read helper** (~450)
Read titles/artists/album/MBIDs/ISRC/`MUSIC_MANAGER_FILE_ID` across mp3/flac/m4a/ogg formats. Tests with one fixture per format.

**3.5 — `mm-import` skeleton + Stage 1 (preflight)** (~400)
Service struct, error enum, ext check, tag-peek for re-link, three Stage-1 cases.

**3.6 — `mm-musicbrainz` cache-only client (Branch A subset)** (~450)
`fetch_recording_by_mbid` + `fetch_release_by_mbid` with read-through cache. No AcoustID yet.

**3.7 — `mm-import` Stage 2 (tag read) + Stage 3 Branch A** (~400)
Wire tag reader, branch on MBIDs-in-tags, fall through to Branch C when absent.

**3.8 — `mm-import` Stage 4 (corruption check)** (~300)
audio-check integration + duration-mismatch signal for MB-backed tracks.

**3.9 — `mm-import` Stage 5 (move) + Stage 6 (DB upsert)** (~500)
File move via mm-storage, transactional upsert of artist/album/track/file/audio_info rows.

**3.10 — `mm-import` Stage 9 (tag write-back)** (~300)
Write `MUSIC_MANAGER_FILE_ID`; for Branch A, write-back of (already-present) MBIDs is a no-op but verify safe.

**3.11 — `import_progress` writes + GraphQL `recentImports`** (~400)
Stage transitions write progress rows; query exposes them.

**3.12 — Watch folder watcher** (~400)
notify-rs (with poll fallback), 2s debounce, extension whitelist, single-producer task that pushes onto MPMC channel.

**3.13 — MPMC import queue + N workers** (~400)
async-channel queue, N consumer tasks, in-flight `HashSet<PathBuf>`, graceful shutdown.

**3.14 — `unimportable_file` log + `.failed/` move on error** (~300)
Failure paths from any stage move file to `.failed/`, write log row.

**3.15 — Re-link path full implementation** (~350)
Stage 1 case-2 logic to update `relative_path` on existing row, no new row, skip remaining stages.

**3.16 — `/imports` frontend route** (~450)
List view with stage badges + branch field, polling per TDD 09 cadence, empty/loading states.

## Risks / unknowns

- **`notify`-rs reliability inside Docker.** Bind mounts on macOS hosts can swallow events. Document the polling-fallback (default 5s) configuration and test on Linux.
- **Lofty's MB tag normalization.** Different formats store MBIDs differently (Vorbis `MUSICBRAINZ_TRACKID` vs ID3 `TXXX:MusicBrainz Track Id` vs MP4 `----:com.apple.iTunes:MusicBrainz Track Id`). Lofty mostly normalizes; verify with fixtures from each format.
- **Tag-write back on FLAC files.** Lofty has known historical issues with some FLAC tagging edge cases. Test with several real-world files before declaring done.
- **Atomic move across filesystems.** `std::fs::rename` returns `EXDEV`; need fallback to copy + verify + delete. The existing v1 code has this pattern (`/Users/kpfromer/programming/personal/music-manager/src/import_track.rs:410`).

## Smoke test

```bash
mkdir -p /tmp/mm-music/_inbox

# Start server (fresh DB)
cargo run -p mm-server &

# Drop a tagged FLAC
cp test-fixtures/pearl-jam-black-tagged.flac /tmp/mm-music/_inbox/

# Watch /imports in browser; see PREFLIGHT → TAGS → IDENTIFY → AUDIO_CHECK → MOVE → DB → TAG_WRITE → DONE
# Verify file moved to /tmp/mm-music/Pearl Jam/Ten (1991)/01-05 - Black.flac
# Verify lofty reads MUSIC_MANAGER_FILE_ID tag from moved file matching file.id

# Drop the same file again (copy from new location back to inbox)
cp '/tmp/mm-music/Pearl Jam/Ten (1991)/01-05 - Black.flac' /tmp/mm-music/_inbox/black-copy.flac
# Verify: existing file row's relative_path updated; no new row
```
