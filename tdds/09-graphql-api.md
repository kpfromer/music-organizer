# 09 — GraphQL API

> Single GraphQL endpoint at `POST /graphql` for almost everything. A handful of REST endpoints exist where GraphQL is a poor fit.

## Stack

- `async-graphql` 7.x with `async-graphql-axum` adapter.
- Schema-first, but the schema *is* generated from rust types (we don't hand-write SDL).
- Frontend uses `graphql-codegen` against the introspection schema → typed React-Query hooks.
- Single endpoint at `/graphql`, plus `/graphql/playground` enabled only when `MM_LOG=debug` or env `MM_GRAPHQL_PLAYGROUND=1`.

## REST exceptions

These do not go through GraphQL:

| Endpoint | Why |
|---|---|
| `GET /api/files/:id/stream` | Range requests, binary stream. |
| `GET /api/files/:id/transcoded` | Streaming transcoded audio. |
| `GET /api/cover_art/:id` | Binary image, aggressive caching. |
| `POST /api/upload` | Multipart upload of large files. |
| `GET /healthz`, `GET /readyz` | Liveness/readiness for ops. |

## Schema overview

Sketch — types and key fields. This is not exhaustive; resolver-level fields are added as the UI needs them.

### Object types

```graphql
type Artist {
  id: ID!
  name: String!
  sortName: String!
  musicbrainzId: String
  source: Source!
  albums(limit: Int, offset: Int): AlbumConnection!
  tracks(limit: Int, offset: Int): TrackConnection!
  coverArt: CoverArt
}

type Album {
  id: ID!
  title: String!
  releaseYear: Int
  releaseDate: String
  musicbrainzReleaseId: String
  musicbrainzReleaseGroupId: String
  source: Source!
  artists: [Artist!]!         # ordered by position
  tracks: [Track!]!           # ordered by disc, track number
  coverArt: CoverArt
}

type Track {
  id: ID!
  title: String!
  trackNumber: Int
  discNumber: Int
  durationMs: Int!
  musicbrainzRecordingId: String
  isrc: String
  source: Source!
  album: Album
  artists: [Artist!]!
  files: [File!]!
  preferredFile: File           # applies file-selection rule (TDD 06)
}

type File {
  id: ID!
  trackId: ID
  relativePath: String!
  originalFilename: String!
  format: String!
  sizeBytes: Int!
  isCorrupt: Boolean!
  corruptionReason: String
  audioInfo: FileAudioInfo!
  importedAt: Int!
  streamUrl: String!            # /api/files/:id/stream — convenience
}

type FileAudioInfo {
  durationMs: Int!
  sampleRateHz: Int!
  channels: Int!
  bitRateKbps: Int
  bitDepth: Int
  codec: String!
  decodeSoftErrors: Int!
}

type CoverArt {
  id: ID!
  kind: CoverArtKind!
  source: CoverArtSource!
  url(width: Int): String!      # resolves to /api/cover_art/:id?w=N
}

type Playlist {
  id: ID!
  name: String!
  description: String
  trackCount: Int!
  pendingCount: Int!            # wishlist items still resolving
  tracks(limit: Int, offset: Int): PlaylistTrackConnection!
  pending: [PlaylistPending!]!
  createdAt: Int!
  updatedAt: Int!
}

type PlaylistTrack {
  position: Int!
  addedAt: Int!
  track: Track!
}

type PlaylistPending {
  position: Int!
  wishlistItem: WishlistItem!
}

type WishlistItem {
  id: ID!
  queryTitle: String!
  queryArtist: String!
  queryAlbum: String
  targetDurationMs: Int
  preferredFormats: [String!]!
  status: WishlistStatus!
  attemptsCount: Int!
  lastAttemptAt: Int
  nextRetryAt: Int
  errorReason: String
  progressText: String
  resultingTrack: Track
  createdAt: Int!
  updatedAt: Int!
}

type ImportProgress {
  id: ID!
  originalFilename: String!
  stage: ImportStage!
  branch: ImportBranch
  errorMessage: String
  file: File
  createdAt: Int!
  updatedAt: Int!
}

type Candidate {
  source: CandidateSource!      # LOCAL_TRACK or MB_RECORDING
  localTrack: Track             # one of these set
  mbRecording: MbRecording
  score: Float!
  reasons: [MatchReason!]!
}

# enums
enum Source { MUSIC_BRAINZ USER_CREATED }
enum WishlistStatus { PENDING SEARCHING DOWNLOADING CHECKING IMPORTING COMPLETED FAILED }
enum ImportStage { QUEUED PREFLIGHT TAGS IDENTIFY AUDIO_CHECK MOVE DB COVER_ART DONE FAILED }
enum ImportBranch { A_MB_TAGS B_FINGERPRINT C_UNMATCHED }
enum CoverArtKind { FRONT BACK ARTIST OTHER }
enum CoverArtSource { CAA EMBEDDED USER_UPLOAD }
enum CandidateSource { LOCAL_TRACK MB_RECORDING }
enum MatchReason {
  ACOUSTID_HIGH ACOUSTID_MODERATE TITLE_EXACT TITLE_FUZZY
  ARTIST_EXACT ARTIST_FUZZY DURATION_WITHIN_3S ISRC_MATCH MB_RECORDING_ID_IN_TAGS
}
```

### Pagination

Cursor-less offset/limit (Connection wrappers carry `totalCount`, `nodes`, `hasMore`). Single user — we don't need cursor pagination's properties.

```graphql
type TrackConnection { totalCount: Int!, nodes: [Track!]!, hasMore: Boolean! }
# similar for AlbumConnection, ArtistConnection, etc.
```

### Queries (sketch)

```graphql
type Query {
  artist(id: ID!): Artist
  artists(search: String, limit: Int = 50, offset: Int = 0): ArtistConnection!
  album(id: ID!): Album
  albums(search: String, artistId: ID, limit: Int = 50, offset: Int = 0): AlbumConnection!
  track(id: ID!): Track
  tracks(search: String, albumId: ID, artistId: ID, limit: Int = 100, offset: Int = 0): TrackConnection!
  file(id: ID!): File
  unmatchedFiles(limit: Int = 50, offset: Int = 0, scoreBand: ScoreBand): FileConnection!
  candidates(fileId: ID!, refreshAcoustId: Boolean = false): [Candidate!]!
  playlist(id: ID!): Playlist
  playlists(search: String, limit: Int = 50, offset: Int = 0): PlaylistConnection!
  wishlistItems(status: WishlistStatus, limit: Int = 50, offset: Int = 0): WishlistItemConnection!
  recentImports(limit: Int = 50): [ImportProgress!]!
  systemStatus: SystemStatus!
}
```

### Mutations (sketch)

```graphql
type Mutation {
  # Library
  rescanFolder: Boolean!  # nudges the watch-folder task to rescan now
  reprocessFile(fileId: ID!): ImportProgress!
  markFileNotCorrupt(fileId: ID!): File!  # manual override
  retryUnimportableFile(sha256: String!): Boolean!

  # Matching
  matchFileToLocalTrack(fileId: ID!, trackId: ID!): File!
  matchFileToMbRecording(fileId: ID!, recordingId: String!, releaseId: String!): File!
  createUserTrackForFile(fileId: ID!, input: CreateUserTrackInput!): File!
  promoteUserTrackToMb(trackId: ID!, recordingId: String!, releaseId: String!): Track!
  searchMb(query: String!, kind: MbSearchKind!): [MbRecording!]!

  # Playlists
  createPlaylist(name: String!, description: String): Playlist!
  renamePlaylist(id: ID!, name: String, description: String): Playlist!
  deletePlaylist(id: ID!): Boolean!
  addTracksToPlaylist(id: ID!, trackIds: [ID!]!, position: Int): Playlist!
  removeTracksFromPlaylist(id: ID!, trackIds: [ID!]!): Playlist!
  reorderPlaylist(id: ID!, trackIds: [ID!]!): Playlist!
  importPlaylist(name: String!, format: PlaylistImportFormat!, content: String!): PlaylistImportResult!

  # Wishlist
  createWishlistItem(input: CreateWishlistItemInput!): WishlistItem!
  retryWishlistItem(id: ID!): WishlistItem!
  cancelWishlistItem(id: ID!): Boolean!
  editWishlistItem(id: ID!, input: EditWishlistItemInput!): WishlistItem!
}
```

### Subscriptions

**None in v2.** Per the discussion in the planning round — single user, polling via React Query is sufficient. If we later want push, async-graphql supports it.

## Error model

`async-graphql`'s default `errors` array. Resolvers return `Result<T, AppError>` where `AppError` is our crate-internal error converted to `async_graphql::Error` with extension fields:

```json
{
  "extensions": {
    "code": "NOT_FOUND",
    "domain": "track"
  }
}
```

Codes (closed set):
- `NOT_FOUND` — entity missing.
- `VALIDATION` — input invalid; `extensions.fields` lists the bad fields.
- `CONFLICT` — e.g. playlist name collision.
- `INTERNAL` — bug or unexpected upstream failure; logged with full stack.
- `UPSTREAM` — MB / AcoustID / Soulseek failure; `extensions.upstream` = which one.

Frontend has a thin error-toast helper that renders by `code`.

## Polling cadences (frontend conventions)

Set as default `refetchInterval` per query on the React Query side:

| Query | Interval |
|---|---|
| `recentImports` (when any non-terminal stage exists) | 2s |
| `wishlistItems` (when any transient status exists) | 2s |
| `systemStatus` | 10s |
| Library browse queries | none (manual refresh) |
| `candidates` (matching detail) | none (refresh on demand) |

`refetchOnWindowFocus` defaults to true.

## Versioning

GraphQL is additive — fields can be added freely, removals require care. v2 ships `v0.1`. We don't expose schema versioning explicitly; `systemStatus` returns `appVersion` for debugging.

## DataLoader

Use `async-graphql`'s `DataLoader` for N+1-prone fields:
- `Track.album` (when listing many tracks).
- `Track.artists`, `Album.artists`.
- `Track.preferredFile` and `Track.files`.
- `WishlistItem.resultingTrack`.

## Future work

- Subscriptions for live progress.
- Persisted queries for the bundle to avoid sending query strings.
- Schema introspection disabled in prod.
