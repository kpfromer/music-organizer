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
    "\n  query PlaylistsForMenu {\n    playlists(page: 1, pageSize: 100) {\n      playlists {\n        id\n        name\n      }\n    }\n  }\n": typeof types.PlaylistsForMenuDocument,
    "\n  mutation CreatePlaylist($name: String!, $description: String) {\n    createPlaylist(name: $name, description: $description) {\n      id\n      name\n      description\n      createdAt\n      updatedAt\n      trackCount\n    }\n  }\n": typeof types.CreatePlaylistDocument,
    "\n  mutation AddTrackToPlaylist($playlistId: Int!, $trackId: Int!) {\n    addTrackToPlaylist(playlistId: $playlistId, trackId: $trackId)\n  }\n": typeof types.AddTrackToPlaylistDocument,
    "\n\tmutation SearchSoulseek($trackTitle: String!, $albumName: String, $artists: [String!], $duration: Int) {\n\t\tsearchSoulseek(\n\t\t\ttrackTitle: $trackTitle\n\t\t\talbumName: $albumName\n\t\t\tartists: $artists\n\t\t\tduration: $duration\n\t\t) {\n\t\t\tusername\n\t\t\ttoken\n\t\t\tfilename\n\t\t\tsize\n\t\t\tavgSpeed\n\t\t\tqueueLength\n\t\t\tslotsFree\n\t\t\tattributes {\n\t\t\t\tattribute\n\t\t\t\tvalue\n\t\t\t}\n\t\t}\n\t}\n": typeof types.SearchSoulseekDocument,
    "\n  query Test {\n    howdy\n  }\n": typeof types.TestDocument,
    "\n\tquery PlaylistTracks($playlistId: Int!, $page: Int, $pageSize: Int) {\n\t\tplaylistTracks(playlistId: $playlistId, page: $page, pageSize: $pageSize) {\n\t\t\ttracks {\n\t\t\t\tid\n\t\t\t\ttitle\n\t\t\t\ttrackNumber\n\t\t\t\tduration\n\t\t\t\tcreatedAt\n\t\t\t\talbum {\n\t\t\t\t\tid\n\t\t\t\t\ttitle\n\t\t\t\t\tyear\n\t\t\t\t\tartworkUrl\n\t\t\t\t}\n\t\t\t\tartists {\n\t\t\t\t\tid\n\t\t\t\t\tname\n\t\t\t\t}\n\t\t\t}\n\t\t\ttotalCount\n\t\t\tpage\n\t\t\tpageSize\n\t\t}\n\t}\n": typeof types.PlaylistTracksDocument,
    "\n\tquery Playlist($id: Int!) {\n\t\tplaylist(id: $id) {\n\t\t\tid\n\t\t\tname\n\t\t\tdescription\n\t\t\ttrackCount\n\t\t}\n\t}\n": typeof types.PlaylistDocument,
    "\n  query Playlists(\n    $page: Int\n    $pageSize: Int\n    $search: String\n    $sortBy: String\n    $sortOrder: String\n  ) {\n    playlists(\n      page: $page\n      pageSize: $pageSize\n      search: $search\n      sortBy: $sortBy\n      sortOrder: $sortOrder\n    ) {\n      playlists {\n        id\n        name\n        description\n        createdAt\n        updatedAt\n        trackCount\n      }\n      totalCount\n      page\n      pageSize\n    }\n  }\n": typeof types.PlaylistsDocument,
    "\n  mutation SyncPlaylistToPlex($playlistId: Int!) {\n    syncPlaylistToPlex(playlistId: $playlistId) {\n      missingTracks {\n        trackId\n        filePath\n        title\n      }\n      tracksAdded\n      tracksRemoved\n      tracksSkipped\n    }\n  }\n": typeof types.SyncPlaylistToPlexDocument,
    "\n  mutation CompletePlexServerAuthentication($serverId: Int!, $pinId: Int!) {\n    completePlexServerAuthentication(serverId: $serverId, pinId: $pinId) {\n      id\n      name\n      serverUrl\n      hasAccessToken\n      createdAt\n      updatedAt\n    }\n  }\n": typeof types.CompletePlexServerAuthenticationDocument,
    "\n  query PlexServers {\n    plexServers {\n      id\n      name\n      serverUrl\n      hasAccessToken\n      createdAt\n      updatedAt\n    }\n  }\n": typeof types.PlexServersDocument,
    "\n  mutation CreatePlexServer($name: String!, $serverUrl: String!) {\n    createPlexServer(name: $name, serverUrl: $serverUrl) {\n      id\n      name\n      serverUrl\n      hasAccessToken\n      createdAt\n      updatedAt\n    }\n  }\n": typeof types.CreatePlexServerDocument,
    "\n  mutation AuthenticatePlexServer($serverId: Int!) {\n    authenticatePlexServer(serverId: $serverId) {\n      authUrl\n      pinId\n    }\n  }\n": typeof types.AuthenticatePlexServerDocument,
    "\n  mutation RefreshMusicLibrary($plexServerId: Int!) {\n    refreshMusicLibrary(plexServerId: $plexServerId) {\n      success\n      message\n      sectionId\n    }\n  }\n": typeof types.RefreshMusicLibraryDocument,
    "\n  query MusicLibraryScanStatus($plexServerId: Int!) {\n    musicLibraryScanStatus(plexServerId: $plexServerId) {\n      isScanning\n      progress\n      title\n      subtitle\n    }\n  }\n": typeof types.MusicLibraryScanStatusDocument,
    "\n  query PlexTracks {\n    plexTracks {\n      ... on PlexTracksSuccess {\n        tracks {\n          title\n          album\n          artist\n        }\n      }\n      ... on NoPlexServerError {\n        message\n      }\n      ... on MultiplePlexServersError {\n        message\n        serverCount\n      }\n      ... on PlexTracksError {\n        message\n      }\n    }\n  }\n": typeof types.PlexTracksDocument,
    "\n  mutation CompleteSpotifyAuth($authCode: String!, $csrfState: String!) {\n    completeSpotifyAuth(authCode: $authCode, csrfState: $csrfState) {\n      id\n      userId\n      displayName\n      createdAt\n      updatedAt\n    }\n  }\n": typeof types.CompleteSpotifyAuthDocument,
    "\n  mutation InitiateSpotifyAuth {\n    initiateSpotifyAuth {\n      redirectUrl\n    }\n  }\n": typeof types.InitiateSpotifyAuthDocument,
    "\n  query SpotifyMatchedTracks($page: Int, $pageSize: Int, $search: String) {\n    spotifyMatchedTracks(page: $page, pageSize: $pageSize, search: $search) {\n      matchedTracks {\n        spotifyTrackId\n        spotifyTitle\n        spotifyArtists\n        spotifyAlbum\n        spotifyIsrc\n        spotifyDuration\n        spotifyCreatedAt\n        spotifyUpdatedAt\n        localTrack {\n          id\n          title\n          trackNumber\n          duration\n          createdAt\n          album {\n            id\n            title\n            year\n            artworkUrl\n          }\n          artists {\n            id\n            name\n          }\n        }\n      }\n      totalCount\n      page\n      pageSize\n    }\n  }\n": typeof types.SpotifyMatchedTracksDocument,
    "\n  query SpotifyAccounts {\n    spotifyAccounts {\n      id\n      userId\n      displayName\n      createdAt\n      updatedAt\n    }\n  }\n": typeof types.SpotifyAccountsDocument,
    "\n  query SpotifyPlaylists($accountId: Int!) {\n    spotifyPlaylists(accountId: $accountId) {\n      id\n      spotifyId\n      name\n      description\n      trackCount\n      createdAt\n      updatedAt\n    }\n  }\n": typeof types.SpotifyPlaylistsDocument,
    "\n  query SpotifyPlaylistSyncState($spotifyPlaylistId: Int!) {\n    spotifyPlaylistSyncState(spotifyPlaylistId: $spotifyPlaylistId) {\n      id\n      spotifyPlaylistId\n      localPlaylistId\n      lastSyncAt\n      syncStatus\n      tracksDownloaded\n      tracksFailed\n      errorLog\n    }\n  }\n": typeof types.SpotifyPlaylistSyncStateDocument,
    "\n  query SpotifyTrackDownloadFailures($spotifyPlaylistId: Int!) {\n    spotifyTrackDownloadFailures(spotifyPlaylistId: $spotifyPlaylistId) {\n      id\n      spotifyPlaylistId\n      spotifyTrackId\n      trackName\n      artistName\n      albumName\n      isrc\n      reason\n      attemptsCount\n      createdAt\n      updatedAt\n    }\n  }\n": typeof types.SpotifyTrackDownloadFailuresDocument,
    "\n  mutation SyncSpotifyPlaylists($accountId: Int!) {\n    syncSpotifyAccountPlaylistsToDb(accountId: $accountId)\n  }\n": typeof types.SyncSpotifyPlaylistsDocument,
    "\n  mutation MatchTracks {\n    matchExistingSpotifyTracksWithLocalTracks\n  }\n": typeof types.MatchTracksDocument,
    "\n  mutation SyncPlaylistToLocalLibrary(\n    $spotifyAccountId: Int!\n    $spotifyPlaylistId: Int!\n    $localPlaylistName: String!\n  ) {\n    syncSpotifyPlaylistToLocalLibrary(\n      spotifyAccountId: $spotifyAccountId\n      spotifyPlaylistId: $spotifyPlaylistId\n      localPlaylistName: $localPlaylistName\n    )\n  }\n": typeof types.SyncPlaylistToLocalLibraryDocument,
    "\n  query Tracks(\n    $pagination: PaginationInput\n    $search: TextSearchInput\n    $sort: [TrackSortInput!]\n  ) {\n    tracks(pagination: $pagination, search: $search, sort: $sort) {\n      tracks {\n        id\n        title\n        trackNumber\n        duration\n        createdAt\n        album {\n          id\n          title\n          year\n          artworkUrl\n        }\n        artists {\n          id\n          name\n        }\n      }\n      totalCount\n      page\n      pageSize\n    }\n  }\n": typeof types.TracksDocument,
    "\n  query UnimportableFiles($page: Int, $pageSize: Int) {\n    unimportableFiles(page: $page, pageSize: $pageSize) {\n      files {\n        id\n        filePath\n        reason\n        createdAt\n        sha256\n      }\n      totalCount\n      page\n      pageSize\n    }\n  }\n": typeof types.UnimportableFilesDocument,
};
const documents: Documents = {
    "\n  query PlaylistsForMenu {\n    playlists(page: 1, pageSize: 100) {\n      playlists {\n        id\n        name\n      }\n    }\n  }\n": types.PlaylistsForMenuDocument,
    "\n  mutation CreatePlaylist($name: String!, $description: String) {\n    createPlaylist(name: $name, description: $description) {\n      id\n      name\n      description\n      createdAt\n      updatedAt\n      trackCount\n    }\n  }\n": types.CreatePlaylistDocument,
    "\n  mutation AddTrackToPlaylist($playlistId: Int!, $trackId: Int!) {\n    addTrackToPlaylist(playlistId: $playlistId, trackId: $trackId)\n  }\n": types.AddTrackToPlaylistDocument,
    "\n\tmutation SearchSoulseek($trackTitle: String!, $albumName: String, $artists: [String!], $duration: Int) {\n\t\tsearchSoulseek(\n\t\t\ttrackTitle: $trackTitle\n\t\t\talbumName: $albumName\n\t\t\tartists: $artists\n\t\t\tduration: $duration\n\t\t) {\n\t\t\tusername\n\t\t\ttoken\n\t\t\tfilename\n\t\t\tsize\n\t\t\tavgSpeed\n\t\t\tqueueLength\n\t\t\tslotsFree\n\t\t\tattributes {\n\t\t\t\tattribute\n\t\t\t\tvalue\n\t\t\t}\n\t\t}\n\t}\n": types.SearchSoulseekDocument,
    "\n  query Test {\n    howdy\n  }\n": types.TestDocument,
    "\n\tquery PlaylistTracks($playlistId: Int!, $page: Int, $pageSize: Int) {\n\t\tplaylistTracks(playlistId: $playlistId, page: $page, pageSize: $pageSize) {\n\t\t\ttracks {\n\t\t\t\tid\n\t\t\t\ttitle\n\t\t\t\ttrackNumber\n\t\t\t\tduration\n\t\t\t\tcreatedAt\n\t\t\t\talbum {\n\t\t\t\t\tid\n\t\t\t\t\ttitle\n\t\t\t\t\tyear\n\t\t\t\t\tartworkUrl\n\t\t\t\t}\n\t\t\t\tartists {\n\t\t\t\t\tid\n\t\t\t\t\tname\n\t\t\t\t}\n\t\t\t}\n\t\t\ttotalCount\n\t\t\tpage\n\t\t\tpageSize\n\t\t}\n\t}\n": types.PlaylistTracksDocument,
    "\n\tquery Playlist($id: Int!) {\n\t\tplaylist(id: $id) {\n\t\t\tid\n\t\t\tname\n\t\t\tdescription\n\t\t\ttrackCount\n\t\t}\n\t}\n": types.PlaylistDocument,
    "\n  query Playlists(\n    $page: Int\n    $pageSize: Int\n    $search: String\n    $sortBy: String\n    $sortOrder: String\n  ) {\n    playlists(\n      page: $page\n      pageSize: $pageSize\n      search: $search\n      sortBy: $sortBy\n      sortOrder: $sortOrder\n    ) {\n      playlists {\n        id\n        name\n        description\n        createdAt\n        updatedAt\n        trackCount\n      }\n      totalCount\n      page\n      pageSize\n    }\n  }\n": types.PlaylistsDocument,
    "\n  mutation SyncPlaylistToPlex($playlistId: Int!) {\n    syncPlaylistToPlex(playlistId: $playlistId) {\n      missingTracks {\n        trackId\n        filePath\n        title\n      }\n      tracksAdded\n      tracksRemoved\n      tracksSkipped\n    }\n  }\n": types.SyncPlaylistToPlexDocument,
    "\n  mutation CompletePlexServerAuthentication($serverId: Int!, $pinId: Int!) {\n    completePlexServerAuthentication(serverId: $serverId, pinId: $pinId) {\n      id\n      name\n      serverUrl\n      hasAccessToken\n      createdAt\n      updatedAt\n    }\n  }\n": types.CompletePlexServerAuthenticationDocument,
    "\n  query PlexServers {\n    plexServers {\n      id\n      name\n      serverUrl\n      hasAccessToken\n      createdAt\n      updatedAt\n    }\n  }\n": types.PlexServersDocument,
    "\n  mutation CreatePlexServer($name: String!, $serverUrl: String!) {\n    createPlexServer(name: $name, serverUrl: $serverUrl) {\n      id\n      name\n      serverUrl\n      hasAccessToken\n      createdAt\n      updatedAt\n    }\n  }\n": types.CreatePlexServerDocument,
    "\n  mutation AuthenticatePlexServer($serverId: Int!) {\n    authenticatePlexServer(serverId: $serverId) {\n      authUrl\n      pinId\n    }\n  }\n": types.AuthenticatePlexServerDocument,
    "\n  mutation RefreshMusicLibrary($plexServerId: Int!) {\n    refreshMusicLibrary(plexServerId: $plexServerId) {\n      success\n      message\n      sectionId\n    }\n  }\n": types.RefreshMusicLibraryDocument,
    "\n  query MusicLibraryScanStatus($plexServerId: Int!) {\n    musicLibraryScanStatus(plexServerId: $plexServerId) {\n      isScanning\n      progress\n      title\n      subtitle\n    }\n  }\n": types.MusicLibraryScanStatusDocument,
    "\n  query PlexTracks {\n    plexTracks {\n      ... on PlexTracksSuccess {\n        tracks {\n          title\n          album\n          artist\n        }\n      }\n      ... on NoPlexServerError {\n        message\n      }\n      ... on MultiplePlexServersError {\n        message\n        serverCount\n      }\n      ... on PlexTracksError {\n        message\n      }\n    }\n  }\n": types.PlexTracksDocument,
    "\n  mutation CompleteSpotifyAuth($authCode: String!, $csrfState: String!) {\n    completeSpotifyAuth(authCode: $authCode, csrfState: $csrfState) {\n      id\n      userId\n      displayName\n      createdAt\n      updatedAt\n    }\n  }\n": types.CompleteSpotifyAuthDocument,
    "\n  mutation InitiateSpotifyAuth {\n    initiateSpotifyAuth {\n      redirectUrl\n    }\n  }\n": types.InitiateSpotifyAuthDocument,
    "\n  query SpotifyMatchedTracks($page: Int, $pageSize: Int, $search: String) {\n    spotifyMatchedTracks(page: $page, pageSize: $pageSize, search: $search) {\n      matchedTracks {\n        spotifyTrackId\n        spotifyTitle\n        spotifyArtists\n        spotifyAlbum\n        spotifyIsrc\n        spotifyDuration\n        spotifyCreatedAt\n        spotifyUpdatedAt\n        localTrack {\n          id\n          title\n          trackNumber\n          duration\n          createdAt\n          album {\n            id\n            title\n            year\n            artworkUrl\n          }\n          artists {\n            id\n            name\n          }\n        }\n      }\n      totalCount\n      page\n      pageSize\n    }\n  }\n": types.SpotifyMatchedTracksDocument,
    "\n  query SpotifyAccounts {\n    spotifyAccounts {\n      id\n      userId\n      displayName\n      createdAt\n      updatedAt\n    }\n  }\n": types.SpotifyAccountsDocument,
    "\n  query SpotifyPlaylists($accountId: Int!) {\n    spotifyPlaylists(accountId: $accountId) {\n      id\n      spotifyId\n      name\n      description\n      trackCount\n      createdAt\n      updatedAt\n    }\n  }\n": types.SpotifyPlaylistsDocument,
    "\n  query SpotifyPlaylistSyncState($spotifyPlaylistId: Int!) {\n    spotifyPlaylistSyncState(spotifyPlaylistId: $spotifyPlaylistId) {\n      id\n      spotifyPlaylistId\n      localPlaylistId\n      lastSyncAt\n      syncStatus\n      tracksDownloaded\n      tracksFailed\n      errorLog\n    }\n  }\n": types.SpotifyPlaylistSyncStateDocument,
    "\n  query SpotifyTrackDownloadFailures($spotifyPlaylistId: Int!) {\n    spotifyTrackDownloadFailures(spotifyPlaylistId: $spotifyPlaylistId) {\n      id\n      spotifyPlaylistId\n      spotifyTrackId\n      trackName\n      artistName\n      albumName\n      isrc\n      reason\n      attemptsCount\n      createdAt\n      updatedAt\n    }\n  }\n": types.SpotifyTrackDownloadFailuresDocument,
    "\n  mutation SyncSpotifyPlaylists($accountId: Int!) {\n    syncSpotifyAccountPlaylistsToDb(accountId: $accountId)\n  }\n": types.SyncSpotifyPlaylistsDocument,
    "\n  mutation MatchTracks {\n    matchExistingSpotifyTracksWithLocalTracks\n  }\n": types.MatchTracksDocument,
    "\n  mutation SyncPlaylistToLocalLibrary(\n    $spotifyAccountId: Int!\n    $spotifyPlaylistId: Int!\n    $localPlaylistName: String!\n  ) {\n    syncSpotifyPlaylistToLocalLibrary(\n      spotifyAccountId: $spotifyAccountId\n      spotifyPlaylistId: $spotifyPlaylistId\n      localPlaylistName: $localPlaylistName\n    )\n  }\n": types.SyncPlaylistToLocalLibraryDocument,
    "\n  query Tracks(\n    $pagination: PaginationInput\n    $search: TextSearchInput\n    $sort: [TrackSortInput!]\n  ) {\n    tracks(pagination: $pagination, search: $search, sort: $sort) {\n      tracks {\n        id\n        title\n        trackNumber\n        duration\n        createdAt\n        album {\n          id\n          title\n          year\n          artworkUrl\n        }\n        artists {\n          id\n          name\n        }\n      }\n      totalCount\n      page\n      pageSize\n    }\n  }\n": types.TracksDocument,
    "\n  query UnimportableFiles($page: Int, $pageSize: Int) {\n    unimportableFiles(page: $page, pageSize: $pageSize) {\n      files {\n        id\n        filePath\n        reason\n        createdAt\n        sha256\n      }\n      totalCount\n      page\n      pageSize\n    }\n  }\n": types.UnimportableFilesDocument,
};

/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n  query PlaylistsForMenu {\n    playlists(page: 1, pageSize: 100) {\n      playlists {\n        id\n        name\n      }\n    }\n  }\n"): typeof import('./graphql').PlaylistsForMenuDocument;
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n  mutation CreatePlaylist($name: String!, $description: String) {\n    createPlaylist(name: $name, description: $description) {\n      id\n      name\n      description\n      createdAt\n      updatedAt\n      trackCount\n    }\n  }\n"): typeof import('./graphql').CreatePlaylistDocument;
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n  mutation AddTrackToPlaylist($playlistId: Int!, $trackId: Int!) {\n    addTrackToPlaylist(playlistId: $playlistId, trackId: $trackId)\n  }\n"): typeof import('./graphql').AddTrackToPlaylistDocument;
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n\tmutation SearchSoulseek($trackTitle: String!, $albumName: String, $artists: [String!], $duration: Int) {\n\t\tsearchSoulseek(\n\t\t\ttrackTitle: $trackTitle\n\t\t\talbumName: $albumName\n\t\t\tartists: $artists\n\t\t\tduration: $duration\n\t\t) {\n\t\t\tusername\n\t\t\ttoken\n\t\t\tfilename\n\t\t\tsize\n\t\t\tavgSpeed\n\t\t\tqueueLength\n\t\t\tslotsFree\n\t\t\tattributes {\n\t\t\t\tattribute\n\t\t\t\tvalue\n\t\t\t}\n\t\t}\n\t}\n"): typeof import('./graphql').SearchSoulseekDocument;
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n  query Test {\n    howdy\n  }\n"): typeof import('./graphql').TestDocument;
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n\tquery PlaylistTracks($playlistId: Int!, $page: Int, $pageSize: Int) {\n\t\tplaylistTracks(playlistId: $playlistId, page: $page, pageSize: $pageSize) {\n\t\t\ttracks {\n\t\t\t\tid\n\t\t\t\ttitle\n\t\t\t\ttrackNumber\n\t\t\t\tduration\n\t\t\t\tcreatedAt\n\t\t\t\talbum {\n\t\t\t\t\tid\n\t\t\t\t\ttitle\n\t\t\t\t\tyear\n\t\t\t\t\tartworkUrl\n\t\t\t\t}\n\t\t\t\tartists {\n\t\t\t\t\tid\n\t\t\t\t\tname\n\t\t\t\t}\n\t\t\t}\n\t\t\ttotalCount\n\t\t\tpage\n\t\t\tpageSize\n\t\t}\n\t}\n"): typeof import('./graphql').PlaylistTracksDocument;
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n\tquery Playlist($id: Int!) {\n\t\tplaylist(id: $id) {\n\t\t\tid\n\t\t\tname\n\t\t\tdescription\n\t\t\ttrackCount\n\t\t}\n\t}\n"): typeof import('./graphql').PlaylistDocument;
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n  query Playlists(\n    $page: Int\n    $pageSize: Int\n    $search: String\n    $sortBy: String\n    $sortOrder: String\n  ) {\n    playlists(\n      page: $page\n      pageSize: $pageSize\n      search: $search\n      sortBy: $sortBy\n      sortOrder: $sortOrder\n    ) {\n      playlists {\n        id\n        name\n        description\n        createdAt\n        updatedAt\n        trackCount\n      }\n      totalCount\n      page\n      pageSize\n    }\n  }\n"): typeof import('./graphql').PlaylistsDocument;
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n  mutation SyncPlaylistToPlex($playlistId: Int!) {\n    syncPlaylistToPlex(playlistId: $playlistId) {\n      missingTracks {\n        trackId\n        filePath\n        title\n      }\n      tracksAdded\n      tracksRemoved\n      tracksSkipped\n    }\n  }\n"): typeof import('./graphql').SyncPlaylistToPlexDocument;
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n  mutation CompletePlexServerAuthentication($serverId: Int!, $pinId: Int!) {\n    completePlexServerAuthentication(serverId: $serverId, pinId: $pinId) {\n      id\n      name\n      serverUrl\n      hasAccessToken\n      createdAt\n      updatedAt\n    }\n  }\n"): typeof import('./graphql').CompletePlexServerAuthenticationDocument;
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n  query PlexServers {\n    plexServers {\n      id\n      name\n      serverUrl\n      hasAccessToken\n      createdAt\n      updatedAt\n    }\n  }\n"): typeof import('./graphql').PlexServersDocument;
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n  mutation CreatePlexServer($name: String!, $serverUrl: String!) {\n    createPlexServer(name: $name, serverUrl: $serverUrl) {\n      id\n      name\n      serverUrl\n      hasAccessToken\n      createdAt\n      updatedAt\n    }\n  }\n"): typeof import('./graphql').CreatePlexServerDocument;
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n  mutation AuthenticatePlexServer($serverId: Int!) {\n    authenticatePlexServer(serverId: $serverId) {\n      authUrl\n      pinId\n    }\n  }\n"): typeof import('./graphql').AuthenticatePlexServerDocument;
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n  mutation RefreshMusicLibrary($plexServerId: Int!) {\n    refreshMusicLibrary(plexServerId: $plexServerId) {\n      success\n      message\n      sectionId\n    }\n  }\n"): typeof import('./graphql').RefreshMusicLibraryDocument;
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n  query MusicLibraryScanStatus($plexServerId: Int!) {\n    musicLibraryScanStatus(plexServerId: $plexServerId) {\n      isScanning\n      progress\n      title\n      subtitle\n    }\n  }\n"): typeof import('./graphql').MusicLibraryScanStatusDocument;
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n  query PlexTracks {\n    plexTracks {\n      ... on PlexTracksSuccess {\n        tracks {\n          title\n          album\n          artist\n        }\n      }\n      ... on NoPlexServerError {\n        message\n      }\n      ... on MultiplePlexServersError {\n        message\n        serverCount\n      }\n      ... on PlexTracksError {\n        message\n      }\n    }\n  }\n"): typeof import('./graphql').PlexTracksDocument;
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n  mutation CompleteSpotifyAuth($authCode: String!, $csrfState: String!) {\n    completeSpotifyAuth(authCode: $authCode, csrfState: $csrfState) {\n      id\n      userId\n      displayName\n      createdAt\n      updatedAt\n    }\n  }\n"): typeof import('./graphql').CompleteSpotifyAuthDocument;
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n  mutation InitiateSpotifyAuth {\n    initiateSpotifyAuth {\n      redirectUrl\n    }\n  }\n"): typeof import('./graphql').InitiateSpotifyAuthDocument;
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n  query SpotifyMatchedTracks($page: Int, $pageSize: Int, $search: String) {\n    spotifyMatchedTracks(page: $page, pageSize: $pageSize, search: $search) {\n      matchedTracks {\n        spotifyTrackId\n        spotifyTitle\n        spotifyArtists\n        spotifyAlbum\n        spotifyIsrc\n        spotifyDuration\n        spotifyCreatedAt\n        spotifyUpdatedAt\n        localTrack {\n          id\n          title\n          trackNumber\n          duration\n          createdAt\n          album {\n            id\n            title\n            year\n            artworkUrl\n          }\n          artists {\n            id\n            name\n          }\n        }\n      }\n      totalCount\n      page\n      pageSize\n    }\n  }\n"): typeof import('./graphql').SpotifyMatchedTracksDocument;
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n  query SpotifyAccounts {\n    spotifyAccounts {\n      id\n      userId\n      displayName\n      createdAt\n      updatedAt\n    }\n  }\n"): typeof import('./graphql').SpotifyAccountsDocument;
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n  query SpotifyPlaylists($accountId: Int!) {\n    spotifyPlaylists(accountId: $accountId) {\n      id\n      spotifyId\n      name\n      description\n      trackCount\n      createdAt\n      updatedAt\n    }\n  }\n"): typeof import('./graphql').SpotifyPlaylistsDocument;
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n  query SpotifyPlaylistSyncState($spotifyPlaylistId: Int!) {\n    spotifyPlaylistSyncState(spotifyPlaylistId: $spotifyPlaylistId) {\n      id\n      spotifyPlaylistId\n      localPlaylistId\n      lastSyncAt\n      syncStatus\n      tracksDownloaded\n      tracksFailed\n      errorLog\n    }\n  }\n"): typeof import('./graphql').SpotifyPlaylistSyncStateDocument;
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n  query SpotifyTrackDownloadFailures($spotifyPlaylistId: Int!) {\n    spotifyTrackDownloadFailures(spotifyPlaylistId: $spotifyPlaylistId) {\n      id\n      spotifyPlaylistId\n      spotifyTrackId\n      trackName\n      artistName\n      albumName\n      isrc\n      reason\n      attemptsCount\n      createdAt\n      updatedAt\n    }\n  }\n"): typeof import('./graphql').SpotifyTrackDownloadFailuresDocument;
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n  mutation SyncSpotifyPlaylists($accountId: Int!) {\n    syncSpotifyAccountPlaylistsToDb(accountId: $accountId)\n  }\n"): typeof import('./graphql').SyncSpotifyPlaylistsDocument;
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n  mutation MatchTracks {\n    matchExistingSpotifyTracksWithLocalTracks\n  }\n"): typeof import('./graphql').MatchTracksDocument;
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n  mutation SyncPlaylistToLocalLibrary(\n    $spotifyAccountId: Int!\n    $spotifyPlaylistId: Int!\n    $localPlaylistName: String!\n  ) {\n    syncSpotifyPlaylistToLocalLibrary(\n      spotifyAccountId: $spotifyAccountId\n      spotifyPlaylistId: $spotifyPlaylistId\n      localPlaylistName: $localPlaylistName\n    )\n  }\n"): typeof import('./graphql').SyncPlaylistToLocalLibraryDocument;
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n  query Tracks(\n    $pagination: PaginationInput\n    $search: TextSearchInput\n    $sort: [TrackSortInput!]\n  ) {\n    tracks(pagination: $pagination, search: $search, sort: $sort) {\n      tracks {\n        id\n        title\n        trackNumber\n        duration\n        createdAt\n        album {\n          id\n          title\n          year\n          artworkUrl\n        }\n        artists {\n          id\n          name\n        }\n      }\n      totalCount\n      page\n      pageSize\n    }\n  }\n"): typeof import('./graphql').TracksDocument;
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n  query UnimportableFiles($page: Int, $pageSize: Int) {\n    unimportableFiles(page: $page, pageSize: $pageSize) {\n      files {\n        id\n        filePath\n        reason\n        createdAt\n        sha256\n      }\n      totalCount\n      page\n      pageSize\n    }\n  }\n"): typeof import('./graphql').UnimportableFilesDocument;


export function graphql(source: string) {
  return (documents as any)[source] ?? {};
}
