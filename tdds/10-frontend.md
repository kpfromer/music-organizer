# 10 — Frontend

> React 18 + TypeScript, strictly client-side, served as static assets by the rust binary. Spotify-like layout: persistent sidebar + persistent now-playing bar + scrollable main view.

## Stack (locked in TDD 00)

- pnpm + Vite.
- React 18, TypeScript strict mode.
- TanStack Router (client-side routing, search-params-first).
- TanStack Query for server state.
- TanStack Table for grids.
- TanStack Form for forms.
- TanStack Hotkeys (or `react-hotkeys-hook` fallback).
- shadcn/ui + Tailwind CSS for components/styling.
- `graphql-codegen` for typed GraphQL hooks.
- Zustand for the few pieces of pure-client state (queue, player).

## Layout

```
┌─────────────────────────────────────────────────────────────┐
│ [sidebar]  │  [main view: scrollable, route-driven]         │
│            │                                                │
│  Library   │                                                │
│  - Tracks  │                                                │
│  - Albums  │                                                │
│  - Artists │                                                │
│            │                                                │
│  Playlists │                                                │
│  - Liked   │                                                │
│  - <each>  │                                                │
│            │                                                │
│  System    │                                                │
│  - Imports │                                                │
│  - Match   │                                                │
│  - Wishlist│                                                │
│            │                                                │
├─────────────────────────────────────────────────────────────┤
│  [now-playing bar: art | title/artist | controls | scrub]   │
└─────────────────────────────────────────────────────────────┘
```

The sidebar and now-playing bar are rendered by the root layout component; routes only render into the main view.

## Routes

(TanStack Router file-based.)

```
/                         → redirects to /library/tracks
/library/tracks           → TrackList
/library/albums           → AlbumGrid
/library/albums/$id       → AlbumDetail
/library/artists          → ArtistList
/library/artists/$id      → ArtistDetail (top tracks, albums)
/playlists                → PlaylistList
/playlists/$id            → PlaylistDetail
/playlists/import         → MassImport
/wishlist                 → WishlistList
/wishlist/$id             → WishlistDetail
/imports                  → ImportProgressList
/matching                 → UnmatchedFilesList
/matching/$fileId         → MatchDetail
/upload                   → DropZone (also: drop anywhere on app for upload)
/search?q=                → Global search across artists/albums/tracks/files (tabs)
/settings                 → user prefs (file format pref, etc. — localStorage-backed)
```

### Search params over state

Per the spec — list filters, sort, pagination all live in URL:

- `/library/tracks?q=Pearl+Jam&sort=title&page=2`
- `/matching?band=high&format=flac`
- `/wishlist?status=FAILED`

Reload preserves view; sharing a URL works (single-user, but useful for browser tabs).

## Data fetching

`graphql-codegen` produces typed query hooks:

```ts
const { data, isPending } = useTracksQuery({ search, limit: 50, offset });
```

Conventions:
- One query per route, fetched at the route loader level when possible.
- Mutations invalidate the relevant query keys (no manual cache patches in v2).
- Error toasts via shadcn `Sonner` triggered by a global `QueryClient` `onError`.

## Component library conventions

- `components/ui/` — shadcn-generated primitives.
- `components/forms/` — composed form fields wrapping TanStack Form bindings (Input, Select, Combobox-with-async). Forms always use these.
- `components/tables/` — composed TanStack Table wrappers (sortable header cells, pagination footer, empty states).
- `components/player/` — now-playing bar + queue drawer.
- Feature folders mirror routes: `features/library/`, `features/wishlist/`, etc.

## Player (client-side state)

`features/player/store.ts`:

```ts
export const usePlayerStore = create<PlayerState>((set, get) => ({
  queue: [],
  currentIndex: -1,
  shuffle: false,
  shuffleOrder: [],
  repeat: "off",
  isPlaying: false,
  positionSec: 0,
  durationSec: 0,
  // actions...
  playTrack(trackId, source) { ... },
  enqueue(trackIds) { ... },
  next() { ... },
  prev() { ... },
  togglePlay() { ... },
  toggleShuffle() { ... },
  setRepeat(mode) { ... },
}));
```

Audio element lives in a single `PlayerProvider` that subscribes to the store and translates state → DOM API calls. Position/duration come from `timeupdate` events.

## Hotkeys

Global:
- `space` — play/pause
- `→` / `←` — next / prev track
- `shift+→` / `shift+←` — seek ±10s
- `m` — mute
- `s` — toggle shuffle
- `r` — cycle repeat
- `/` — focus global search (top-bar input; submit → navigate to `/search?q=...`)

Page-specific (matching detail) per TDD 05.

## Drop zone

A wrapper at the root level that listens for `dragenter`/`dragover`/`drop` on `window` (only when files dragged from OS, detected via `dataTransfer.types.includes("Files")`). On drop:
- Show a full-screen overlay with file list.
- POST each to `/api/upload` (multipart, one at a time, with progress).
- Toast on each completion.

## Cover art handling

`<CoverArt coverArtId={...} size="thumb" />` resolves to `<img src="/api/cover_art/:id?w=64" loading="lazy" />`. Sizes: `thumb` (64), `card` (200), `hero` (600).

Fallback: SVG placeholder generated from album/artist initials (deterministic color from id hash).

## Theming

Light + dark via Tailwind `class` strategy + system pref. shadcn defaults. No theme switcher in v2 settings — punt; respect OS only.

## Bundle delivery

Two modes:
- **Embedded** (release): `rust-embed` bakes the `frontend/dist/` into the binary; `mm-server` serves at `/`.
- **Dev split** (`MM_FRONTEND_DIR=...`): rust serves from disk, Vite dev server runs separately on port 5173 with proxy for `/graphql` and `/api/*`.

## Type safety contract

The frontend never sees raw GraphQL strings at runtime — all queries go through codegen-produced hooks. CI runs `pnpm graphql-codegen` against a saved schema file (`frontend/schema.graphql` exported from the rust binary) and fails if generated artifacts drift.

## Test plan

- Vitest for store logic (queue advancement, shuffle ordering, repeat).
- Vitest + React Testing Library for the matching detail page (the most logic-heavy view).
- No e2e in v2 (punt).

## Future work

- Persistent queue across reloads (localStorage).
- Mobile-responsive layout.
- Theme switcher.
- Keyboard-driven command palette (`cmd+k`).
- Visualizer.
- Lyrics pane.
