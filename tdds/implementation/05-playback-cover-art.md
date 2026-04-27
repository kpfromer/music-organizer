# Phase 5 — Playback & Cover Art

> Streaming endpoints, transcoder, full now-playing player, cover-art pipeline (CAA + embedded fallback). After this phase, the app is *usable* as a music player.

## Goals

### Backend
- `GET /api/files/:id/stream` — passthrough with HTTP range support via `tower-http`.
- `GET /api/files/:id/transcoded?fmt=opus&bitrate=128` — spawns ffmpeg, streams stdout.
- `GET /api/cover_art/:id?w=N` — serves cover-art images with on-the-fly resize cache.
- Stage 7 of the import pipeline: real implementation with CAA + embedded fallback.
- `mm-playback` crate handles transcoding lifecycle (child process, cancellation on client disconnect).
- `cover_art_cache` populated on first import per release.

### Frontend
- `PlayerProvider` owns a single `<audio>` element + Zustand store.
- Now-playing bar fully functional: play/pause, prev/next, scrubber, volume, queue button.
- Queue drawer.
- File-selection rule (preferred-formats) implemented.
- Auto-fallback to transcoded URL on `<audio>` error.
- MediaSession metadata + handlers.
- `<CoverArt>` component renders real images with the size-variant cache.
- Hotkeys (space, arrows, m, s, r) wired globally.
- Click-to-play from track rows / album detail / artist detail.

## Acceptance criteria

- [ ] Click play on a track in `/library/tracks` → audio plays via passthrough.
- [ ] Skip to next / prev work; queue advances.
- [ ] Shuffle on → Fisher-Yates reorder; current track stays at index 0; off → original order restored.
- [ ] Repeat one → same track replays; repeat all → wraps; repeat off → stops at end.
- [ ] Range-request: scrubbing the timeline issues `Range:` requests; backend serves them.
- [ ] On a Safari-only-incompatible file (FLAC) the `<audio>` errors → frontend falls back to `/api/files/:id/transcoded?fmt=opus`; playback resumes.
- [ ] OS media controls (macOS Now Playing, headphones play/pause) work via MediaSession.
- [ ] Cover art appears for MB-backed albums (CAA fetch).
- [ ] For USER_CREATED entities or files with no CAA hit, embedded art is extracted and shown.
- [ ] Resized variants (`?w=300`) cached on disk; second request 304s or serves from cache instantly.
- [ ] All keyboard hotkeys work as specified in TDD 10.

## TDD references

- TDD 06 (entire doc)
- TDD 03 §Stage 7
- TDD 04 §Cover Art Archive
- TDD 09 §REST exceptions
- TDD 10 §Player, §Hotkeys
- TDD 13 §`features/playback/`

## Out of scope

- Gapless / crossfade / replay-gain (punted permanently).
- HLS / proper transcoded-seek (punted).
- Persistent queue across reloads (punted).

## PR breakdown

**5.1 — `mm-playback` skeleton + `/api/files/:id/stream` passthrough** (~400)
Crate skeleton, range-supporting passthrough handler via `tower-http` style ServeFile semantics.

**5.2 — `/api/files/:id/transcoded` ffmpeg streaming** (~450)
Child-process spawn, stdout streamed to response body, kill-on-disconnect.

**5.3 — `/api/cover_art/:id` + resize cache** (~450)
Image read, optional `?w=N` resize via `image` crate, disk-cache `_cover_art_cache_/`.

**5.4 — Stage 7 CAA fetch** (~400)
Async post-import task, cover-art-archive crate, write to disk, insert `cover_art` row, update `cover_art_cache`.

**5.5 — Stage 7 embedded-art fallback** (~350)
lofty picture extraction when CAA returns nothing or entity is USER_CREATED; `source='EMBEDDED'`.

**5.6 — Backfill cover-art CLI subcommand** (~250)
`mm-server backfill-cover-art` walks albums missing art, runs Stage 7 path.

**5.7 — Frontend `<CoverArt>` component** (~300)
Component, initials-placeholder fallback, three sizes (thumb/card/hero).

**5.8 — Player Zustand store + audio singleton** (~450)
Store shape, actions (play/pause/next/prev/enqueue/shuffle/repeat), audio element ref outside React render.

**5.9 — `PlayerProvider` + `<audio>` wiring** (~350)
React context, subscribes to store, translates state → DOM API, `timeupdate`/`ended`/`error` handlers.

**5.10 — Now-playing bar component** (~450)
Cover thumb, title/artist links, play/pause/prev/next, scrubber, volume.

**5.11 — Queue drawer** (~350)
Drawer component, current-list + upcoming-list, click-to-jump.

**5.12 — File-selection rule + transcoded fallback** (~300)
Pure function (`pickFile(track, prefs)`), `error` handler swaps src to transcoded URL.

**5.13 — MediaSession integration** (~250)
`navigator.mediaSession.metadata` updates on track change, action handlers.

**5.14 — Global player hotkeys** (~250)
space/arrow/m/s/r wiring at root layout level.

## Risks / unknowns

- **ffmpeg child-process cleanup.** When the client disconnects mid-stream, the child must be killed. Use tokio's `Child::start_kill` on response stream drop. Test by aborting curl mid-download.
- **Browser FLAC support.** Modern Chrome/Firefox handle FLAC; older Safari doesn't. Detect via `<audio>.canPlayType('audio/flac')` to pre-route to transcoded URL rather than waiting for `error`. (Or just always rely on the error-fallback path; simpler.)
- **Cover art image size.** CAA returns full-size images (often >1MB). On-the-fly resize on every request would burn CPU. Disk cache solves this; verify cache TTL = forever (file content is immutable per `cover_art.id`).
- **MediaSession on Linux/Firefox.** Spotty support; degrade gracefully.
- **Audio element + React Strict Mode double-mount.** Strict-mode unmount/remount can stop audio. PlayerProvider needs to handle this — keep the audio element outside React's render via a ref + a singleton.

## Smoke test

```bash
# After phase 5 lands, smoke test = "use the app for an hour"
# Specifically:
# 1. Click play on 5 different albums (different formats: flac, mp3, m4a, opus)
# 2. Build a 20-track queue, shuffle on/off, repeat all
# 3. Skip + scrub + volume + pause/resume — all should be smooth
# 4. Lock screen on macOS — check Now Playing widget shows track + works
# 5. Plug in headphones with media keys, verify play/pause/next
# 6. Open in Safari (if Mac), play a FLAC, verify transcoded fallback kicks in
# 7. Verify import of a new MB-backed album fetches cover art into _cover_art_/
```
