/* eslint-disable */
import * as types from './graphql';



/**
 * Map of all GraphQL operations in the project.
 *
 * This map has several performance disadvantages:
 * 1. It is not tree-shakeable, so it will include all operations in the project.
 * 2. It is not minifiable, so the string of a GraphQL query will be multiple times inside the bundle.
 * 3. It does not support dead code elimination, so it will add unused operations.
 *
 * Therefore it is highly recommended to use the babel or swc plugin for production.
 * Learn more about it here: https://the-guild.dev/graphql/codegen/plugins/presets/preset-client#reducing-bundle-size
 */
type Documents = {
    "\n\tmutation SearchSoulseek($trackTitle: String!, $albumName: String, $artists: [String!], $duration: Int) {\n\tsearchSoulseek(\n\t\ttrackTitle: $trackTitle\n\t\talbumName: $albumName\n\t\tartists: $artists\n\t\tduration: $duration\n\t) {\n\t\tusername\n\t\ttoken\n\t\tfilename\n\t\tsize\n\t\tavgSpeed\n\t\tattributes {\n\t\tattribute\n\t\tvalue\n\t\t}\n\t}\n\t}\n": typeof types.SearchSoulseekDocument,
    "\n\tmutation DownloadFromSoulseek($username: String!, $filename: String!, $size: Int!, $token: String!) {\n\tdownloadSoulseekFile(\n\t\tusername: $username\n\t\tfilename: $filename\n\t\tsize: $size\n\t\ttoken: $token\n\t) {\n\t\tsuccess\n\t\tmessage\n\t}\n\t}\n": typeof types.DownloadFromSoulseekDocument,
    "\n  query Test {\n    howdy\n  }\n": typeof types.TestDocument,
    "\n  query Tracks {\n    tracks {\n      id\n      title\n      trackNumber\n      duration\n      createdAt\n      album {\n        id\n        title\n        year\n        artworkUrl\n      }\n      artists {\n        id\n        name\n      }\n    }\n  }\n": typeof types.TracksDocument,
};
const documents: Documents = {
    "\n\tmutation SearchSoulseek($trackTitle: String!, $albumName: String, $artists: [String!], $duration: Int) {\n\tsearchSoulseek(\n\t\ttrackTitle: $trackTitle\n\t\talbumName: $albumName\n\t\tartists: $artists\n\t\tduration: $duration\n\t) {\n\t\tusername\n\t\ttoken\n\t\tfilename\n\t\tsize\n\t\tavgSpeed\n\t\tattributes {\n\t\tattribute\n\t\tvalue\n\t\t}\n\t}\n\t}\n": types.SearchSoulseekDocument,
    "\n\tmutation DownloadFromSoulseek($username: String!, $filename: String!, $size: Int!, $token: String!) {\n\tdownloadSoulseekFile(\n\t\tusername: $username\n\t\tfilename: $filename\n\t\tsize: $size\n\t\ttoken: $token\n\t) {\n\t\tsuccess\n\t\tmessage\n\t}\n\t}\n": types.DownloadFromSoulseekDocument,
    "\n  query Test {\n    howdy\n  }\n": types.TestDocument,
    "\n  query Tracks {\n    tracks {\n      id\n      title\n      trackNumber\n      duration\n      createdAt\n      album {\n        id\n        title\n        year\n        artworkUrl\n      }\n      artists {\n        id\n        name\n      }\n    }\n  }\n": types.TracksDocument,
};

/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n\tmutation SearchSoulseek($trackTitle: String!, $albumName: String, $artists: [String!], $duration: Int) {\n\tsearchSoulseek(\n\t\ttrackTitle: $trackTitle\n\t\talbumName: $albumName\n\t\tartists: $artists\n\t\tduration: $duration\n\t) {\n\t\tusername\n\t\ttoken\n\t\tfilename\n\t\tsize\n\t\tavgSpeed\n\t\tattributes {\n\t\tattribute\n\t\tvalue\n\t\t}\n\t}\n\t}\n"): typeof import('./graphql').SearchSoulseekDocument;
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n\tmutation DownloadFromSoulseek($username: String!, $filename: String!, $size: Int!, $token: String!) {\n\tdownloadSoulseekFile(\n\t\tusername: $username\n\t\tfilename: $filename\n\t\tsize: $size\n\t\ttoken: $token\n\t) {\n\t\tsuccess\n\t\tmessage\n\t}\n\t}\n"): typeof import('./graphql').DownloadFromSoulseekDocument;
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n  query Test {\n    howdy\n  }\n"): typeof import('./graphql').TestDocument;
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n  query Tracks {\n    tracks {\n      id\n      title\n      trackNumber\n      duration\n      createdAt\n      album {\n        id\n        title\n        year\n        artworkUrl\n      }\n      artists {\n        id\n        name\n      }\n    }\n  }\n"): typeof import('./graphql').TracksDocument;


export function graphql(source: string) {
  return (documents as any)[source] ?? {};
}
