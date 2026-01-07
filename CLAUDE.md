# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Music Manager is a full-stack music library management application that imports music files with automatic metadata enrichment (AcoustID + MusicBrainz), provides a web interface for browsing/playback, and integrates with Soulseek for peer-to-peer music downloads and Plex for library synchronization.

**Tech Stack:**
- Backend: Rust (Axum + async-graphql + SeaORM + SQLite)
- Frontend: React 19 + Bun + Rspack + TanStack (Form/Query/Table) + Tailwind CSS
- Build Tool: just (command runner)

## Common Commands

All commands use `just` (justfile runner). Run `just --list` to see all available commands.

### Development

```bash
# Start both backend and frontend in watch mode
just watch

# Backend only (with directory watching)
just backend-watch

# Frontend only (dev server + codegen watcher)
just frontend-dev

# GraphQL type generation (requires backend running on :3000)
just frontend-codegen
just generate-graphql-types  # Automated: starts backend, runs codegen, stops backend
```

### Building

```bash
# Build both frontend and backend for production
just build

# Backend only (release mode)
just backend-build-release

# Frontend only (production build to frontend/dist/)
just frontend-build
```

### Testing & Linting

```bash
# Run all checks (format check + lint + typecheck for both)
just check

# Backend checks (format check + clippy + tests)
just backend-check
just backend-test           # Run Rust tests only
just backend-lint           # Clippy with -D warnings

# Frontend checks (biome check + TypeScript typecheck)
just frontend-check
just frontend-lint          # Biome linting
just frontend-check-typescript

# Format code
just format                 # Both frontend and backend
just backend-format         # cargo fmt
just frontend-format        # biome format

# Auto-fix frontend issues
just frontend-fix           # biome format + check --write
```

### Database Migrations

```bash
# Run pending migrations
just migrate-up

# Rollback last migration
just migrate-down

# Create new migration
just migrate-generate "migration_name"
```

Database file location: `./music/library.db` (SQLite)

### Backend CLI Commands

The Rust binary has multiple subcommands:

```bash
# Import music from directory
cargo run -- import --directory ./music

# Watch directory and auto-import
cargo run -- watch --directory ./music

# Start HTTP server (GraphQL + REST)
cargo run -- serve --log-level info --directory downloads

# Soulseek downloader TUI
cargo run -- download

# Config commands
cargo run -- config show
cargo run -- config set <key> <value>
```

## Architecture

### Backend Structure (`src/`)

```
src/
├── main.rs                    # CLI entry point (clap commands)
├── http_server/              # HTTP server (Axum)
│   ├── app.rs               # Server setup, routing, background tasks
│   ├── state.rs             # AppState (DB, Soulseek client, config)
│   ├── graphql/             # async-graphql schema & resolvers
│   │   ├── mod.rs          # Query/Mutation root, schema builder
│   │   ├── query_builder/  # Pagination/search/sort helpers
│   │   ├── soulseek_mutations.rs
│   │   ├── playlist_*.rs
│   │   └── plex_*.rs
│   └── http_routes/         # REST endpoints
│       ├── audio_file.rs          # Audio streaming (HTTP Range support)
│       ├── album_art_image.rs     # Album art extraction
│       └── download_file.rs       # Soulseek download (NDJSON streaming)
├── soulseek/                # Soulseek P2P client wrapper
│   ├── client.rs           # Async wrapper around sync soulseek-rs-lib
│   └── types.rs            # Search config, track, file result types
├── entities/                # SeaORM database models
│   ├── track.rs, album.rs, artist.rs, track_artist.rs
│   ├── playlist.rs, playlist_track.rs
│   ├── plex_server.rs
│   └── unimportable_file.rs
├── database.rs              # SeaORM operations (upsert_artist, insert_track, etc.)
├── import_track.rs          # Track import pipeline (metadata enrichment)
├── musicbrainz.rs           # MusicBrainz API integration
├── acoustid.rs              # AcoustID fingerprint lookup
├── chromaprint.rs           # Audio fingerprinting
└── config.rs                # Configuration file management
```

**Key Backend Patterns:**

1. **AppState as Shared Context**: All handlers receive `Extension<AppState>` with DB connection, Soulseek client, config
2. **GraphQL API**: Primary API surface at `/graphql` (POST for queries/mutations, GET for GraphiQL)
3. **HTTP Routes for Media**: Separate REST endpoints for audio streaming and downloads
4. **Background Tasks**: Directory watcher spawned on server start, Soulseek session watchdog
5. **Async/Sync Boundary**: Soulseek sync client wrapped with `spawn_blocking` for tokio integration
6. **Rate Limiting**: Governor rate limiter on Soulseek searches (34 searches per 220 seconds)

### Frontend Structure (`frontend/src/`)

```
frontend/src/
├── index.ts                  # Bun server (SPA routing, HMR in dev)
├── App.tsx                   # React Router route definitions
├── pages/                    # Page components
│   ├── tracks.tsx           # Track browser (pagination, search, sort)
│   ├── download.tsx         # Soulseek search & download UI
│   ├── playlists.tsx        # Playlist management
│   ├── plex-servers.tsx     # Plex OAuth & server management
│   └── unimportable-files.tsx
├── components/
│   ├── audio-player.tsx     # Fixed bottom audio player
│   ├── app-sidebar.tsx      # Navigation sidebar
│   ├── form/                # TanStack Form helpers
│   └── ui/                  # Radix UI primitives (shadcn pattern)
├── stores/
│   └── audio-player-store.ts  # Zustand store for playback state
├── lib/
│   ├── execute-graphql.ts   # GraphQL client (POST to backend)
│   └── query-builder.ts     # Query parameter builders
└── graphql/                 # Auto-generated from backend schema
    ├── gql.ts              # Document type helper
    └── graphql.ts          # Generated types (~1000+ lines)
```

**Key Frontend Patterns:**

1. **TanStack Form + Zod**: See `.cursorrules` for comprehensive form building pattern (form-level schema validation, never manual state)
2. **GraphQL Code Generation**: TypeScript types auto-generated from backend schema via `@graphql-codegen`
3. **Bun Server**: Custom `Bun.serve()` for SPA routing (NOT Vite/Next.js)
4. **Zustand for Player State**: Audio playback state managed globally, persists volume to localStorage
5. **Streaming Downloads**: `/download-file` endpoint returns NDJSON progress events
6. **Audio Streaming**: `/audio-file/{track_id}` with HTTP Range support for seeking

### Data Flow

**GraphQL Communication:**
```typescript
// Frontend
const TracksQuery = graphql(`query Tracks($pagination: PaginationInput) { ... }`);
const result = await execute(TracksQuery, { pagination: { page: 1, pageSize: 25 } });

// Backend (async-graphql)
#[Object]
impl Query {
    async fn tracks(&self, ctx: &Context<'_>, pagination: Option<PaginationInput>) -> Result<Vec<Track>> {
        // SeaORM query with pagination
    }
}
```

**Audio Playback Flow:**
1. User clicks track → `audio-player-store.ts` `playTrack(track)`
2. Audio element `src` set to `/audio-file/{track_id}`
3. Browser requests file (sends Range header for seeking)
4. Backend streams audio with MIME type detection
5. Album art loaded from `/album-art-image/{track_id}`

**Import Workflow:**
1. File detected in watch directory
2. Compute SHA-256 hash → Extract chromaprint fingerprint
3. AcoustID lookup → MusicBrainz fetch (recording details)
4. Duplicate check by MusicBrainz ID
5. Upsert artists/album → Insert track with relationships
6. On error → Mark in `unimportable_file` table with reason

**Soulseek Download:**
1. Frontend searches via `searchSoulseek` mutation
2. Backend queries Soulseek with multiple search strings (removes diacritics)
3. Results ranked by bitrate, duration match, encoder quality
4. Frontend POSTs to `/download-file` with file details
5. Backend streams NDJSON progress events (`{ type: "Progress", bytesDownloaded, totalBytes }`)
6. File written to `downloads/` directory

### Environment Configuration

**Backend** (via `.env`, CLI args, or config file):
- `MUSIC_MANAGER_HTTP_PORT`: Server port (default 3000)
- `MUSIC_MANAGER_WATCH_DIRECTORY`: Directory to monitor
- `ACOUSTID_API_KEY`: AcoustID API key (required for metadata)
- `SOULSEEK_USERNAME` / `SOULSEEK_PASSWORD`: Soulseek credentials
- `SOULSEEK_DOWNLOAD_DIRECTORY`: Download target directory
- `BASE_URL`: CORS/auth redirect base (required in production for Plex OAuth)
- `MUSIC_MANAGER_CONFIG`: Config file path
- `MUSIC_MANAGER_LOG_FILE`: Log file path

**Frontend** (via `.env` in `frontend/`):
- `PUBLIC_GRAPHQL_URL`: Backend GraphQL endpoint (e.g., `http://localhost:3000/graphql`)

Build-time configuration uses rspack (see `frontend/rspack.config.js`).

## Important Patterns

### Form Building (TanStack Form + Zod)

**Always follow this pattern** (detailed in `.cursorrules`):

1. Define Zod schema first (provides both validation and TypeScript types)
2. Use form-level validation (`validators.onChange: schema`) for automatic error propagation
3. Use `useAppForm` hook with schema
4. Use `form.AppField` for each field (never raw inputs or `useState`)
5. Wrap fields in `FormFieldContainer` for consistent styling
6. Use `form.FormSubmitButton` for submission with loading states

**Example:**
```typescript
const formSchema = z.object({
  trackTitle: z.string().min(1, "Track title is required"),
  albumName: z.string().optional(),
});

type FormData = z.infer<typeof formSchema>;

const form = useAppForm({
  defaultValues: { trackTitle: "", albumName: "" },
  validators: { onChange: formSchema }, // Form-level validation
  onSubmit: async ({ value }: { value: FormData }) => { /* ... */ },
});

return (
  <form.AppForm>
    <form.AppField name="trackTitle">
      {() => (
        <FormFieldContainer label="Track Title *">
          <FormTextField placeholder="Enter title" />
        </FormFieldContainer>
      )}
    </form.AppField>
    <form.FormSubmitButton label="Submit" />
  </form.AppForm>
);
```

**Never:**
- Use `useState` for form fields
- Use raw `<Input>` components (use `FormTextField`)
- Manually type form data (use `z.infer<typeof schema>`)
- Skip `FormFieldContainer` wrapper
- Use old modal patterns (CreateArtistModal, CreateAlbumModal) with manual state

### Soulseek Client Usage

The `SoulSeekClient` wrapper manages session state automatically:

```rust
// Search for track
let results = soulseek_client.search_for_track(&track, limit).await?;

// Download file
let (rx, _handle) = soulseek_client.download_file(username, path, filename).await?;
while let Some(progress) = rx.recv().await {
    // Handle progress
}
```

**Session State Machine:**
- `Disconnected` → `Connecting` → `LoggedIn` → (on error) → `Backoff` → `Connecting`
- Exponential backoff on connection failures
- Session watchdog reconnects on timeouts

### Database Operations

Use `Database` struct methods (NOT raw SeaORM queries) for consistency:

```rust
// Insert track with artist relationships
db.insert_track(
    album_id,
    track_number,
    title,
    duration_seconds,
    &artist_ids,
    &file_path,
    sha256_hash,
).await?;

// Upsert artist (idempotent)
let artist = db.upsert_artist(name, mb_id, sort_name).await?;

// Mark failed import
db.mark_unimportable_file(&file_path, &error_json).await?;
```

### GraphQL Query Building

Use query builder helpers for pagination/search/sort:

```rust
use crate::http_server::graphql::query_builder::*;

let mut query = entities::track::Entity::find();
query = apply_pagination(query, pagination);
query = apply_multi_column_text_search(query, search, &[entities::track::Column::Title]);
query = apply_sort(query, sort, &column_map);
let tracks = query.all(&db.conn).await?;
```

**Common Inputs:**
- `PaginationInput { page: i64, page_size: i64 }`: Default page_size is 25
- `TextSearchInput { search: String }`: Full-text search
- `TrackSortInput { created_at: SortDirection?, duration: SortDirection? }`: Multi-column sort

## Development Tips

1. **GraphQL Schema Changes**: After modifying GraphQL schema in `src/http_server/graphql/`, run `just frontend-codegen` or `just generate-graphql-types` to update TypeScript types

2. **Database Migrations**: After creating entities, generate migration with `just migrate-generate "name"`, then apply with `just migrate-up`

3. **Bun NOT Node**: Frontend uses Bun runtime exclusively (see `frontend/CLAUDE.md` for Bun-specific APIs)

4. **Production Build**: Frontend builds to `frontend/dist/`, which backend serves as static files in release mode

5. **CORS**: Backend enables CORS for all origins in development, requires `BASE_URL` in production

6. **Parallel Execution**: Use `[parallel]` in justfile for concurrent tasks (`just watch` runs backend + frontend simultaneously)

7. **Log Levels**: Backend supports `--log-level` flag (trace, debug, info, warn, error)

8. **Soulseek Rate Limits**: Backend enforces 34 searches per 220 seconds, frontend should debounce search requests

9. **Audio Formats**: Supported import formats: mp3, flac, m4a, aac, ogg, wav (detected via `infer` crate)

10. **N+1 Query Issue**: `get_track_artists()` in `database.rs` has known N+1 issue (load artists individually per track) - consider using `find_with_related` for bulk operations
