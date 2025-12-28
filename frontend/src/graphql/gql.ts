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
    "\n\tmutation SearchSoulseek($trackTitle: String!, $albumName: String, $artists: [String!], $duration: Int) {\n\t\tsearchSoulseek(\n\t\t\ttrackTitle: $trackTitle\n\t\t\talbumName: $albumName\n\t\t\tartists: $artists\n\t\t\tduration: $duration\n\t\t) {\n\t\t\tusername\n\t\t\ttoken\n\t\t\tfilename\n\t\t\tsize\n\t\t\tavgSpeed\n\t\t\tqueueLength\n\t\t\tslotsFree\n\t\t\tattributes {\n\t\t\t\tattribute\n\t\t\t\tvalue\n\t\t\t}\n\t\t}\n\t}\n": typeof types.SearchSoulseekDocument,
    "\n\tmutation DownloadFromSoulseek($username: String!, $filename: String!, $size: Int!, $token: String!) {\n\t\tdownloadSoulseekFile(\n\t\t\tusername: $username\n\t\t\tfilename: $filename\n\t\t\tsize: $size\n\t\t\ttoken: $token\n\t\t) {\n\t\t\tsuccess\n\t\t\tmessage\n\t\t}\n\t}\n": typeof types.DownloadFromSoulseekDocument,
    "\n  query Test {\n    howdy\n  }\n": typeof types.TestDocument,
    "\n  query Tracks($page: Int, $pageSize: Int) {\n    tracks(page: $page, pageSize: $pageSize) {\n      tracks {\n        id\n        title\n        trackNumber\n        duration\n        createdAt\n        album {\n          id\n          title\n          year\n          artworkUrl\n        }\n        artists {\n          id\n          name\n        }\n      }\n      totalCount\n      page\n      pageSize\n    }\n  }\n": typeof types.TracksDocument,
    "\n  query UnimportableFiles($page: Int, $pageSize: Int) {\n    unimportableFiles(page: $page, pageSize: $pageSize) {\n      files {\n        id\n        filePath\n        reason\n        createdAt\n        sha256\n      }\n      totalCount\n      page\n      pageSize\n    }\n  }\n": typeof types.UnimportableFilesDocument,
};
const documents: Documents = {
    "\n\tmutation SearchSoulseek($trackTitle: String!, $albumName: String, $artists: [String!], $duration: Int) {\n\t\tsearchSoulseek(\n\t\t\ttrackTitle: $trackTitle\n\t\t\talbumName: $albumName\n\t\t\tartists: $artists\n\t\t\tduration: $duration\n\t\t) {\n\t\t\tusername\n\t\t\ttoken\n\t\t\tfilename\n\t\t\tsize\n\t\t\tavgSpeed\n\t\t\tqueueLength\n\t\t\tslotsFree\n\t\t\tattributes {\n\t\t\t\tattribute\n\t\t\t\tvalue\n\t\t\t}\n\t\t}\n\t}\n": types.SearchSoulseekDocument,
    "\n\tmutation DownloadFromSoulseek($username: String!, $filename: String!, $size: Int!, $token: String!) {\n\t\tdownloadSoulseekFile(\n\t\t\tusername: $username\n\t\t\tfilename: $filename\n\t\t\tsize: $size\n\t\t\ttoken: $token\n\t\t) {\n\t\t\tsuccess\n\t\t\tmessage\n\t\t}\n\t}\n": types.DownloadFromSoulseekDocument,
    "\n  query Test {\n    howdy\n  }\n": types.TestDocument,
    "\n  query Tracks($page: Int, $pageSize: Int) {\n    tracks(page: $page, pageSize: $pageSize) {\n      tracks {\n        id\n        title\n        trackNumber\n        duration\n        createdAt\n        album {\n          id\n          title\n          year\n          artworkUrl\n        }\n        artists {\n          id\n          name\n        }\n      }\n      totalCount\n      page\n      pageSize\n    }\n  }\n": types.TracksDocument,
    "\n  query UnimportableFiles($page: Int, $pageSize: Int) {\n    unimportableFiles(page: $page, pageSize: $pageSize) {\n      files {\n        id\n        filePath\n        reason\n        createdAt\n        sha256\n      }\n      totalCount\n      page\n      pageSize\n    }\n  }\n": types.UnimportableFilesDocument,
};

/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n\tmutation SearchSoulseek($trackTitle: String!, $albumName: String, $artists: [String!], $duration: Int) {\n\t\tsearchSoulseek(\n\t\t\ttrackTitle: $trackTitle\n\t\t\talbumName: $albumName\n\t\t\tartists: $artists\n\t\t\tduration: $duration\n\t\t) {\n\t\t\tusername\n\t\t\ttoken\n\t\t\tfilename\n\t\t\tsize\n\t\t\tavgSpeed\n\t\t\tqueueLength\n\t\t\tslotsFree\n\t\t\tattributes {\n\t\t\t\tattribute\n\t\t\t\tvalue\n\t\t\t}\n\t\t}\n\t}\n"): typeof import('./graphql').SearchSoulseekDocument;
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n\tmutation DownloadFromSoulseek($username: String!, $filename: String!, $size: Int!, $token: String!) {\n\t\tdownloadSoulseekFile(\n\t\t\tusername: $username\n\t\t\tfilename: $filename\n\t\t\tsize: $size\n\t\t\ttoken: $token\n\t\t) {\n\t\t\tsuccess\n\t\t\tmessage\n\t\t}\n\t}\n"): typeof import('./graphql').DownloadFromSoulseekDocument;
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n  query Test {\n    howdy\n  }\n"): typeof import('./graphql').TestDocument;
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n  query Tracks($page: Int, $pageSize: Int) {\n    tracks(page: $page, pageSize: $pageSize) {\n      tracks {\n        id\n        title\n        trackNumber\n        duration\n        createdAt\n        album {\n          id\n          title\n          year\n          artworkUrl\n        }\n        artists {\n          id\n          name\n        }\n      }\n      totalCount\n      page\n      pageSize\n    }\n  }\n"): typeof import('./graphql').TracksDocument;
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n  query UnimportableFiles($page: Int, $pageSize: Int) {\n    unimportableFiles(page: $page, pageSize: $pageSize) {\n      files {\n        id\n        filePath\n        reason\n        createdAt\n        sha256\n      }\n      totalCount\n      page\n      pageSize\n    }\n  }\n"): typeof import('./graphql').UnimportableFilesDocument;


export function graphql(source: string) {
  return (documents as any)[source] ?? {};
}
