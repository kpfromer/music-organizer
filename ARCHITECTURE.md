# Architecture: Services & Ports

This document describes the backend architecture for this project. It exists to keep the codebase consistent as we migrate from fat GraphQL resolvers to a clean layered structure.

## Why

Our GraphQL resolvers currently do three jobs at once:

1. Extract `AppState` from the async-graphql `Context`
2. Run business logic (validation, orchestration, multi-step workflows)
3. Execute Sea-ORM queries and map results to GraphQL response types

This makes the logic hard to test without a full GraphQL harness, impossible to reuse from CLI commands (`import`, `download`, `watch`), and easy to accidentally duplicate (e.g. "dismiss pending candidates" appears in multiple Spotify mutations).

We fix this by pulling business logic into **services** and defining **port traits** for external APIs. Resolvers become thin wrappers that extract context, call a service, and map the result.

## Directory Structure

```
src/
├── ports/                       # Traits for external APIs (mockable boundaries)
│   ├── mod.rs
│   └── spotify.rs               # SpotifyClient trait + API types
│
├── services/                    # Business logic as service structs
│   ├── mod.rs
│   ├── playlist.rs              # PlaylistService
│   ├── background/              # Background task infrastructure
│   ├── spotify/
│   │   ├── mod.rs
│   │   ├── client.rs            # SpotifyApiCredentials + SpotifyRsAdapter
│   │   ├── matching.rs          # SpotifyMatchingService
│   │   ├── sync.rs              # SpotifySyncService
│   │   ├── download_best_match_for_spotify_track.rs
│   │   ├── matching_local_tracks/   # Fuzzy matching engine + background task
│   │   └── sync_spotify_playlist_to_local_library/  # Playlist-to-local sync task
│   └── youtube/
│
├── http_server/
│   ├── graphql/                 # GraphQL resolvers (thin) + response types
│   │   ├── spotify/
│   │   │   ├── spotify_mutations.rs  # Thin: extract context -> call service -> map result
│   │   │   ├── spotify_queries.rs    # Query resolvers + GraphQL response types
│   │   │   └── context.rs           # Helper to build SpotifyRsAdapter from AppState
│   │   ├── playlist_mutations.rs
│   │   ├── playlist_queries.rs
│   │   └── ...
│   └── http_routes/             # REST endpoints (audio streaming, images, downloads)
│
├── entities/                    # Sea-ORM generated models (unchanged)
├── database.rs                  # Database wrapper (concrete, no trait)
├── main.rs                      # Wiring + CLI
└── config.rs                    # Configuration
```

## The Layers

### 1. Services (`src/services/`)

Services are structs that hold their dependencies and expose business operations as methods.

```rust
// services/spotify/matching.rs

pub struct SpotifyMatchingService {
    db: Arc<Database>,
}

impl SpotifyMatchingService {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    pub async fn accept_candidate(&self, candidate_id: i64) -> Result<()> {
        let candidate = self.find_candidate(candidate_id).await?;
        self.link_spotify_to_local(&candidate.spotify_track_id, candidate.local_track_id).await?;
        self.set_candidate_status(candidate_id, CandidateStatus::Accepted).await?;
        self.dismiss_pending_candidates(&candidate.spotify_track_id).await
    }
}
```

For services that depend on external APIs, the port trait becomes a generic parameter:

```rust
// services/spotify/sync.rs

pub struct SpotifySyncService<C: SpotifyClient> {
    db: Arc<Database>,
    client: C,
}

impl<C: SpotifyClient> SpotifySyncService<C> {
    pub async fn sync_account_playlists(&self, account_id: i64) -> Result<()> {
        let playlists = self.client.current_user_playlists().await?;
        // ... upsert logic using self.db ...
    }
}
```

**Conventions:**

- Services return `color_eyre::Result<T>`.
- Services return entity models directly (no separate domain models yet — we'll add them when needed).
- Services do **not** depend on `async_graphql`. No `Context`, no `#[Object]`, no `SimpleObject`.
- Use `.wrap_err("context")` over `map_err(|e| eyre!("msg: {}", e))` — it preserves the error chain.
- Services that only need the database hold a concrete `Arc<Database>`. Services that need external APIs use generic trait bounds for those dependencies.
- Services are constructed inline in resolvers (cheap `Arc` clone), not pre-built in `AppState`.

### 2. Ports (`src/ports/`)

Ports are traits that **wrap external services**. They are defined in `src/ports/` and implemented in `src/services/` (next to the adapter code).

```rust
// ports/spotify.rs

pub trait SpotifyClient: Send + Sync {
    fn current_user_playlists(&self) -> impl Future<Output = Result<Vec<SpotifyApiPlaylist>>> + Send;
    fn playlist_tracks(&self, playlist_id: &str) -> impl Future<Output = Result<Vec<SpotifyApiTrack>>> + Send;
}
```

**When to define a port:**

- External HTTP APIs (Spotify, Plex, YouTube, MusicBrainz) — these are slow, flaky, and need mocking in tests.
- External P2P protocols (Soulseek) — same reasoning.

**When NOT to define a port:**

- SQLite database access. We use a concrete `Database` type directly because we can test with an in-memory SQLite database. Adding a trait over Sea-ORM queries adds ceremony without real benefit.

### 3. GraphQL Resolvers (`src/http_server/graphql/`)

Resolvers are **thin**. They:

- Extract dependencies from the async-graphql `Context`
- Construct a service (inline, from `AppState` fields)
- Call service methods
- Map results to GraphQL response types

```rust
// http_server/graphql/spotify/spotify_mutations.rs

#[Object]
impl SpotifyMutation {
    async fn accept_spotify_match_candidate(
        &self,
        ctx: &Context<'_>,
        candidate_id: i64,
    ) -> GraphqlResult<bool> {
        let db = &get_app_state(ctx)?.db;
        let service = SpotifyMatchingService::new(db.clone());
        service.accept_candidate(candidate_id).await?;
        Ok(true)
    }
}
```

A resolver should **not**:

- Contain Sea-ORM queries directly
- Contain business logic (validation, multi-step orchestration, conditional branching)
- Construct `ActiveModel` instances or call `.insert()` / `.update()` / `.delete()`

## Error Handling

- **Services** return `color_eyre::Result<T>`.
- **Resolvers** convert to `GraphqlResult<T>` via the existing `From<color_eyre::Report> for GraphqlError` impl — this happens automatically with `?`.

Future improvement: define domain-specific error enums for cases where the caller needs to handle specific failure modes.

## Migration Strategy

We're migrating incrementally, not all at once. Priority order:

### Phase 1: High value (duplicated logic, complex orchestration) - DONE
1. **`SpotifyMatchingService`** — `accept_candidate`, `manually_match`, `dismiss_track`
2. **`SpotifySyncService`** — `sync_account_playlists` (with `SpotifyClient` port)
3. **`PlaylistService`** — `create`, `add_track`
4. **Move business logic modules** — `matching_local_tracks/`, `sync_spotify_playlist_to_local_library/`, `download_best_match_for_spotify_track.rs` from `graphql/spotify/` to `services/spotify/`

### Phase 2: Medium value (moderate complexity)
5. **`TrackService`** — `list` (search, sort, pagination), `get_with_artists`
6. **`PlexService`** — server mutations, playlist mutations, library refresh
7. **`SoulseekService`** — search and download orchestration
8. **`YoutubeService`** — subscription management, video queries

### Phase 3: Lower value (simple CRUD)
9. Simple queries that are just `Entity::find().all()` + mapping — migrate these last.

## What This Does NOT Cover (Future Work)

- **Domain models** — Services currently return entity models directly. When we need decoupling (e.g., different representations for different consumers), we'll add domain model types.
- **DataLoader / N+1 resolution** — Orthogonal to this architecture, can be added independently.
- **Domain-specific error enums** — Replace `color_eyre::Result` with typed errors where callers need to branch on failure modes.
