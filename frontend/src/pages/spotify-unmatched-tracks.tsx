import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import {
  Check,
  ChevronDown,
  ChevronLeft,
  ChevronRight,
  ChevronUp,
  Music,
  Search,
  X,
  XCircle,
} from "lucide-react";
import { useEffect, useState } from "react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { graphql } from "@/graphql";
import { execute } from "@/lib/execute-graphql";

const SpotifyAccountsQuery = graphql(`
  query SpotifyAccountsForFilter {
    spotifyAccounts {
      id
      displayName
      userId
    }
  }
`);

const SpotifyPlaylistsQuery = graphql(`
  query SpotifyPlaylistsForFilter($accountId: Int!) {
    spotifyPlaylists(accountId: $accountId) {
      id
      name
      trackCount
    }
  }
`);

const SpotifyUnmatchedTracksQuery = graphql(`
  query SpotifyUnmatchedTracks($page: Int, $pageSize: Int, $search: String, $hasCandidates: Boolean, $sortByScore: Boolean, $playlistId: Int) {
    spotifyUnmatchedTracks(page: $page, pageSize: $pageSize, search: $search, hasCandidates: $hasCandidates, sortByScore: $sortByScore, playlistId: $playlistId) {
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
`);

const SearchLocalTracksQuery = graphql(`
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
`);

const AcceptCandidateMutation = graphql(`
  mutation AcceptSpotifyMatchCandidate($candidateId: Int!) {
    acceptSpotifyMatchCandidate(candidateId: $candidateId)
  }
`);

const DismissTrackMutation = graphql(`
  mutation DismissSpotifyUnmatchedTrack($spotifyTrackId: String!) {
    dismissSpotifyUnmatchedTrack(spotifyTrackId: $spotifyTrackId)
  }
`);

const ManualMatchMutation = graphql(`
  mutation ManuallyMatchSpotifyTrack($spotifyTrackId: String!, $localTrackId: Int!) {
    manuallyMatchSpotifyTrack(spotifyTrackId: $spotifyTrackId, localTrackId: $localTrackId)
  }
`);

function formatDuration(seconds: number | null | undefined): string {
  if (!seconds) return "0:00";
  const minutes = Math.floor(seconds / 60);
  const remainingSeconds = seconds % 60;
  return `${minutes}:${remainingSeconds.toString().padStart(2, "0")}`;
}

function formatArtists(artists: string[] | { name: string }[]): string {
  if (artists.length === 0) return "-";
  if (typeof artists[0] === "string") {
    return (artists as string[]).join(", ");
  }
  return (artists as { name: string }[]).map((a) => a.name).join(", ");
}

function confidenceBadgeVariant(
  confidence: string,
): "default" | "secondary" | "destructive" | "outline" {
  switch (confidence) {
    case "High":
      return "default";
    case "Medium":
      return "secondary";
    case "Low":
      return "destructive";
    default:
      return "outline";
  }
}

function ScoreBar({ value, label }: { value: number; label: string }) {
  const percent = Math.round(value * 100);
  return (
    <div className="flex items-center gap-2 text-xs">
      <span className="w-16 text-muted-foreground">{label}</span>
      <div className="h-1.5 flex-1 rounded-full bg-muted">
        <div
          className="h-full rounded-full bg-primary"
          style={{ width: `${percent}%` }}
        />
      </div>
      <span className="w-8 text-right">{percent}%</span>
    </div>
  );
}

function LibrarySearch({
  spotifyTrackId,
  onClose,
}: {
  spotifyTrackId: string;
  onClose: () => void;
}) {
  const queryClient = useQueryClient();
  const [librarySearch, setLibrarySearch] = useState("");
  const [searchSubmitted, setSearchSubmitted] = useState("");

  const { data: searchResults, isLoading: searchLoading } = useQuery({
    queryKey: ["searchLocalTracks", searchSubmitted],
    queryFn: () =>
      execute(SearchLocalTracksQuery, {
        search: searchSubmitted,
        page: 1,
        pageSize: 10,
      }),
    enabled: searchSubmitted.length > 0,
  });

  const manualMatch = useMutation({
    mutationFn: (localTrackId: number) =>
      execute(ManualMatchMutation, {
        spotifyTrackId,
        localTrackId: localTrackId,
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["spotifyUnmatchedTracks"] });
    },
  });

  return (
    <div className="mt-2 rounded border p-3 space-y-2">
      <div className="flex items-center justify-between">
        <span className="text-sm font-medium">Search Library</span>
        <button type="button" onClick={onClose}>
          <X className="h-4 w-4 text-muted-foreground" />
        </button>
      </div>
      <div className="flex gap-2">
        <Input
          placeholder="Search by title..."
          value={librarySearch}
          onChange={(e) => setLibrarySearch(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "Enter") setSearchSubmitted(librarySearch);
          }}
          className="text-sm"
        />
        <Button
          size="sm"
          variant="outline"
          onClick={() => setSearchSubmitted(librarySearch)}
        >
          <Search className="h-3 w-3" />
        </Button>
      </div>
      {searchLoading && (
        <div className="flex justify-center p-2">
          <Music className="h-4 w-4 animate-pulse" />
        </div>
      )}
      {searchResults?.searchLocalTracksForMatching.tracks.map((track) => (
        <div
          key={track.id}
          className="flex items-center justify-between rounded border p-2 text-sm"
        >
          <div>
            <div className="font-medium">{track.title}</div>
            <div className="text-muted-foreground text-xs">
              {formatArtists(track.artists)} - {track.album.title}
              {track.duration ? ` (${formatDuration(track.duration)})` : ""}
            </div>
          </div>
          <Button
            size="sm"
            variant="outline"
            onClick={() => manualMatch.mutate(track.id)}
            disabled={manualMatch.isPending}
          >
            <Check className="mr-1 h-3 w-3" />
            Match
          </Button>
        </div>
      ))}
      {searchResults?.searchLocalTracksForMatching.tracks.length === 0 && (
        <p className="text-xs text-muted-foreground text-center py-2">
          No tracks found.
        </p>
      )}
    </div>
  );
}

export function SpotifyUnmatchedTracks() {
  const queryClient = useQueryClient();
  const [page, setPage] = useState(1);
  const [pageSize, setPageSize] = useState(25);
  const [search, setSearch] = useState("");
  const [searchInput, setSearchInput] = useState("");
  const [hasCandidates, setHasCandidates] = useState<boolean | undefined>(
    undefined,
  );
  const [sortByScore, setSortByScore] = useState(false);
  const [selectedAccountId, setSelectedAccountId] = useState<
    number | undefined
  >(undefined);
  const [selectedPlaylistId, setSelectedPlaylistId] = useState<
    number | undefined
  >(undefined);
  const [expandedTrackId, setExpandedTrackId] = useState<string | null>(null);
  const [librarySearchTrackId, setLibrarySearchTrackId] = useState<
    string | null
  >(null);

  const { data: accountsData } = useQuery({
    queryKey: ["spotifyAccountsForFilter"],
    queryFn: () => execute(SpotifyAccountsQuery),
  });

  // Auto-select the first account to load its playlists
  const accounts = accountsData?.spotifyAccounts;
  const firstAccountId = accounts?.[0]?.id;
  useEffect(() => {
    if (firstAccountId !== undefined && selectedAccountId === undefined) {
      setSelectedAccountId(firstAccountId);
    }
  }, [firstAccountId, selectedAccountId]);

  const { data: playlistsData } = useQuery({
    queryKey: ["spotifyPlaylistsForFilter", selectedAccountId],
    queryFn: () =>
      execute(SpotifyPlaylistsQuery, {
        accountId: selectedAccountId!,
      }),
    enabled: selectedAccountId !== undefined,
  });

  const { data, isLoading } = useQuery({
    queryKey: [
      "spotifyUnmatchedTracks",
      page,
      pageSize,
      search,
      hasCandidates,
      sortByScore,
      selectedPlaylistId,
    ],
    queryFn: () =>
      execute(SpotifyUnmatchedTracksQuery, {
        page,
        pageSize,
        search: search || undefined,
        hasCandidates,
        sortByScore: sortByScore || undefined,
        playlistId: selectedPlaylistId,
      }),
  });

  const acceptCandidate = useMutation({
    mutationFn: (candidateId: number) =>
      execute(AcceptCandidateMutation, { candidateId }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["spotifyUnmatchedTracks"] });
    },
  });

  const dismissTrack = useMutation({
    mutationFn: (spotifyTrackId: string) =>
      execute(DismissTrackMutation, { spotifyTrackId }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["spotifyUnmatchedTracks"] });
    },
  });

  const handleSearch = () => {
    setSearch(searchInput);
    setPage(1);
  };

  const handleClearSearch = () => {
    setSearchInput("");
    setSearch("");
    setPage(1);
  };

  const totalPages = data
    ? Math.ceil(
        data.spotifyUnmatchedTracks.totalCount /
          data.spotifyUnmatchedTracks.pageSize,
      )
    : 0;

  const unmatchedTracks = data?.spotifyUnmatchedTracks.unmatchedTracks ?? [];

  return (
    <div className="container mx-auto p-8 space-y-6">
      <div className="flex items-center justify-between">
        <h1 className="text-3xl font-bold">Unmatched Spotify Tracks</h1>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>Review Queue</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          {/* Search */}
          <div className="flex gap-2">
            <div className="relative flex-1">
              <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
              <Input
                placeholder="Search by track title or album..."
                value={searchInput}
                onChange={(e) => setSearchInput(e.target.value)}
                onKeyDown={(e) => {
                  if (e.key === "Enter") handleSearch();
                }}
                className="pl-9"
              />
              {searchInput && (
                <button
                  type="button"
                  onClick={handleClearSearch}
                  className="absolute right-3 top-1/2 -translate-y-1/2"
                >
                  <X className="h-4 w-4 text-muted-foreground" />
                </button>
              )}
            </div>
            <Button onClick={handleSearch} variant="outline">
              <Search className="mr-2 h-4 w-4" />
              Search
            </Button>
          </div>

          {/* Filters */}
          <div className="flex flex-wrap items-center gap-4">
            <div className="flex items-center gap-2">
              <span className="text-muted-foreground text-sm">Playlist:</span>
              <Select
                value={
                  selectedPlaylistId !== undefined
                    ? String(selectedPlaylistId)
                    : "all"
                }
                onValueChange={(value) => {
                  if (value === "all") {
                    setSelectedPlaylistId(undefined);
                  } else {
                    setSelectedPlaylistId(Number.parseInt(value, 10));
                  }
                  setPage(1);
                }}
              >
                <SelectTrigger className="w-48">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="all">All playlists</SelectItem>
                  {(playlistsData?.spotifyPlaylists ?? []).map((playlist) => (
                    <SelectItem key={playlist.id} value={String(playlist.id)}>
                      {playlist.name} ({playlist.trackCount})
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
            <div className="flex items-center gap-2">
              <span className="text-muted-foreground text-sm">Candidates:</span>
              <Select
                value={
                  hasCandidates === undefined
                    ? "all"
                    : hasCandidates
                      ? "with"
                      : "without"
                }
                onValueChange={(value) => {
                  setHasCandidates(
                    value === "all"
                      ? undefined
                      : value === "with"
                        ? true
                        : false,
                  );
                  setPage(1);
                }}
              >
                <SelectTrigger className="w-40">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="all">All</SelectItem>
                  <SelectItem value="with">With candidates</SelectItem>
                  <SelectItem value="without">Without candidates</SelectItem>
                </SelectContent>
              </Select>
            </div>
            <div className="flex items-center gap-2">
              <span className="text-muted-foreground text-sm">Sort by:</span>
              <Select
                value={sortByScore ? "score" : "recent"}
                onValueChange={(value) => {
                  setSortByScore(value === "score");
                  setPage(1);
                }}
              >
                <SelectTrigger className="w-40">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="recent">Most recent</SelectItem>
                  <SelectItem value="score">Best match score</SelectItem>
                </SelectContent>
              </Select>
            </div>
          </div>

          {/* Content */}
          {isLoading ? (
            <div className="flex items-center justify-center p-8">
              <Music className="h-6 w-6 animate-pulse" />
            </div>
          ) : unmatchedTracks.length === 0 ? (
            <div className="flex flex-col items-center justify-center p-8 text-center">
              <Music className="mb-4 h-12 w-12 text-muted-foreground" />
              <p className="text-muted-foreground">
                {search
                  ? "No unmatched tracks found matching your search."
                  : "No unmatched tracks with candidates. Run the matching operation first."}
              </p>
            </div>
          ) : (
            <>
              <div className="space-y-2">
                {unmatchedTracks.map((track) => {
                  const isExpanded = expandedTrackId === track.spotifyTrackId;
                  return (
                    <div
                      key={track.spotifyTrackId}
                      className="rounded-lg border"
                    >
                      {/* Track header */}
                      <div className="flex items-center justify-between p-4">
                        <button
                          type="button"
                          className="flex flex-1 items-center gap-4 text-left"
                          onClick={() =>
                            setExpandedTrackId(
                              isExpanded ? null : track.spotifyTrackId,
                            )
                          }
                        >
                          {isExpanded ? (
                            <ChevronUp className="h-4 w-4 shrink-0" />
                          ) : (
                            <ChevronDown className="h-4 w-4 shrink-0" />
                          )}
                          <div className="min-w-0">
                            <div className="font-medium truncate">
                              {track.spotifyTitle}
                            </div>
                            <div className="text-muted-foreground text-sm">
                              {formatArtists(track.spotifyArtists)} -{" "}
                              {track.spotifyAlbum}
                            </div>
                          </div>
                        </button>
                        <div className="flex items-center gap-2 shrink-0 ml-4">
                          {track.spotifyDuration && (
                            <span className="text-muted-foreground text-sm">
                              {formatDuration(track.spotifyDuration)}
                            </span>
                          )}
                          <Badge variant="outline">
                            {track.candidates.length} candidate
                            {track.candidates.length !== 1 ? "s" : ""}
                          </Badge>
                          <Button
                            size="sm"
                            variant="ghost"
                            onClick={() =>
                              setLibrarySearchTrackId(
                                librarySearchTrackId === track.spotifyTrackId
                                  ? null
                                  : track.spotifyTrackId,
                              )
                            }
                          >
                            <Search className="mr-1 h-3 w-3" />
                            Search
                          </Button>
                          <Button
                            size="sm"
                            variant="ghost"
                            onClick={() =>
                              dismissTrack.mutate(track.spotifyTrackId)
                            }
                            disabled={dismissTrack.isPending}
                          >
                            <XCircle className="mr-1 h-3 w-3" />
                            Dismiss
                          </Button>
                        </div>
                      </div>

                      {/* Library search */}
                      {librarySearchTrackId === track.spotifyTrackId && (
                        <div className="px-4 pb-4">
                          <LibrarySearch
                            spotifyTrackId={track.spotifyTrackId}
                            onClose={() => setLibrarySearchTrackId(null)}
                          />
                        </div>
                      )}

                      {/* Expanded candidates */}
                      {isExpanded && track.candidates.length > 0 && (
                        <div className="border-t px-4 pb-4 pt-2 space-y-3">
                          {track.candidates.map((candidate) => (
                            <div
                              key={candidate.id}
                              className="flex items-start gap-4 rounded border p-3"
                            >
                              <div className="flex-1 space-y-2">
                                <div className="flex items-center gap-2">
                                  <span className="font-medium">
                                    {candidate.localTrack.title}
                                  </span>
                                  <Badge
                                    variant={confidenceBadgeVariant(
                                      candidate.confidence,
                                    )}
                                  >
                                    {candidate.confidence}
                                  </Badge>
                                  <span className="text-sm text-muted-foreground">
                                    {Math.round(candidate.score * 100)}%
                                  </span>
                                </div>
                                <div className="text-muted-foreground text-sm">
                                  {formatArtists(candidate.localTrack.artists)}{" "}
                                  - {candidate.localTrack.album.title}
                                  {candidate.localTrack.duration
                                    ? ` (${formatDuration(candidate.localTrack.duration)})`
                                    : ""}
                                </div>
                                <div className="grid grid-cols-3 gap-2">
                                  <ScoreBar
                                    value={candidate.titleSimilarity}
                                    label="Title"
                                  />
                                  <ScoreBar
                                    value={candidate.artistSimilarity}
                                    label="Artist"
                                  />
                                  <ScoreBar
                                    value={candidate.albumSimilarity}
                                    label="Album"
                                  />
                                </div>
                                <div className="flex gap-4 text-xs text-muted-foreground">
                                  <span>
                                    Duration: {candidate.durationMatch}
                                  </span>
                                  <span>Version: {candidate.versionMatch}</span>
                                </div>
                              </div>
                              <Button
                                size="sm"
                                onClick={() =>
                                  acceptCandidate.mutate(candidate.id)
                                }
                                disabled={acceptCandidate.isPending}
                              >
                                <Check className="mr-1 h-3 w-3" />
                                Accept
                              </Button>
                            </div>
                          ))}
                        </div>
                      )}

                      {isExpanded && track.candidates.length === 0 && (
                        <div className="border-t px-4 py-3 text-sm text-muted-foreground text-center">
                          No candidates found. Use "Search" to find a match
                          manually.
                        </div>
                      )}
                    </div>
                  );
                })}
              </div>

              {/* Pagination */}
              {totalPages > 1 && data && (
                <div className="flex items-center justify-between">
                  <div className="text-muted-foreground text-sm">
                    Showing{" "}
                    {(data.spotifyUnmatchedTracks.page - 1) *
                      data.spotifyUnmatchedTracks.pageSize +
                      1}{" "}
                    to{" "}
                    {Math.min(
                      data.spotifyUnmatchedTracks.page *
                        data.spotifyUnmatchedTracks.pageSize,
                      data.spotifyUnmatchedTracks.totalCount,
                    )}{" "}
                    of {data.spotifyUnmatchedTracks.totalCount} unmatched tracks
                  </div>
                  <div className="flex items-center gap-2">
                    <Button
                      variant="outline"
                      size="sm"
                      onClick={() => setPage((p) => Math.max(1, p - 1))}
                      disabled={page === 1}
                    >
                      <ChevronLeft className="h-4 w-4" />
                      Previous
                    </Button>
                    <span className="text-muted-foreground text-sm">
                      Page {data.spotifyUnmatchedTracks.page} of {totalPages}
                    </span>
                    <Button
                      variant="outline"
                      size="sm"
                      onClick={() =>
                        setPage((p) => Math.min(totalPages, p + 1))
                      }
                      disabled={page >= totalPages}
                    >
                      Next
                      <ChevronRight className="h-4 w-4" />
                    </Button>
                  </div>
                </div>
              )}

              {/* Page Size Selector */}
              <div className="flex items-center gap-2">
                <span className="text-muted-foreground text-sm">
                  Items per page:
                </span>
                <Select
                  value={pageSize.toString()}
                  onValueChange={(value) => {
                    setPageSize(Number.parseInt(value, 10));
                    setPage(1);
                  }}
                >
                  <SelectTrigger className="w-20">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="25">25</SelectItem>
                    <SelectItem value="50">50</SelectItem>
                    <SelectItem value="100">100</SelectItem>
                  </SelectContent>
                </Select>
              </div>
            </>
          )}
        </CardContent>
      </Card>
    </div>
  );
}
