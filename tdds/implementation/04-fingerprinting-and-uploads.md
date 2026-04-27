# Phase 4 — Fingerprinting & Uploads

> Branch B (Chromaprint + AcoustID + MB lookup) and the drop-zone upload endpoint. After this phase, every import path is real.

## Goals

- Chromaprint via `fpcalc` + AcoustID via the existing soulseek-rs `musicbrainz` crate are wired into stage 3 of the import pipeline.
- Branch B fully working: untagged file → fingerprint → AcoustID → MB recording + release → tag write-back of MBIDs + `MUSIC_MANAGER_FILE_ID`.
- MB / AcoustID local cache tables populated and read-through.
- Rate limiting verified to hold under concurrency (single shared `governor`).
- `/api/upload` multipart endpoint accepts files, renames into watch folder via `.uploading/` staging.
- Upload-marker sidecar files signal `provenance = 'UPLOAD'` to the watcher.
- `/upload` route + global drop overlay on the frontend.
- `reprocessFile(fileId)` mutation re-runs stages 3–7 for a previously-imported file.

## Acceptance criteria

- [ ] Drop an untagged MP3 with a known fingerprint → AcoustID returns recording → MB returns release → file imported as MUSIC_BRAINZ track.
- [ ] After import, the file on disk has MBIDs written into its tags (verify via lofty).
- [ ] Same untagged file imported a second time (different copy) → AcoustID cache hit on second call (no new HTTP request).
- [ ] AcoustID returns no result → file imports as Branch C (USER_CREATED, no track).
- [ ] AcoustID returns low-confidence (<0.85) result → Branch C.
- [ ] Drag a file into the browser → uploads via `/api/upload` → ends up in watch folder → imports normally with `file.provenance = 'UPLOAD'`.
- [ ] Drag many files at once → upload queue shows progress per file.
- [ ] Click "Reprocess" on a Branch-C file in `/imports` → it re-runs identify; if AcoustID now returns a hit, the file's `track_id` is set without re-moving the file.
- [ ] Run 10 concurrent imports → AcoustID rate-limit not exceeded (verify by counting requests in wiremock-driven test).

## TDD references

- TDD 03 §Stage 3 Branch B, §Re-import / re-process
- TDD 04 §Cache tables, §Rate limiting, §User-Agent, §Edge cases
- TDD 08 §File provenance tracking (upload path)
- TDD 09 §`reprocessFile` mutation, §`/api/upload` REST exception

## Out of scope

- Cover art fetching (phase 5) — but the import pipeline must call into a no-op stage 7 placeholder that becomes the real thing in phase 5.
- Soulseek-provenance imports (phase 8).
- Embedded art extraction (phase 5).

## PR breakdown

**4.1 — Chromaprint via fpcalc** (~250)
Wrap existing `musicbrainz::chromaprint::compute_fingerprint`; readiness check for `fpcalc` on PATH at startup.

**4.2 — AcoustID client + cache** (~400)
`acoustid_cache` table reads/writes; `acoustid::lookup_fingerprint` wired with shared `governor` rate limiter.

**4.3 — `mm-import` Stage 3 Branch B** (~450)
fpcalc → AcoustID → MB recording → MB release; pick best (>0.85 confidence) result; cache writes.

**4.4 — Tag write-back of MBIDs (Branch B)** (~300)
Extend Stage 9 to write resolved MBIDs (not just file ID) into the moved file's tags.

**4.5 — `/api/upload` multipart endpoint** (~450)
Axum multipart handler, body-limit raise, streaming write to `.uploading/<uuid>`, atomic rename into watch folder, sidecar `.upload-marker`.

**4.6 — Provenance marker → watcher** (~300)
Watcher reads `.upload-marker` to set provenance=UPLOAD; default WATCH_FOLDER otherwise; plumb through Stage 6.

**4.7 — Frontend drop zone overlay** (~450)
Global window-level drag listener, full-screen overlay, per-file upload progress; `features/uploads/upload-queue.ts`.

**4.8 — Frontend `/upload` route** (~250)
Dedicated upload page (alternative entry to drag-and-drop).

**4.9 — `reprocessFile` mutation + UI button** (~400)
`ImportService::reprocess(file_id)`, mutation, button on `/imports` rows for FAILED/Branch-C entries.

## Risks / unknowns

- **fpcalc availability.** Confirmed in Dockerfile (phase 1). For local dev, document `brew install chromaprint` / `apt install libchromaprint-tools`.
- **AcoustID rate limit under MPMC workers.** `governor` already serializes inside the soulseek-rs crate. Confirm by checking that two parallel workers calling `lookup_track` don't both hit AcoustID in the same second.
- **Multipart upload size limit.** Axum's default body limit is 2MB — needs raising via `DefaultBodyLimit::max(...)`. Consider streaming-write-to-disk to avoid buffering entire file in memory. Use `tokio_util::io::StreamReader`.
- **Reprocess race.** If a file is being played while reprocess is running, the old file path remains valid (we don't move on reprocess). But if reprocess promotes Branch C → A and a new track is created, the player needs to re-fetch metadata. UI invalidation per TDD 13 query-key invalidation.

## Smoke test

```bash
# Drop an untagged file
cp test-fixtures/untagged-known-fingerprint.mp3 /tmp/mm-music/_inbox/

# Watch /imports: stages should include IDENTIFY (Branch B); branch field = B_FINGERPRINT
# After DONE: /library/tracks shows the resolved track
# Verify file at new location has MBIDs in tags + our file ID tag

# Drop same file again
cp test-fixtures/untagged-known-fingerprint.mp3 /tmp/mm-music/_inbox/copy.mp3
# Watch tracing logs: should see "acoustid cache hit"

# Drop via UI
# Open frontend, drag a file onto the page → see overlay → upload progress → file.provenance = UPLOAD in DB
```
