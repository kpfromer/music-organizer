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
  downloadSoulseekFile: DownloadStatus;
  searchSoulseek: Array<SoulSeekSearchResult>;
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

export type DownloadStatus = {
  __typename?: 'DownloadStatus';
  message: Scalars['String']['output'];
  success: Scalars['Boolean']['output'];
};

export type Mutation = {
  __typename?: 'Mutation';
  downloadSoulseekFile: DownloadStatus;
  searchSoulseek: Array<SoulSeekSearchResult>;
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

export type Query = {
  __typename?: 'Query';
  errorExample: Scalars['String']['output'];
  howdy: Scalars['String']['output'];
  tracks: Array<Track>;
};

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

export type TestQueryVariables = Exact<{ [key: string]: never; }>;


export type TestQuery = { __typename?: 'Query', howdy: string };

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

export const TestDocument = new TypedDocumentString(`
    query Test {
  howdy
}
    `) as unknown as TypedDocumentString<TestQuery, TestQueryVariables>;