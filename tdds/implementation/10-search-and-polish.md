# Phase 10 — Search & Polish

> Global search, settings page, comprehensive empty/loading/error states across the app. Smallest phase by feature surface; meaningful by daily-use quality.

## Goals

- `/search?q=` route with tabs: Artists / Albums / Tracks / Files.
- Global top-bar search input (focus via `/` hotkey) submitting to `/search?q=...`.
- New GraphQL queries: `search(q, limit, offset)` returning typed connections per type.
- `/settings` page with localStorage-backed preferences (per TDD 06 / TDD 05):
  - Preferred file format priority list.
  - `matchCommitDelayMs`.
  - `matchQueueMaxDepth`.
  - Frontend version + build info.
- Empty states for every list view (no playlists, no wishlist items, no unmatched files, etc.).
- Loading skeletons for every detail view + table.
- Error boundary at root + per-route; surfaces graphql `code` extension via toast helper.
- 404 page.
- Cosmetic polish: spacing, dark/light mode default-system check, MediaSession edge cases verified.

## Acceptance criteria

- [ ] Press `/` from anywhere → top-bar search focuses → typing + Enter navigates to `/search?q=...`.
- [ ] `/search` shows tabs with counts; clicking a tab filters; results paginate.
- [ ] Empty DB → all browse views show specific empty states (not the same generic one).
- [ ] Initial page load shows skeletons, then real data.
- [ ] Settings page: change preferred-format order → next track-play picks file matching new pref.
- [ ] Settings: set `matchCommitDelayMs = 0` → matching UI commits instantly with no toast.
- [ ] Server returns 500 on a query → error boundary shows actionable message + reload button.
- [ ] 404 on unknown route.
- [ ] Lighthouse / a11y pass: keyboard navigation reaches every interactive element; aria labels on icon-only buttons.

## TDD references

- TDD 09 §Queries (search subset to be added)
- TDD 10 §Routes, §Search, §Theming
- TDD 13 §Naming, §Styling

## Out of scope

- Cmd+K command palette (punted).
- Customizable hotkeys via UI (punted).
- Full a11y audit / WCAG compliance (we make a best-effort pass; not promised).

## PR breakdown

**10.1 — `search` GraphQL query** (~400)
LIKE-based search across artists/albums/tracks/files; per-type connections; tests.

**10.2 — Top-bar search input + `/` hotkey** (~300)
Global input in layout, focus on `/`, Enter navigates to `/search?q=`.

**10.3 — `/search` route with tabs** (~450)
Tabs Artist/Album/Track/File, paginated results, search-param-driven.

**10.4 — `useLocalStorage<T>` hook + cross-tab sync** (~250)
Reusable hook with `storage` event listener; tests.

**10.5 — `/settings` route** (~400)
Form for preferred-format priority, match commit delay, queue depth; reads/writes via the hook.

**10.6 — Preferred-format wired into player** (~250)
File-selection rule reads from settings hook.

**10.7 — Match-queue settings wired** (~250)
Queue store reads delay/depth from settings.

**10.8 — Empty states sweep** (~350)
Per-list specific empty states across the app.

**10.9 — Loading skeletons + error boundary + 404** (~450)
Skeletons for tables/details, root error boundary with toast helper, 404 route.

**10.10 — Theme: system pref + dark/light vars** (~250)
Tailwind class strategy + matchMedia listener.

## Risks / unknowns

- **Search performance at scale.** SQLite `LIKE '%query%'` over titles is OK for tens of thousands of tracks but degrades. If it's slow, add an FTS5 virtual table mirror (but that's a punt unless it actually hurts).
- **Settings synchronization.** localStorage is per-browser; multiple tabs need a `storage` event listener to react. Implement; minor.
- **Theme.** Tailwind `class` strategy + system pref via `useEffect(() => matchMedia(...))`. Easy.

## Smoke test

```bash
# Use the app for a week. Note any:
# - Empty state that's missing or generic
# - Loading state that flickers wrong
# - Error that crashes a route instead of being caught
# - Keyboard interaction that's broken
# - Setting that doesn't take effect immediately

# Then: file fixes, ship the phase, declare v2 ready for daily use.
```

## v2 done — what next?

Once phase 10 lands, v2 is the spec. From here, "future work" items live as GitHub / forgejo issues, prioritized by what bugs you in daily use. The largest deferred buckets:
- MP3-player SD-card sync CLI (TDD 00 punt).
- Refresh-from-MB tooling (TDD 04 punt).
- Delete / cleanup operations (gap 3 punt).
- Listen history + scrobbling (TDD 00 punt).
- Gapless / replay-gain (TDD 00 punt).
- Smart playlists (TDD 00 punt).
- Persistent queue across reloads (TDD 06 punt).
