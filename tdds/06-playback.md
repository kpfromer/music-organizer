# 06 — Playback

> Browser-based playback of files served by the rust binary. Per Q10 answer: serve the original file when the browser supports it; otherwise transcode to a browser-friendly format while preserving quality as best we can.

## Endpoints

Two REST endpoints (not GraphQL — audio streaming is range-request-shaped, not query-shaped).

### `GET /api/files/:file_id/stream`

Headers: `Range`, `Accept`.

Behavior:
1. Look up the file. 404 if not found, 410 if `is_corrupt = 1` (UI should already gate this — defense in depth).
2. Decide based on `Accept` + format:
   - Browser sends `Accept: audio/*` typically; we look at the file's format.
   - If format ∈ {`mp3`, `m4a`, `aac`, `opus`}: passthrough — `tower-http`'s `ServeFile`/`ServeDir` style range support, return file directly with correct `Content-Type`.
   - If format ∈ {`flac`, `wav`, `ogg`}: depends on browser. Chrome/Firefox handle FLAC and Vorbis fine; Safari does not (as of 2026, FLAC support is incomplete on older Safari). For v2 simplicity: **always passthrough on /stream**, and a separate transcoded endpoint exists for the rare incompatibility case.

### `GET /api/files/:file_id/transcoded?fmt=opus&bitrate=128`

Forces transcoding. Frontend uses this when an `<audio>` element fires `error` on the passthrough URL.

Implementation:
- Spawns `ffmpeg` as a child process (we already need `fpcalc` from Chromaprint, so `ffmpeg` is also a documented system dependency in the Dockerfile).
- ffmpeg writes to stdout, axum streams it to the response (`StreamBody` over the child's stdout pipe).
- No range support on transcoded streams in v2 (browsers handle it via re-request and `Connection: close`; some seek-bar UX is degraded). Punt: HLS or pre-transcoded variants for true scrub-friendly playback.

`MM_TRANSCODE_TARGET` env var (default `opus@128k`) sets the default; query params override.

### `GET /api/cover_art/:cover_art_id`

Serves cover art images. Same `tower-http` static-file-style delivery. Cached aggressively (`Cache-Control: public, max-age=31536000, immutable`) — files are content-addressed by `cover_art.id`, never mutated.

Variants by query param: `?w=300` resizes on the fly via `image` crate. Resized variants are written to a disk cache (`MM_MUSIC_ROOT/.cover_art_cache/<id>-w300.jpg`) so the first request pays and subsequent requests serve from disk.

## Frontend playback model

### Now-playing bar

A persistent bottom bar (always rendered in the root layout). Shows: cover art thumb, track title (link), artists (links), play/pause, prev/next, scrubber, volume, queue button, shuffle toggle, repeat toggle.

### Queue

In-memory client-side state (Zustand or similar — keep it minimal). Shape:

```ts
type QueueItem = {
  trackId: number;
  fileId: number;        // resolved at enqueue time; if multiple files exist for a track, prefer best format per user pref
  source: "playlist:42" | "album:17" | "artist:9" | "search" | "ad-hoc";
};

type QueueState = {
  items: QueueItem[];
  currentIndex: number;
  shuffle: boolean;
  shuffleOrder: number[]; // index permutation when shuffle on
  repeat: "off" | "all" | "one";
};
```

Queue is **not** persisted across reloads in v2 (punt: persist to localStorage; simple but not in scope).

### File-selection rule

When a track has multiple files, the player picks one per:
1. Skip files with `is_corrupt = 1`.
2. Prefer formats in user-pref order (default: `flac > opus > m4a > mp3 > ogg > wav`).
3. Prefer higher bitrate within a format.
4. Tiebreak by lowest `file.id` (stable).

Pref order configurable via a settings modal (writes to localStorage; not server-side state in v2).

### Audio element

A single `HTMLAudioElement` instance owned by a context provider. On `ended` → advance queue. On `error` → fall back to `/transcoded?fmt=opus`; if that also errors, skip to next track and surface a toast.

### MediaSession

Wire `navigator.mediaSession.metadata` (title/artist/artwork) and action handlers (play/pause/prev/next) so OS-level controls (macOS Now Playing, headphones) work.

### Shuffle semantics

- **Pure Fisher-Yates over the queue, recomputed each time shuffle toggles on.** No "smart" / weighted / artist-spreading shuffle, ever. Random means random.
- The currently-playing track stays at index 0 of the new shuffle order.

### Repeat

- `off`: stop after last item.
- `all`: wrap to first.
- `one`: replay current track on `ended`.

## Performance / scale

- Library expected size: tens of thousands of files single-user.
- No server-side state per playback session — every request is independent.
- `/stream` passthrough is essentially zero CPU.
- Transcoded endpoint is the only CPU spike; expect rare use.
- Cover art resizing has the disk cache so it's a one-time cost per (cover, size).

## Future work

- Gapless playback (Web Audio API instead of `<audio>`; preload next track's buffer; punt).
- Crossfade (punt).
- Replay-gain normalization (punt).
- Cast / AirPlay (punt).
- HLS streaming for proper transcoded-seek (punt).
- Persistent queue across reloads.
