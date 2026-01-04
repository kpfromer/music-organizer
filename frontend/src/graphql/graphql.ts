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

export type DownloadStatus = {
  __typename?: 'DownloadStatus';
  message: Scalars['String']['output'];
  success: Scalars['Boolean']['output'];
};

export type Mutation = {
  __typename?: 'Mutation';
  addTrackToPlaylist: Scalars['Boolean']['output'];
  createPlaylist: Playlist;
  downloadSoulseekFile: DownloadStatus;
  searchSoulseek: Array<SoulSeekSearchResult>;
};


export type MutationAddTrackToPlaylistArgs = {
  playlistId: Scalars['Int']['input'];
  trackId: Scalars['Int']['input'];
};


export type MutationCreatePlaylistArgs = {
  description?: InputMaybe<Scalars['String']['input']>;
  name: Scalars['String']['input'];
};


export type MutationDownloadSoulseekFileArgs = {
  filename: Scalars['String']['input'];
  size: Scalars['Int']['input'];
  token: Scalars['String']['input'];
  username: Scalars['String']['input'];
};


export type MutationSearchSoulseekArgs = {
  albumName?: InputMaybe<Scalars['String']['input']>;
  artists?: InputMaybe<Array<Scalars['String']['input']>>;
  duration?: InputMaybe<Scalars['Int']['input']>;
  trackTitle: Scalars['String']['input'];
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
  trackCount: Scalars['Int']['output'];
  updatedAt: Scalars['DateTime']['output'];
};

export type PlaylistsResponse = {
  __typename?: 'PlaylistsResponse';
  page: Scalars['Int']['output'];
  pageSize: Scalars['Int']['output'];
  playlists: Array<Playlist>;
  totalCount: Scalars['Int']['output'];
};

export type Query = {
  __typename?: 'Query';
  errorExample: Scalars['String']['output'];
  howdy: Scalars['String']['output'];
  playlist?: Maybe<Playlist>;
  playlistTracks: TracksResponse;
  playlists: PlaylistsResponse;
  tracks: TracksResponse;
  unimportableFiles: UnimportableFilesResponse;
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


export type QueryTracksArgs = {
  pagination?: InputMaybe<PaginationInput>;
  search?: InputMaybe<TextSearchInput>;
  sort?: InputMaybe<Array<TrackSortInput>>;
};


export type QueryUnimportableFilesArgs = {
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
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
  ChromaprintError = 'CHROMAPRINT_ERROR',
  DatabaseError = 'DATABASE_ERROR',
  DuplicateTrack = 'DUPLICATE_TRACK',
  FileSystemError = 'FILE_SYSTEM_ERROR',
  HashComputationError = 'HASH_COMPUTATION_ERROR',
  MusicBrainzError = 'MUSIC_BRAINZ_ERROR',
  UnsupportedFileType = 'UNSUPPORTED_FILE_TYPE'
}

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

export type DownloadFromSoulseekMutationVariables = Exact<{
  username: Scalars['String']['input'];
  filename: Scalars['String']['input'];
  size: Scalars['Int']['input'];
  token: Scalars['String']['input'];
}>;


export type DownloadFromSoulseekMutation = { __typename?: 'Mutation', downloadSoulseekFile: { __typename?: 'DownloadStatus', success: boolean, message: string } };

export type TestQueryVariables = Exact<{ [key: string]: never; }>;


export type TestQuery = { __typename?: 'Query', howdy: string };

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


export type PlaylistsQuery = { __typename?: 'Query', playlists: { __typename?: 'PlaylistsResponse', totalCount: number, page: number, pageSize: number, playlists: Array<{ __typename?: 'Playlist', id: number, name: string, description?: string | null, createdAt: any, updatedAt: any, trackCount: number }> } };

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
export const DownloadFromSoulseekDocument = new TypedDocumentString(`
    mutation DownloadFromSoulseek($username: String!, $filename: String!, $size: Int!, $token: String!) {
  downloadSoulseekFile(
    username: $username
    filename: $filename
    size: $size
    token: $token
  ) {
    success
    message
  }
}
    `) as unknown as TypedDocumentString<DownloadFromSoulseekMutation, DownloadFromSoulseekMutationVariables>;
export const TestDocument = new TypedDocumentString(`
    query Test {
  howdy
}
    `) as unknown as TypedDocumentString<TestQuery, TestQueryVariables>;
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