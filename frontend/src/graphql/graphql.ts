/* eslint-disable */
import { DocumentTypeDecoration } from '@graphql-typed-document-node/core';
export type Maybe<T> = T | null;
export type InputMaybe<T> = T | null | undefined;
export type Exact<T extends { [key: string]: unknown }> = { [K in keyof T]: T[K] };
export type MakeOptional<T, K extends keyof T> = Omit<T, K> & { [SubKey in K]?: Maybe<T[SubKey]> };
export type MakeMaybe<T, K extends keyof T> = Omit<T, K> & { [SubKey in K]: Maybe<T[SubKey]> };
export type MakeEmpty<T extends { [key: string]: unknown }, K extends keyof T> = { [_ in K]?: never };
export type Incremental<T> = T | { [P in keyof T]?: P extends ' $fragmentName' | '__typename' ? T[P] : never };
/** All built-in and custom scalars, mapped to their actual values */
export type Scalars = {
  ID: { input: string; output: string; }
  String: { input: string; output: string; }
  Boolean: { input: boolean; output: boolean; }
  Int: { input: number; output: number; }
  Float: { input: number; output: number; }
  /**
   * Implement the DateTime<Utc> scalar
   *
   * The input/output is a string in RFC3339 format.
   */
  DateTime: { input: any; output: any; }
};

export type Album = {
  __typename?: 'Album';
  artworkUrl?: Maybe<Scalars['String']['output']>;
  id: Scalars['Int']['output'];
  title: Scalars['String']['output'];
  year?: Maybe<Scalars['Int']['output']>;
};

export type Artist = {
  __typename?: 'Artist';
  id: Scalars['Int']['output'];
  name: Scalars['String']['output'];
};

export type AuthResponse = {
  __typename?: 'AuthResponse';
  authUrl: Scalars['String']['output'];
  pinId: Scalars['Int']['output'];
};

export type DownloadStatus = {
  __typename?: 'DownloadStatus';
  message: Scalars['String']['output'];
  success: Scalars['Boolean']['output'];
};

export type LibraryScanStatus = {
  __typename?: 'LibraryScanStatus';
  isScanning: Scalars['Boolean']['output'];
  progress?: Maybe<Scalars['Float']['output']>;
  subtitle?: Maybe<Scalars['String']['output']>;
  title?: Maybe<Scalars['String']['output']>;
};

export type MissingTrackInfo = {
  __typename?: 'MissingTrackInfo';
  filePath: Scalars['String']['output'];
  title: Scalars['String']['output'];
  trackId: Scalars['Int']['output'];
};

export type MultiplePlexServersError = {
  __typename?: 'MultiplePlexServersError';
  message: Scalars['String']['output'];
  serverCount: Scalars['Int']['output'];
};

export type Mutation = {
  __typename?: 'Mutation';
  /** Accept a match candidate — links the Spotify track to the local track and dismisses other candidates */
  acceptSpotifyMatchCandidate: Scalars['Boolean']['output'];
  addToWishlist: WishlistItem;
  addTrackToPlaylist: Scalars['Boolean']['output'];
  addYoutubeSubscription: Scalars['Boolean']['output'];
  authenticatePlexServer: AuthResponse;
  completePlexServerAuthentication: PlexServer;
  /** Complete Spotify OAuth by exchanging code for access token */
  completeSpotifyAuth: SpotifyAccount;
  createPlaylist: Playlist;
  createPlexServer: PlexServer;
  deleteSpotifyAccount: Scalars['Boolean']['output'];
  /** Dismiss all pending candidates for a Spotify track (removes from review queue without matching) */
  dismissSpotifyUnmatchedTrack: Scalars['Boolean']['output'];
  downloadSoulseekFile: DownloadStatus;
  /** Initiate Spotify OAuth flow */
  initiateSpotifyAuth: SpotifyAuthResponse;
  /** Manually match a Spotify track to a local track (from library search) */
  manuallyMatchSpotifyTrack: Scalars['Boolean']['output'];
  markYoutubeVideoAsUnwatched: Scalars['Boolean']['output'];
  markYoutubeVideoAsWatched: Scalars['Boolean']['output'];
  matchExistingSpotifyTracksWithLocalTracks: Scalars['Boolean']['output'];
  /** Trigger a refresh/rescan of the music library on a Plex server */
  refreshMusicLibrary: RefreshLibraryResult;
  removeFromWishlist: Scalars['Boolean']['output'];
  removeYoutubeSubscription: Scalars['Boolean']['output'];
  retryWishlistItem: WishlistItem;
  searchSoulseek: Array<SoulSeekSearchResult>;
  /** Sync a database playlist to Plex */
  syncPlaylistToPlex: SyncPlaylistToPlexResult;
  syncSpotifyAccountPlaylistsToDb: Scalars['Boolean']['output'];
  /**
   * Sync a Spotify playlist to a local playlist by matching tracks.
   * Does NOT download — only matches against existing local tracks.
   */
  syncSpotifyPlaylistToLocal: SyncSpotifyPlaylistToLocalResultGql;
  syncSpotifyPlaylistToLocalLibrary: Scalars['Boolean']['output'];
};


export type MutationAcceptSpotifyMatchCandidateArgs = {
  candidateId: Scalars['Int']['input'];
};


export type MutationAddToWishlistArgs = {
  spotifyTrackId: Scalars['String']['input'];
};


export type MutationAddTrackToPlaylistArgs = {
  playlistId: Scalars['Int']['input'];
  trackId: Scalars['Int']['input'];
};


export type MutationAddYoutubeSubscriptionArgs = {
  name: Scalars['String']['input'];
};


export type MutationAuthenticatePlexServerArgs = {
  serverId: Scalars['Int']['input'];
};


export type MutationCompletePlexServerAuthenticationArgs = {
  pinId: Scalars['Int']['input'];
  serverId: Scalars['Int']['input'];
};


export type MutationCompleteSpotifyAuthArgs = {
  authCode: Scalars['String']['input'];
  csrfState: Scalars['String']['input'];
};


export type MutationCreatePlaylistArgs = {
  description?: InputMaybe<Scalars['String']['input']>;
  name: Scalars['String']['input'];
};


export type MutationCreatePlexServerArgs = {
  name: Scalars['String']['input'];
  serverUrl: Scalars['String']['input'];
};


export type MutationDeleteSpotifyAccountArgs = {
  accountId: Scalars['Int']['input'];
};


export type MutationDismissSpotifyUnmatchedTrackArgs = {
  spotifyTrackId: Scalars['String']['input'];
};


export type MutationDownloadSoulseekFileArgs = {
  filename: Scalars['String']['input'];
  size: Scalars['Int']['input'];
  token: Scalars['String']['input'];
  username: Scalars['String']['input'];
};


export type MutationManuallyMatchSpotifyTrackArgs = {
  localTrackId: Scalars['Int']['input'];
  spotifyTrackId: Scalars['String']['input'];
};


export type MutationMarkYoutubeVideoAsUnwatchedArgs = {
  id: Scalars['Int']['input'];
};


export type MutationMarkYoutubeVideoAsWatchedArgs = {
  id: Scalars['Int']['input'];
};


export type MutationRefreshMusicLibraryArgs = {
  plexServerId: Scalars['Int']['input'];
};


export type MutationRemoveFromWishlistArgs = {
  id: Scalars['Int']['input'];
};


export type MutationRemoveYoutubeSubscriptionArgs = {
  id: Scalars['Int']['input'];
};


export type MutationRetryWishlistItemArgs = {
  id: Scalars['Int']['input'];
};


export type MutationSearchSoulseekArgs = {
  albumName?: InputMaybe<Scalars['String']['input']>;
  artists?: InputMaybe<Array<Scalars['String']['input']>>;
  duration?: InputMaybe<Scalars['Int']['input']>;
  trackTitle: Scalars['String']['input'];
};


export type MutationSyncPlaylistToPlexArgs = {
  playlistId: Scalars['Int']['input'];
};


export type MutationSyncSpotifyAccountPlaylistsToDbArgs = {
  accountId: Scalars['Int']['input'];
};


export type MutationSyncSpotifyPlaylistToLocalArgs = {
  localPlaylistName: Scalars['String']['input'];
  spotifyPlaylistId: Scalars['Int']['input'];
};


export type MutationSyncSpotifyPlaylistToLocalLibraryArgs = {
  localPlaylistName: Scalars['String']['input'];
  spotifyAccountId: Scalars['Int']['input'];
  spotifyPlaylistId: Scalars['Int']['input'];
};

export type NoPlexServerError = {
  __typename?: 'NoPlexServerError';
  message: Scalars['String']['output'];
};

export type PaginationInput = {
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
};

export type Playlist = {
  __typename?: 'Playlist';
  createdAt: Scalars['DateTime']['output'];
  description?: Maybe<Scalars['String']['output']>;
  id: Scalars['Int']['output'];
  name: Scalars['String']['output'];
  spotifyPlaylistId?: Maybe<Scalars['Int']['output']>;
  trackCount: Scalars['Int']['output'];
  unmatchedSpotifyTrackCount?: Maybe<Scalars['Int']['output']>;
  updatedAt: Scalars['DateTime']['output'];
};

export type PlaylistsResponse = {
  __typename?: 'PlaylistsResponse';
  page: Scalars['Int']['output'];
  pageSize: Scalars['Int']['output'];
  playlists: Array<Playlist>;
  totalCount: Scalars['Int']['output'];
};

export type PlexPlaylist = {
  __typename?: 'PlexPlaylist';
  duration?: Maybe<Scalars['Int']['output']>;
  leafCount?: Maybe<Scalars['Int']['output']>;
  playlistType: Scalars['String']['output'];
  ratingKey: Scalars['String']['output'];
  title: Scalars['String']['output'];
};

export type PlexPlaylistsResponse = {
  __typename?: 'PlexPlaylistsResponse';
  playlists: Array<PlexPlaylist>;
};

export type PlexServer = {
  __typename?: 'PlexServer';
  createdAt: Scalars['DateTime']['output'];
  hasAccessToken: Scalars['Boolean']['output'];
  id: Scalars['Int']['output'];
  name: Scalars['String']['output'];
  serverUrl: Scalars['String']['output'];
  updatedAt: Scalars['DateTime']['output'];
};

export type PlexTrack = {
  __typename?: 'PlexTrack';
  album?: Maybe<Scalars['String']['output']>;
  artist?: Maybe<Scalars['String']['output']>;
  title: Scalars['String']['output'];
};

export type PlexTracksError = {
  __typename?: 'PlexTracksError';
  message: Scalars['String']['output'];
};

export type PlexTracksResult = MultiplePlexServersError | NoPlexServerError | PlexTracksError | PlexTracksSuccess;

export type PlexTracksSuccess = {
  __typename?: 'PlexTracksSuccess';
  tracks: Array<PlexTrack>;
};

export type Query = {
  __typename?: 'Query';
  /** Get the current scan status for the music library on a Plex server */
  musicLibraryScanStatus: LibraryScanStatus;
  playlist?: Maybe<Playlist>;
  playlistTracks: TracksResponse;
  playlists: PlaylistsResponse;
  plexPlaylists: PlexPlaylistsResponse;
  plexServers: Array<PlexServer>;
  plexTracks: PlexTracksResult;
  /** Search local tracks for manual matching */
  searchLocalTracksForMatching: SearchLocalTracksResponse;
  /** Get all Spotify accounts */
  spotifyAccounts: Array<SpotifyAccount>;
  /** Get matched Spotify tracks with their local track information */
  spotifyMatchedTracks: SpotifyMatchedTracksResponse;
  /** Get sync state for a Spotify playlist */
  spotifyPlaylistSyncState?: Maybe<SpotifyPlaylistSyncState>;
  /** Get playlists for a Spotify account */
  spotifyPlaylists: Array<SpotifyPlaylist>;
  /** Get download failures for a Spotify playlist */
  spotifyTrackDownloadFailures: Array<SpotifyTrackDownloadFailure>;
  /** Get unmatched Spotify tracks with their match candidates for review */
  spotifyUnmatchedTracks: SpotifyUnmatchedTracksResponse;
  tracks: TracksResponse;
  unimportableFiles: UnimportableFilesResponse;
  wishlistItems: WishlistItemsResponse;
  wishlistStats: WishlistStatsGql;
  youtubeSubscriptions: Array<YoutubeSubscription>;
  /**
   * Get all videos from subscribed channels
   * Cache for 3 minutes
   */
  youtubeVideos: Array<Video>;
};


export type QueryMusicLibraryScanStatusArgs = {
  plexServerId: Scalars['Int']['input'];
};


export type QueryPlaylistArgs = {
  id: Scalars['Int']['input'];
};


export type QueryPlaylistTracksArgs = {
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
  playlistId: Scalars['Int']['input'];
};


export type QueryPlaylistsArgs = {
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
  search?: InputMaybe<Scalars['String']['input']>;
  sortBy?: InputMaybe<Scalars['String']['input']>;
  sortOrder?: InputMaybe<Scalars['String']['input']>;
};


export type QuerySearchLocalTracksForMatchingArgs = {
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
  search: Scalars['String']['input'];
};


export type QuerySpotifyMatchedTracksArgs = {
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
  search?: InputMaybe<Scalars['String']['input']>;
};


export type QuerySpotifyPlaylistSyncStateArgs = {
  spotifyPlaylistId: Scalars['Int']['input'];
};


export type QuerySpotifyPlaylistsArgs = {
  accountId: Scalars['Int']['input'];
};


export type QuerySpotifyTrackDownloadFailuresArgs = {
  spotifyPlaylistId: Scalars['Int']['input'];
};


export type QuerySpotifyUnmatchedTracksArgs = {
  hasCandidates?: InputMaybe<Scalars['Boolean']['input']>;
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
  playlistId?: InputMaybe<Scalars['Int']['input']>;
  search?: InputMaybe<Scalars['String']['input']>;
  sortByScore?: InputMaybe<Scalars['Boolean']['input']>;
};


export type QueryTracksArgs = {
  pagination?: InputMaybe<PaginationInput>;
  search?: InputMaybe<TextSearchInput>;
  sort?: InputMaybe<Array<TrackSortInput>>;
};


export type QueryUnimportableFilesArgs = {
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
};


export type QueryWishlistItemsArgs = {
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
  status?: InputMaybe<Scalars['String']['input']>;
};


export type QueryYoutubeVideosArgs = {
  watched?: InputMaybe<Scalars['Boolean']['input']>;
};

export type RefreshLibraryResult = {
  __typename?: 'RefreshLibraryResult';
  message: Scalars['String']['output'];
  sectionId: Scalars['String']['output'];
  success: Scalars['Boolean']['output'];
};

export type SearchLocalTracksResponse = {
  __typename?: 'SearchLocalTracksResponse';
  page: Scalars['Int']['output'];
  pageSize: Scalars['Int']['output'];
  totalCount: Scalars['Int']['output'];
  tracks: Array<Track>;
};

export enum SortOrder {
  Asc = 'ASC',
  Desc = 'DESC'
}

export enum SoulSeekFileAttribute {
  Bitrate = 'BITRATE',
  BitDepth = 'BIT_DEPTH',
  Duration = 'DURATION',
  Encoder = 'ENCODER',
  SampleRate = 'SAMPLE_RATE',
  VariableBitRate = 'VARIABLE_BIT_RATE'
}

export type SoulSeekFileAttributeValue = {
  __typename?: 'SoulSeekFileAttributeValue';
  attribute: SoulSeekFileAttribute;
  value: Scalars['Int']['output'];
};

export type SoulSeekSearchResult = {
  __typename?: 'SoulSeekSearchResult';
  attributes: Array<SoulSeekFileAttributeValue>;
  avgSpeed: Scalars['Float']['output'];
  filename: Scalars['String']['output'];
  queueLength: Scalars['Int']['output'];
  size: Scalars['Int']['output'];
  slotsFree: Scalars['Boolean']['output'];
  token: Scalars['String']['output'];
  username: Scalars['String']['output'];
};

export type SpotifyAccount = {
  __typename?: 'SpotifyAccount';
  createdAt: Scalars['DateTime']['output'];
  displayName?: Maybe<Scalars['String']['output']>;
  id: Scalars['Int']['output'];
  updatedAt: Scalars['DateTime']['output'];
  userId: Scalars['String']['output'];
};

export type SpotifyAuthResponse = {
  __typename?: 'SpotifyAuthResponse';
  redirectUrl: Scalars['String']['output'];
};

export type SpotifyMatchCandidate = {
  __typename?: 'SpotifyMatchCandidate';
  albumSimilarity: Scalars['Float']['output'];
  artistSimilarity: Scalars['Float']['output'];
  confidence: Scalars['String']['output'];
  durationMatch: Scalars['String']['output'];
  id: Scalars['Int']['output'];
  localTrack: Track;
  score: Scalars['Float']['output'];
  titleSimilarity: Scalars['Float']['output'];
  versionMatch: Scalars['String']['output'];
};

export type SpotifyMatchedTrack = {
  __typename?: 'SpotifyMatchedTrack';
  localTrack: Track;
  spotifyAlbum: Scalars['String']['output'];
  spotifyArtists: Array<Scalars['String']['output']>;
  spotifyCreatedAt: Scalars['DateTime']['output'];
  spotifyDuration?: Maybe<Scalars['Int']['output']>;
  spotifyIsrc?: Maybe<Scalars['String']['output']>;
  spotifyTitle: Scalars['String']['output'];
  spotifyTrackId: Scalars['String']['output'];
  spotifyUpdatedAt: Scalars['DateTime']['output'];
};

export type SpotifyMatchedTracksResponse = {
  __typename?: 'SpotifyMatchedTracksResponse';
  matchedTracks: Array<SpotifyMatchedTrack>;
  page: Scalars['Int']['output'];
  pageSize: Scalars['Int']['output'];
  totalCount: Scalars['Int']['output'];
};

export type SpotifyPlaylist = {
  __typename?: 'SpotifyPlaylist';
  createdAt: Scalars['DateTime']['output'];
  description?: Maybe<Scalars['String']['output']>;
  id: Scalars['Int']['output'];
  name: Scalars['String']['output'];
  spotifyId: Scalars['String']['output'];
  trackCount: Scalars['Int']['output'];
  updatedAt: Scalars['DateTime']['output'];
};

export type SpotifyPlaylistSyncState = {
  __typename?: 'SpotifyPlaylistSyncState';
  errorLog?: Maybe<Scalars['String']['output']>;
  id: Scalars['Int']['output'];
  lastSyncAt?: Maybe<Scalars['Int']['output']>;
  localPlaylistId?: Maybe<Scalars['Int']['output']>;
  spotifyPlaylistId: Scalars['Int']['output'];
  syncStatus: Scalars['String']['output'];
  tracksDownloaded: Scalars['Int']['output'];
  tracksFailed: Scalars['Int']['output'];
};

export type SpotifyTrackDownloadFailure = {
  __typename?: 'SpotifyTrackDownloadFailure';
  albumName?: Maybe<Scalars['String']['output']>;
  artistName: Scalars['String']['output'];
  attemptsCount: Scalars['Int']['output'];
  createdAt: Scalars['DateTime']['output'];
  id: Scalars['Int']['output'];
  isrc?: Maybe<Scalars['String']['output']>;
  reason: Scalars['String']['output'];
  spotifyPlaylistId: Scalars['Int']['output'];
  spotifyTrackId: Scalars['String']['output'];
  trackName: Scalars['String']['output'];
  updatedAt: Scalars['DateTime']['output'];
};

export type SpotifyUnmatchedTrack = {
  __typename?: 'SpotifyUnmatchedTrack';
  candidates: Array<SpotifyMatchCandidate>;
  spotifyAlbum: Scalars['String']['output'];
  spotifyArtists: Array<Scalars['String']['output']>;
  spotifyDuration?: Maybe<Scalars['Int']['output']>;
  spotifyIsrc?: Maybe<Scalars['String']['output']>;
  spotifyTitle: Scalars['String']['output'];
  spotifyTrackId: Scalars['String']['output'];
};

export type SpotifyUnmatchedTracksResponse = {
  __typename?: 'SpotifyUnmatchedTracksResponse';
  page: Scalars['Int']['output'];
  pageSize: Scalars['Int']['output'];
  totalCount: Scalars['Int']['output'];
  unmatchedTracks: Array<SpotifyUnmatchedTrack>;
};

export type SyncPlaylistToPlexResult = {
  __typename?: 'SyncPlaylistToPlexResult';
  missingTracks: Array<MissingTrackInfo>;
  tracksAdded: Scalars['Int']['output'];
  tracksRemoved: Scalars['Int']['output'];
  tracksSkipped: Scalars['Int']['output'];
};

export type SyncSpotifyPlaylistToLocalResultGql = {
  __typename?: 'SyncSpotifyPlaylistToLocalResultGql';
  matchedTracks: Scalars['Int']['output'];
  newMatchesFound: Scalars['Int']['output'];
  totalTracks: Scalars['Int']['output'];
  unmatchedTracks: Scalars['Int']['output'];
};

export type TextSearchInput = {
  search?: InputMaybe<Scalars['String']['input']>;
};

export type Track = {
  __typename?: 'Track';
  album: Album;
  artists: Array<Artist>;
  createdAt: Scalars['DateTime']['output'];
  duration?: Maybe<Scalars['Int']['output']>;
  id: Scalars['Int']['output'];
  title: Scalars['String']['output'];
  trackNumber?: Maybe<Scalars['Int']['output']>;
};

export enum TrackSortField {
  CreatedAt = 'CREATED_AT',
  Duration = 'DURATION',
  Id = 'ID',
  Title = 'TITLE',
  TrackNumber = 'TRACK_NUMBER',
  UpdatedAt = 'UPDATED_AT'
}

export type TrackSortInput = {
  field: TrackSortField;
  order: SortOrder;
};

export type TracksResponse = {
  __typename?: 'TracksResponse';
  page: Scalars['Int']['output'];
  pageSize: Scalars['Int']['output'];
  totalCount: Scalars['Int']['output'];
  tracks: Array<Track>;
};

export type UnimportableFile = {
  __typename?: 'UnimportableFile';
  createdAt: Scalars['DateTime']['output'];
  filePath: Scalars['String']['output'];
  id: Scalars['Int']['output'];
  reason: UnimportableReason;
  sha256: Scalars['String']['output'];
};

export type UnimportableFilesResponse = {
  __typename?: 'UnimportableFilesResponse';
  files: Array<UnimportableFile>;
  page: Scalars['Int']['output'];
  pageSize: Scalars['Int']['output'];
  totalCount: Scalars['Int']['output'];
};

export enum UnimportableReason {
  AcoustIdError = 'ACOUST_ID_ERROR',
  AlreadyTriedToImport = 'ALREADY_TRIED_TO_IMPORT',
  ChromaprintError = 'CHROMAPRINT_ERROR',
  DatabaseError = 'DATABASE_ERROR',
  DuplicateTrack = 'DUPLICATE_TRACK',
  FileSystemError = 'FILE_SYSTEM_ERROR',
  HashComputationError = 'HASH_COMPUTATION_ERROR',
  MusicBrainzError = 'MUSIC_BRAINZ_ERROR',
  UnsupportedFileType = 'UNSUPPORTED_FILE_TYPE'
}

export type Video = {
  __typename?: 'Video';
  channelName: Scalars['String']['output'];
  id: Scalars['Int']['output'];
  publishedAt?: Maybe<Scalars['DateTime']['output']>;
  thumbnailUrl: Scalars['String']['output'];
  title: Scalars['String']['output'];
  videoUrl: Scalars['String']['output'];
  watched: Scalars['Boolean']['output'];
};

export type WishlistItem = {
  __typename?: 'WishlistItem';
  attemptsCount: Scalars['Int']['output'];
  createdAt: Scalars['DateTime']['output'];
  errorReason?: Maybe<Scalars['String']['output']>;
  id: Scalars['Int']['output'];
  lastAttemptAt?: Maybe<Scalars['DateTime']['output']>;
  nextRetryAt?: Maybe<Scalars['DateTime']['output']>;
  spotifyTrackId: Scalars['String']['output'];
  status: Scalars['String']['output'];
  trackAlbum: Scalars['String']['output'];
  trackArtists: Array<Scalars['String']['output']>;
  trackTitle: Scalars['String']['output'];
  updatedAt: Scalars['DateTime']['output'];
};

export type WishlistItemsResponse = {
  __typename?: 'WishlistItemsResponse';
  items: Array<WishlistItem>;
  page: Scalars['Int']['output'];
  pageSize: Scalars['Int']['output'];
  totalCount: Scalars['Int']['output'];
};

export type WishlistStatsGql = {
  __typename?: 'WishlistStatsGql';
  completed: Scalars['Int']['output'];
  downloading: Scalars['Int']['output'];
  failed: Scalars['Int']['output'];
  importing: Scalars['Int']['output'];
  pending: Scalars['Int']['output'];
  searching: Scalars['Int']['output'];
};

export type YoutubeSubscription = {
  __typename?: 'YoutubeSubscription';
  createdAt: Scalars['DateTime']['output'];
  id: Scalars['Int']['output'];
  name: Scalars['String']['output'];
  updatedAt: Scalars['DateTime']['output'];
  youtubeId: Scalars['String']['output'];
};

export type PlaylistsForMenuQueryVariables = Exact<{ [key: string]: never; }>;


export type PlaylistsForMenuQuery = { __typename?: 'Query', playlists: { __typename?: 'PlaylistsResponse', playlists: Array<{ __typename?: 'Playlist', id: number, name: string }> } };

export type CreatePlaylistMutationVariables = Exact<{
  name: Scalars['String']['input'];
  description?: InputMaybe<Scalars['String']['input']>;
}>;


export type CreatePlaylistMutation = { __typename?: 'Mutation', createPlaylist: { __typename?: 'Playlist', id: number, name: string, description?: string | null, createdAt: any, updatedAt: any, trackCount: number } };

export type AddTrackToPlaylistMutationVariables = Exact<{
  playlistId: Scalars['Int']['input'];
  trackId: Scalars['Int']['input'];
}>;


export type AddTrackToPlaylistMutation = { __typename?: 'Mutation', addTrackToPlaylist: boolean };

export type SearchSoulseekMutationVariables = Exact<{
  trackTitle: Scalars['String']['input'];
  albumName?: InputMaybe<Scalars['String']['input']>;
  artists?: InputMaybe<Array<Scalars['String']['input']> | Scalars['String']['input']>;
  duration?: InputMaybe<Scalars['Int']['input']>;
}>;


export type SearchSoulseekMutation = { __typename?: 'Mutation', searchSoulseek: Array<{ __typename?: 'SoulSeekSearchResult', username: string, token: string, filename: string, size: number, avgSpeed: number, queueLength: number, slotsFree: boolean, attributes: Array<{ __typename?: 'SoulSeekFileAttributeValue', attribute: SoulSeekFileAttribute, value: number }> }> };

export type PlaylistTracksQueryVariables = Exact<{
  playlistId: Scalars['Int']['input'];
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
}>;


export type PlaylistTracksQuery = { __typename?: 'Query', playlistTracks: { __typename?: 'TracksResponse', totalCount: number, page: number, pageSize: number, tracks: Array<{ __typename?: 'Track', id: number, title: string, trackNumber?: number | null, duration?: number | null, createdAt: any, album: { __typename?: 'Album', id: number, title: string, year?: number | null, artworkUrl?: string | null }, artists: Array<{ __typename?: 'Artist', id: number, name: string }> }> } };

export type PlaylistQueryVariables = Exact<{
  id: Scalars['Int']['input'];
}>;


export type PlaylistQuery = { __typename?: 'Query', playlist?: { __typename?: 'Playlist', id: number, name: string, description?: string | null, trackCount: number } | null };

export type PlaylistsQueryVariables = Exact<{
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
  search?: InputMaybe<Scalars['String']['input']>;
  sortBy?: InputMaybe<Scalars['String']['input']>;
  sortOrder?: InputMaybe<Scalars['String']['input']>;
}>;


export type PlaylistsQuery = { __typename?: 'Query', playlists: { __typename?: 'PlaylistsResponse', totalCount: number, page: number, pageSize: number, playlists: Array<{ __typename?: 'Playlist', id: number, name: string, description?: string | null, spotifyPlaylistId?: number | null, unmatchedSpotifyTrackCount?: number | null, createdAt: any, updatedAt: any, trackCount: number }> } };

export type SyncPlaylistToPlexMutationVariables = Exact<{
  playlistId: Scalars['Int']['input'];
}>;


export type SyncPlaylistToPlexMutation = { __typename?: 'Mutation', syncPlaylistToPlex: { __typename?: 'SyncPlaylistToPlexResult', tracksAdded: number, tracksRemoved: number, tracksSkipped: number, missingTracks: Array<{ __typename?: 'MissingTrackInfo', trackId: number, filePath: string, title: string }> } };

export type CompletePlexServerAuthenticationMutationVariables = Exact<{
  serverId: Scalars['Int']['input'];
  pinId: Scalars['Int']['input'];
}>;


export type CompletePlexServerAuthenticationMutation = { __typename?: 'Mutation', completePlexServerAuthentication: { __typename?: 'PlexServer', id: number, name: string, serverUrl: string, hasAccessToken: boolean, createdAt: any, updatedAt: any } };

export type PlexServersQueryVariables = Exact<{ [key: string]: never; }>;


export type PlexServersQuery = { __typename?: 'Query', plexServers: Array<{ __typename?: 'PlexServer', id: number, name: string, serverUrl: string, hasAccessToken: boolean, createdAt: any, updatedAt: any }> };

export type CreatePlexServerMutationVariables = Exact<{
  name: Scalars['String']['input'];
  serverUrl: Scalars['String']['input'];
}>;


export type CreatePlexServerMutation = { __typename?: 'Mutation', createPlexServer: { __typename?: 'PlexServer', id: number, name: string, serverUrl: string, hasAccessToken: boolean, createdAt: any, updatedAt: any } };

export type AuthenticatePlexServerMutationVariables = Exact<{
  serverId: Scalars['Int']['input'];
}>;


export type AuthenticatePlexServerMutation = { __typename?: 'Mutation', authenticatePlexServer: { __typename?: 'AuthResponse', authUrl: string, pinId: number } };

export type RefreshMusicLibraryMutationVariables = Exact<{
  plexServerId: Scalars['Int']['input'];
}>;


export type RefreshMusicLibraryMutation = { __typename?: 'Mutation', refreshMusicLibrary: { __typename?: 'RefreshLibraryResult', success: boolean, message: string, sectionId: string } };

export type MusicLibraryScanStatusQueryVariables = Exact<{
  plexServerId: Scalars['Int']['input'];
}>;


export type MusicLibraryScanStatusQuery = { __typename?: 'Query', musicLibraryScanStatus: { __typename?: 'LibraryScanStatus', isScanning: boolean, progress?: number | null, title?: string | null, subtitle?: string | null } };

export type PlexTracksQueryVariables = Exact<{ [key: string]: never; }>;


export type PlexTracksQuery = { __typename?: 'Query', plexTracks:
    | { __typename?: 'MultiplePlexServersError', message: string, serverCount: number }
    | { __typename?: 'NoPlexServerError', message: string }
    | { __typename?: 'PlexTracksError', message: string }
    | { __typename?: 'PlexTracksSuccess', tracks: Array<{ __typename?: 'PlexTrack', title: string, album?: string | null, artist?: string | null }> }
   };

export type CompleteSpotifyAuthMutationVariables = Exact<{
  authCode: Scalars['String']['input'];
  csrfState: Scalars['String']['input'];
}>;


export type CompleteSpotifyAuthMutation = { __typename?: 'Mutation', completeSpotifyAuth: { __typename?: 'SpotifyAccount', id: number, userId: string, displayName?: string | null, createdAt: any, updatedAt: any } };

export type InitiateSpotifyAuthMutationVariables = Exact<{ [key: string]: never; }>;


export type InitiateSpotifyAuthMutation = { __typename?: 'Mutation', initiateSpotifyAuth: { __typename?: 'SpotifyAuthResponse', redirectUrl: string } };

export type SpotifyMatchedTracksQueryVariables = Exact<{
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
  search?: InputMaybe<Scalars['String']['input']>;
}>;


export type SpotifyMatchedTracksQuery = { __typename?: 'Query', spotifyMatchedTracks: { __typename?: 'SpotifyMatchedTracksResponse', totalCount: number, page: number, pageSize: number, matchedTracks: Array<{ __typename?: 'SpotifyMatchedTrack', spotifyTrackId: string, spotifyTitle: string, spotifyArtists: Array<string>, spotifyAlbum: string, spotifyIsrc?: string | null, spotifyDuration?: number | null, spotifyCreatedAt: any, spotifyUpdatedAt: any, localTrack: { __typename?: 'Track', id: number, title: string, trackNumber?: number | null, duration?: number | null, createdAt: any, album: { __typename?: 'Album', id: number, title: string, year?: number | null, artworkUrl?: string | null }, artists: Array<{ __typename?: 'Artist', id: number, name: string }> } }> } };

export type SpotifyAccountsForFilterQueryVariables = Exact<{ [key: string]: never; }>;


export type SpotifyAccountsForFilterQuery = { __typename?: 'Query', spotifyAccounts: Array<{ __typename?: 'SpotifyAccount', id: number, displayName?: string | null, userId: string }> };

export type SpotifyPlaylistsForFilterQueryVariables = Exact<{
  accountId: Scalars['Int']['input'];
}>;


export type SpotifyPlaylistsForFilterQuery = { __typename?: 'Query', spotifyPlaylists: Array<{ __typename?: 'SpotifyPlaylist', id: number, name: string, trackCount: number }> };

export type SpotifyUnmatchedTracksQueryVariables = Exact<{
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
  search?: InputMaybe<Scalars['String']['input']>;
  hasCandidates?: InputMaybe<Scalars['Boolean']['input']>;
  sortByScore?: InputMaybe<Scalars['Boolean']['input']>;
  playlistId?: InputMaybe<Scalars['Int']['input']>;
}>;


export type SpotifyUnmatchedTracksQuery = { __typename?: 'Query', spotifyUnmatchedTracks: { __typename?: 'SpotifyUnmatchedTracksResponse', totalCount: number, page: number, pageSize: number, unmatchedTracks: Array<{ __typename?: 'SpotifyUnmatchedTrack', spotifyTrackId: string, spotifyTitle: string, spotifyArtists: Array<string>, spotifyAlbum: string, spotifyIsrc?: string | null, spotifyDuration?: number | null, candidates: Array<{ __typename?: 'SpotifyMatchCandidate', id: number, score: number, confidence: string, titleSimilarity: number, artistSimilarity: number, albumSimilarity: number, durationMatch: string, versionMatch: string, localTrack: { __typename?: 'Track', id: number, title: string, trackNumber?: number | null, duration?: number | null, createdAt: any, album: { __typename?: 'Album', id: number, title: string, year?: number | null, artworkUrl?: string | null }, artists: Array<{ __typename?: 'Artist', id: number, name: string }> } }> }> } };

export type SearchLocalTracksForMatchingQueryVariables = Exact<{
  search: Scalars['String']['input'];
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
}>;


export type SearchLocalTracksForMatchingQuery = { __typename?: 'Query', searchLocalTracksForMatching: { __typename?: 'SearchLocalTracksResponse', totalCount: number, page: number, pageSize: number, tracks: Array<{ __typename?: 'Track', id: number, title: string, trackNumber?: number | null, duration?: number | null, createdAt: any, album: { __typename?: 'Album', id: number, title: string, year?: number | null, artworkUrl?: string | null }, artists: Array<{ __typename?: 'Artist', id: number, name: string }> }> } };

export type AcceptSpotifyMatchCandidateMutationVariables = Exact<{
  candidateId: Scalars['Int']['input'];
}>;


export type AcceptSpotifyMatchCandidateMutation = { __typename?: 'Mutation', acceptSpotifyMatchCandidate: boolean };

export type DismissSpotifyUnmatchedTrackMutationVariables = Exact<{
  spotifyTrackId: Scalars['String']['input'];
}>;


export type DismissSpotifyUnmatchedTrackMutation = { __typename?: 'Mutation', dismissSpotifyUnmatchedTrack: boolean };

export type ManuallyMatchSpotifyTrackMutationVariables = Exact<{
  spotifyTrackId: Scalars['String']['input'];
  localTrackId: Scalars['Int']['input'];
}>;


export type ManuallyMatchSpotifyTrackMutation = { __typename?: 'Mutation', manuallyMatchSpotifyTrack: boolean };

export type AddToWishlistFromUnmatchedMutationVariables = Exact<{
  spotifyTrackId: Scalars['String']['input'];
}>;


export type AddToWishlistFromUnmatchedMutation = { __typename?: 'Mutation', addToWishlist: { __typename?: 'WishlistItem', id: number, status: string } };

export type SpotifyAccountsQueryVariables = Exact<{ [key: string]: never; }>;


export type SpotifyAccountsQuery = { __typename?: 'Query', spotifyAccounts: Array<{ __typename?: 'SpotifyAccount', id: number, userId: string, displayName?: string | null, createdAt: any, updatedAt: any }> };

export type SpotifyPlaylistsQueryVariables = Exact<{
  accountId: Scalars['Int']['input'];
}>;


export type SpotifyPlaylistsQuery = { __typename?: 'Query', spotifyPlaylists: Array<{ __typename?: 'SpotifyPlaylist', id: number, spotifyId: string, name: string, description?: string | null, trackCount: number, createdAt: any, updatedAt: any }> };

export type SyncSpotifyPlaylistsMutationVariables = Exact<{
  accountId: Scalars['Int']['input'];
}>;


export type SyncSpotifyPlaylistsMutation = { __typename?: 'Mutation', syncSpotifyAccountPlaylistsToDb: boolean };

export type MatchTracksMutationVariables = Exact<{ [key: string]: never; }>;


export type MatchTracksMutation = { __typename?: 'Mutation', matchExistingSpotifyTracksWithLocalTracks: boolean };

export type SyncSpotifyPlaylistToLocalMutationVariables = Exact<{
  spotifyPlaylistId: Scalars['Int']['input'];
  localPlaylistName: Scalars['String']['input'];
}>;


export type SyncSpotifyPlaylistToLocalMutation = { __typename?: 'Mutation', syncSpotifyPlaylistToLocal: { __typename?: 'SyncSpotifyPlaylistToLocalResultGql', totalTracks: number, matchedTracks: number, unmatchedTracks: number, newMatchesFound: number } };

export type TracksQueryVariables = Exact<{
  pagination?: InputMaybe<PaginationInput>;
  search?: InputMaybe<TextSearchInput>;
  sort?: InputMaybe<Array<TrackSortInput> | TrackSortInput>;
}>;


export type TracksQuery = { __typename?: 'Query', tracks: { __typename?: 'TracksResponse', totalCount: number, page: number, pageSize: number, tracks: Array<{ __typename?: 'Track', id: number, title: string, trackNumber?: number | null, duration?: number | null, createdAt: any, album: { __typename?: 'Album', id: number, title: string, year?: number | null, artworkUrl?: string | null }, artists: Array<{ __typename?: 'Artist', id: number, name: string }> }> } };

export type UnimportableFilesQueryVariables = Exact<{
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
}>;


export type UnimportableFilesQuery = { __typename?: 'Query', unimportableFiles: { __typename?: 'UnimportableFilesResponse', totalCount: number, page: number, pageSize: number, files: Array<{ __typename?: 'UnimportableFile', id: number, filePath: string, reason: UnimportableReason, createdAt: any, sha256: string }> } };

export type WishlistItemsQueryVariables = Exact<{
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
  status?: InputMaybe<Scalars['String']['input']>;
}>;


export type WishlistItemsQuery = { __typename?: 'Query', wishlistItems: { __typename?: 'WishlistItemsResponse', totalCount: number, page: number, pageSize: number, items: Array<{ __typename?: 'WishlistItem', id: number, spotifyTrackId: string, status: string, errorReason?: string | null, attemptsCount: number, lastAttemptAt?: any | null, nextRetryAt?: any | null, createdAt: any, updatedAt: any, trackTitle: string, trackArtists: Array<string>, trackAlbum: string }> } };

export type WishlistStatsQueryVariables = Exact<{ [key: string]: never; }>;


export type WishlistStatsQuery = { __typename?: 'Query', wishlistStats: { __typename?: 'WishlistStatsGql', pending: number, searching: number, downloading: number, importing: number, completed: number, failed: number } };

export type RemoveFromWishlistMutationVariables = Exact<{
  id: Scalars['Int']['input'];
}>;


export type RemoveFromWishlistMutation = { __typename?: 'Mutation', removeFromWishlist: boolean };

export type RetryWishlistItemMutationVariables = Exact<{
  id: Scalars['Int']['input'];
}>;


export type RetryWishlistItemMutation = { __typename?: 'Mutation', retryWishlistItem: { __typename?: 'WishlistItem', id: number, status: string } };

export type YoutubeSubscriptionsQueryVariables = Exact<{ [key: string]: never; }>;


export type YoutubeSubscriptionsQuery = { __typename?: 'Query', youtubeSubscriptions: Array<{ __typename?: 'YoutubeSubscription', id: number, name: string }> };

export type YoutubeAddSubscriptionMutationVariables = Exact<{
  name: Scalars['String']['input'];
}>;


export type YoutubeAddSubscriptionMutation = { __typename?: 'Mutation', addYoutubeSubscription: boolean };

export type YoutubeRemoveSubscriptionMutationVariables = Exact<{
  id: Scalars['Int']['input'];
}>;


export type YoutubeRemoveSubscriptionMutation = { __typename?: 'Mutation', removeYoutubeSubscription: boolean };

export type YoutubeVideosQueryVariables = Exact<{
  watched?: InputMaybe<Scalars['Boolean']['input']>;
}>;


export type YoutubeVideosQuery = { __typename?: 'Query', youtubeVideos: Array<{ __typename?: 'Video', id: number, title: string, channelName: string, publishedAt?: any | null, thumbnailUrl: string, videoUrl: string, watched: boolean }> };

export type YoutubeMarkVideoAsWatchedMutationVariables = Exact<{
  id: Scalars['Int']['input'];
}>;


export type YoutubeMarkVideoAsWatchedMutation = { __typename?: 'Mutation', markYoutubeVideoAsWatched: boolean };

export type YoutubeMarkVideoAsUnwatchedMutationVariables = Exact<{
  id: Scalars['Int']['input'];
}>;


export type YoutubeMarkVideoAsUnwatchedMutation = { __typename?: 'Mutation', markYoutubeVideoAsUnwatched: boolean };

export class TypedDocumentString<TResult, TVariables>
  extends String
  implements DocumentTypeDecoration<TResult, TVariables>
{
  __apiType?: NonNullable<DocumentTypeDecoration<TResult, TVariables>['__apiType']>;
  private value: string;
  public __meta__?: Record<string, any> | undefined;

  constructor(value: string, __meta__?: Record<string, any> | undefined) {
    super(value);
    this.value = value;
    this.__meta__ = __meta__;
  }

  override toString(): string & DocumentTypeDecoration<TResult, TVariables> {
    return this.value;
  }
}

export const PlaylistsForMenuDocument = new TypedDocumentString(`
    query PlaylistsForMenu {
  playlists(page: 1, pageSize: 100) {
    playlists {
      id
      name
    }
  }
}
    `) as unknown as TypedDocumentString<PlaylistsForMenuQuery, PlaylistsForMenuQueryVariables>;
export const CreatePlaylistDocument = new TypedDocumentString(`
    mutation CreatePlaylist($name: String!, $description: String) {
  createPlaylist(name: $name, description: $description) {
    id
    name
    description
    createdAt
    updatedAt
    trackCount
  }
}
    `) as unknown as TypedDocumentString<CreatePlaylistMutation, CreatePlaylistMutationVariables>;
export const AddTrackToPlaylistDocument = new TypedDocumentString(`
    mutation AddTrackToPlaylist($playlistId: Int!, $trackId: Int!) {
  addTrackToPlaylist(playlistId: $playlistId, trackId: $trackId)
}
    `) as unknown as TypedDocumentString<AddTrackToPlaylistMutation, AddTrackToPlaylistMutationVariables>;
export const SearchSoulseekDocument = new TypedDocumentString(`
    mutation SearchSoulseek($trackTitle: String!, $albumName: String, $artists: [String!], $duration: Int) {
  searchSoulseek(
    trackTitle: $trackTitle
    albumName: $albumName
    artists: $artists
    duration: $duration
  ) {
    username
    token
    filename
    size
    avgSpeed
    queueLength
    slotsFree
    attributes {
      attribute
      value
    }
  }
}
    `) as unknown as TypedDocumentString<SearchSoulseekMutation, SearchSoulseekMutationVariables>;
export const PlaylistTracksDocument = new TypedDocumentString(`
    query PlaylistTracks($playlistId: Int!, $page: Int, $pageSize: Int) {
  playlistTracks(playlistId: $playlistId, page: $page, pageSize: $pageSize) {
    tracks {
      id
      title
      trackNumber
      duration
      createdAt
      album {
        id
        title
        year
        artworkUrl
      }
      artists {
        id
        name
      }
    }
    totalCount
    page
    pageSize
  }
}
    `) as unknown as TypedDocumentString<PlaylistTracksQuery, PlaylistTracksQueryVariables>;
export const PlaylistDocument = new TypedDocumentString(`
    query Playlist($id: Int!) {
  playlist(id: $id) {
    id
    name
    description
    trackCount
  }
}
    `) as unknown as TypedDocumentString<PlaylistQuery, PlaylistQueryVariables>;
export const PlaylistsDocument = new TypedDocumentString(`
    query Playlists($page: Int, $pageSize: Int, $search: String, $sortBy: String, $sortOrder: String) {
  playlists(
    page: $page
    pageSize: $pageSize
    search: $search
    sortBy: $sortBy
    sortOrder: $sortOrder
  ) {
    playlists {
      id
      name
      description
      spotifyPlaylistId
      unmatchedSpotifyTrackCount
      createdAt
      updatedAt
      trackCount
    }
    totalCount
    page
    pageSize
  }
}
    `) as unknown as TypedDocumentString<PlaylistsQuery, PlaylistsQueryVariables>;
export const SyncPlaylistToPlexDocument = new TypedDocumentString(`
    mutation SyncPlaylistToPlex($playlistId: Int!) {
  syncPlaylistToPlex(playlistId: $playlistId) {
    missingTracks {
      trackId
      filePath
      title
    }
    tracksAdded
    tracksRemoved
    tracksSkipped
  }
}
    `) as unknown as TypedDocumentString<SyncPlaylistToPlexMutation, SyncPlaylistToPlexMutationVariables>;
export const CompletePlexServerAuthenticationDocument = new TypedDocumentString(`
    mutation CompletePlexServerAuthentication($serverId: Int!, $pinId: Int!) {
  completePlexServerAuthentication(serverId: $serverId, pinId: $pinId) {
    id
    name
    serverUrl
    hasAccessToken
    createdAt
    updatedAt
  }
}
    `) as unknown as TypedDocumentString<CompletePlexServerAuthenticationMutation, CompletePlexServerAuthenticationMutationVariables>;
export const PlexServersDocument = new TypedDocumentString(`
    query PlexServers {
  plexServers {
    id
    name
    serverUrl
    hasAccessToken
    createdAt
    updatedAt
  }
}
    `) as unknown as TypedDocumentString<PlexServersQuery, PlexServersQueryVariables>;
export const CreatePlexServerDocument = new TypedDocumentString(`
    mutation CreatePlexServer($name: String!, $serverUrl: String!) {
  createPlexServer(name: $name, serverUrl: $serverUrl) {
    id
    name
    serverUrl
    hasAccessToken
    createdAt
    updatedAt
  }
}
    `) as unknown as TypedDocumentString<CreatePlexServerMutation, CreatePlexServerMutationVariables>;
export const AuthenticatePlexServerDocument = new TypedDocumentString(`
    mutation AuthenticatePlexServer($serverId: Int!) {
  authenticatePlexServer(serverId: $serverId) {
    authUrl
    pinId
  }
}
    `) as unknown as TypedDocumentString<AuthenticatePlexServerMutation, AuthenticatePlexServerMutationVariables>;
export const RefreshMusicLibraryDocument = new TypedDocumentString(`
    mutation RefreshMusicLibrary($plexServerId: Int!) {
  refreshMusicLibrary(plexServerId: $plexServerId) {
    success
    message
    sectionId
  }
}
    `) as unknown as TypedDocumentString<RefreshMusicLibraryMutation, RefreshMusicLibraryMutationVariables>;
export const MusicLibraryScanStatusDocument = new TypedDocumentString(`
    query MusicLibraryScanStatus($plexServerId: Int!) {
  musicLibraryScanStatus(plexServerId: $plexServerId) {
    isScanning
    progress
    title
    subtitle
  }
}
    `) as unknown as TypedDocumentString<MusicLibraryScanStatusQuery, MusicLibraryScanStatusQueryVariables>;
export const PlexTracksDocument = new TypedDocumentString(`
    query PlexTracks {
  plexTracks {
    ... on PlexTracksSuccess {
      tracks {
        title
        album
        artist
      }
    }
    ... on NoPlexServerError {
      message
    }
    ... on MultiplePlexServersError {
      message
      serverCount
    }
    ... on PlexTracksError {
      message
    }
  }
}
    `) as unknown as TypedDocumentString<PlexTracksQuery, PlexTracksQueryVariables>;
export const CompleteSpotifyAuthDocument = new TypedDocumentString(`
    mutation CompleteSpotifyAuth($authCode: String!, $csrfState: String!) {
  completeSpotifyAuth(authCode: $authCode, csrfState: $csrfState) {
    id
    userId
    displayName
    createdAt
    updatedAt
  }
}
    `) as unknown as TypedDocumentString<CompleteSpotifyAuthMutation, CompleteSpotifyAuthMutationVariables>;
export const InitiateSpotifyAuthDocument = new TypedDocumentString(`
    mutation InitiateSpotifyAuth {
  initiateSpotifyAuth {
    redirectUrl
  }
}
    `) as unknown as TypedDocumentString<InitiateSpotifyAuthMutation, InitiateSpotifyAuthMutationVariables>;
export const SpotifyMatchedTracksDocument = new TypedDocumentString(`
    query SpotifyMatchedTracks($page: Int, $pageSize: Int, $search: String) {
  spotifyMatchedTracks(page: $page, pageSize: $pageSize, search: $search) {
    matchedTracks {
      spotifyTrackId
      spotifyTitle
      spotifyArtists
      spotifyAlbum
      spotifyIsrc
      spotifyDuration
      spotifyCreatedAt
      spotifyUpdatedAt
      localTrack {
        id
        title
        trackNumber
        duration
        createdAt
        album {
          id
          title
          year
          artworkUrl
        }
        artists {
          id
          name
        }
      }
    }
    totalCount
    page
    pageSize
  }
}
    `) as unknown as TypedDocumentString<SpotifyMatchedTracksQuery, SpotifyMatchedTracksQueryVariables>;
export const SpotifyAccountsForFilterDocument = new TypedDocumentString(`
    query SpotifyAccountsForFilter {
  spotifyAccounts {
    id
    displayName
    userId
  }
}
    `) as unknown as TypedDocumentString<SpotifyAccountsForFilterQuery, SpotifyAccountsForFilterQueryVariables>;
export const SpotifyPlaylistsForFilterDocument = new TypedDocumentString(`
    query SpotifyPlaylistsForFilter($accountId: Int!) {
  spotifyPlaylists(accountId: $accountId) {
    id
    name
    trackCount
  }
}
    `) as unknown as TypedDocumentString<SpotifyPlaylistsForFilterQuery, SpotifyPlaylistsForFilterQueryVariables>;
export const SpotifyUnmatchedTracksDocument = new TypedDocumentString(`
    query SpotifyUnmatchedTracks($page: Int, $pageSize: Int, $search: String, $hasCandidates: Boolean, $sortByScore: Boolean, $playlistId: Int) {
  spotifyUnmatchedTracks(
    page: $page
    pageSize: $pageSize
    search: $search
    hasCandidates: $hasCandidates
    sortByScore: $sortByScore
    playlistId: $playlistId
  ) {
    unmatchedTracks {
      spotifyTrackId
      spotifyTitle
      spotifyArtists
      spotifyAlbum
      spotifyIsrc
      spotifyDuration
      candidates {
        id
        localTrack {
          id
          title
          trackNumber
          duration
          createdAt
          album {
            id
            title
            year
            artworkUrl
          }
          artists {
            id
            name
          }
        }
        score
        confidence
        titleSimilarity
        artistSimilarity
        albumSimilarity
        durationMatch
        versionMatch
      }
    }
    totalCount
    page
    pageSize
  }
}
    `) as unknown as TypedDocumentString<SpotifyUnmatchedTracksQuery, SpotifyUnmatchedTracksQueryVariables>;
export const SearchLocalTracksForMatchingDocument = new TypedDocumentString(`
    query SearchLocalTracksForMatching($search: String!, $page: Int, $pageSize: Int) {
  searchLocalTracksForMatching(search: $search, page: $page, pageSize: $pageSize) {
    tracks {
      id
      title
      trackNumber
      duration
      createdAt
      album {
        id
        title
        year
        artworkUrl
      }
      artists {
        id
        name
      }
    }
    totalCount
    page
    pageSize
  }
}
    `) as unknown as TypedDocumentString<SearchLocalTracksForMatchingQuery, SearchLocalTracksForMatchingQueryVariables>;
export const AcceptSpotifyMatchCandidateDocument = new TypedDocumentString(`
    mutation AcceptSpotifyMatchCandidate($candidateId: Int!) {
  acceptSpotifyMatchCandidate(candidateId: $candidateId)
}
    `) as unknown as TypedDocumentString<AcceptSpotifyMatchCandidateMutation, AcceptSpotifyMatchCandidateMutationVariables>;
export const DismissSpotifyUnmatchedTrackDocument = new TypedDocumentString(`
    mutation DismissSpotifyUnmatchedTrack($spotifyTrackId: String!) {
  dismissSpotifyUnmatchedTrack(spotifyTrackId: $spotifyTrackId)
}
    `) as unknown as TypedDocumentString<DismissSpotifyUnmatchedTrackMutation, DismissSpotifyUnmatchedTrackMutationVariables>;
export const ManuallyMatchSpotifyTrackDocument = new TypedDocumentString(`
    mutation ManuallyMatchSpotifyTrack($spotifyTrackId: String!, $localTrackId: Int!) {
  manuallyMatchSpotifyTrack(
    spotifyTrackId: $spotifyTrackId
    localTrackId: $localTrackId
  )
}
    `) as unknown as TypedDocumentString<ManuallyMatchSpotifyTrackMutation, ManuallyMatchSpotifyTrackMutationVariables>;
export const AddToWishlistFromUnmatchedDocument = new TypedDocumentString(`
    mutation AddToWishlistFromUnmatched($spotifyTrackId: String!) {
  addToWishlist(spotifyTrackId: $spotifyTrackId) {
    id
    status
  }
}
    `) as unknown as TypedDocumentString<AddToWishlistFromUnmatchedMutation, AddToWishlistFromUnmatchedMutationVariables>;
export const SpotifyAccountsDocument = new TypedDocumentString(`
    query SpotifyAccounts {
  spotifyAccounts {
    id
    userId
    displayName
    createdAt
    updatedAt
  }
}
    `) as unknown as TypedDocumentString<SpotifyAccountsQuery, SpotifyAccountsQueryVariables>;
export const SpotifyPlaylistsDocument = new TypedDocumentString(`
    query SpotifyPlaylists($accountId: Int!) {
  spotifyPlaylists(accountId: $accountId) {
    id
    spotifyId
    name
    description
    trackCount
    createdAt
    updatedAt
  }
}
    `) as unknown as TypedDocumentString<SpotifyPlaylistsQuery, SpotifyPlaylistsQueryVariables>;
export const SyncSpotifyPlaylistsDocument = new TypedDocumentString(`
    mutation SyncSpotifyPlaylists($accountId: Int!) {
  syncSpotifyAccountPlaylistsToDb(accountId: $accountId)
}
    `) as unknown as TypedDocumentString<SyncSpotifyPlaylistsMutation, SyncSpotifyPlaylistsMutationVariables>;
export const MatchTracksDocument = new TypedDocumentString(`
    mutation MatchTracks {
  matchExistingSpotifyTracksWithLocalTracks
}
    `) as unknown as TypedDocumentString<MatchTracksMutation, MatchTracksMutationVariables>;
export const SyncSpotifyPlaylistToLocalDocument = new TypedDocumentString(`
    mutation SyncSpotifyPlaylistToLocal($spotifyPlaylistId: Int!, $localPlaylistName: String!) {
  syncSpotifyPlaylistToLocal(
    spotifyPlaylistId: $spotifyPlaylistId
    localPlaylistName: $localPlaylistName
  ) {
    totalTracks
    matchedTracks
    unmatchedTracks
    newMatchesFound
  }
}
    `) as unknown as TypedDocumentString<SyncSpotifyPlaylistToLocalMutation, SyncSpotifyPlaylistToLocalMutationVariables>;
export const TracksDocument = new TypedDocumentString(`
    query Tracks($pagination: PaginationInput, $search: TextSearchInput, $sort: [TrackSortInput!]) {
  tracks(pagination: $pagination, search: $search, sort: $sort) {
    tracks {
      id
      title
      trackNumber
      duration
      createdAt
      album {
        id
        title
        year
        artworkUrl
      }
      artists {
        id
        name
      }
    }
    totalCount
    page
    pageSize
  }
}
    `) as unknown as TypedDocumentString<TracksQuery, TracksQueryVariables>;
export const UnimportableFilesDocument = new TypedDocumentString(`
    query UnimportableFiles($page: Int, $pageSize: Int) {
  unimportableFiles(page: $page, pageSize: $pageSize) {
    files {
      id
      filePath
      reason
      createdAt
      sha256
    }
    totalCount
    page
    pageSize
  }
}
    `) as unknown as TypedDocumentString<UnimportableFilesQuery, UnimportableFilesQueryVariables>;
export const WishlistItemsDocument = new TypedDocumentString(`
    query WishlistItems($page: Int, $pageSize: Int, $status: String) {
  wishlistItems(page: $page, pageSize: $pageSize, status: $status) {
    items {
      id
      spotifyTrackId
      status
      errorReason
      attemptsCount
      lastAttemptAt
      nextRetryAt
      createdAt
      updatedAt
      trackTitle
      trackArtists
      trackAlbum
    }
    totalCount
    page
    pageSize
  }
}
    `) as unknown as TypedDocumentString<WishlistItemsQuery, WishlistItemsQueryVariables>;
export const WishlistStatsDocument = new TypedDocumentString(`
    query WishlistStats {
  wishlistStats {
    pending
    searching
    downloading
    importing
    completed
    failed
  }
}
    `) as unknown as TypedDocumentString<WishlistStatsQuery, WishlistStatsQueryVariables>;
export const RemoveFromWishlistDocument = new TypedDocumentString(`
    mutation RemoveFromWishlist($id: Int!) {
  removeFromWishlist(id: $id)
}
    `) as unknown as TypedDocumentString<RemoveFromWishlistMutation, RemoveFromWishlistMutationVariables>;
export const RetryWishlistItemDocument = new TypedDocumentString(`
    mutation RetryWishlistItem($id: Int!) {
  retryWishlistItem(id: $id) {
    id
    status
  }
}
    `) as unknown as TypedDocumentString<RetryWishlistItemMutation, RetryWishlistItemMutationVariables>;
export const YoutubeSubscriptionsDocument = new TypedDocumentString(`
    query YoutubeSubscriptions {
  youtubeSubscriptions {
    id
    name
  }
}
    `) as unknown as TypedDocumentString<YoutubeSubscriptionsQuery, YoutubeSubscriptionsQueryVariables>;
export const YoutubeAddSubscriptionDocument = new TypedDocumentString(`
    mutation YoutubeAddSubscription($name: String!) {
  addYoutubeSubscription(name: $name)
}
    `) as unknown as TypedDocumentString<YoutubeAddSubscriptionMutation, YoutubeAddSubscriptionMutationVariables>;
export const YoutubeRemoveSubscriptionDocument = new TypedDocumentString(`
    mutation YoutubeRemoveSubscription($id: Int!) {
  removeYoutubeSubscription(id: $id)
}
    `) as unknown as TypedDocumentString<YoutubeRemoveSubscriptionMutation, YoutubeRemoveSubscriptionMutationVariables>;
export const YoutubeVideosDocument = new TypedDocumentString(`
    query YoutubeVideos($watched: Boolean) {
  youtubeVideos(watched: $watched) {
    id
    title
    channelName
    publishedAt
    thumbnailUrl
    videoUrl
    watched
  }
}
    `) as unknown as TypedDocumentString<YoutubeVideosQuery, YoutubeVideosQueryVariables>;
export const YoutubeMarkVideoAsWatchedDocument = new TypedDocumentString(`
    mutation YoutubeMarkVideoAsWatched($id: Int!) {
  markYoutubeVideoAsWatched(id: $id)
}
    `) as unknown as TypedDocumentString<YoutubeMarkVideoAsWatchedMutation, YoutubeMarkVideoAsWatchedMutationVariables>;
export const YoutubeMarkVideoAsUnwatchedDocument = new TypedDocumentString(`
    mutation YoutubeMarkVideoAsUnwatched($id: Int!) {
  markYoutubeVideoAsUnwatched(id: $id)
}
    `) as unknown as TypedDocumentString<YoutubeMarkVideoAsUnwatchedMutation, YoutubeMarkVideoAsUnwatchedMutationVariables>;