import { useQuery } from "@tanstack/react-query";
import {
  type ColumnDef,
  flexRender,
  getCoreRowModel,
  useReactTable,
} from "@tanstack/react-table";
import { parseISO } from "date-fns";
import { ChevronLeft, ChevronRight, Music, Search, X } from "lucide-react";
import { useState } from "react";
import { Link } from "react-router-dom";
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
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { graphql } from "@/graphql";
import type { SpotifyMatchedTrack } from "@/graphql/graphql";
import { execute } from "@/lib/execute-graphql";

const SpotifyMatchedTracksQuery = graphql(`
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
`);

type MatchedTrack = Omit<
  SpotifyMatchedTrack,
  "spotifyCreatedAt" | "spotifyUpdatedAt"
> & {
  spotifyCreatedAt: Date;
  spotifyUpdatedAt: Date;
  localTrack: Omit<SpotifyMatchedTrack["localTrack"], "createdAt"> & {
    createdAt: Date;
  };
};

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

export function SpotifyMatchedTracks() {
  const [page, setPage] = useState(1);
  const [pageSize, setPageSize] = useState(25);
  const [search, setSearch] = useState("");
  const [searchInput, setSearchInput] = useState("");

  const { data, isLoading } = useQuery({
    queryKey: ["spotifyMatchedTracks", page, pageSize, search],
    queryFn: async () => {
      const result = await execute(SpotifyMatchedTracksQuery, {
        page,
        pageSize,
        search: search || undefined,
      });
      return {
        matchedTracks: result.spotifyMatchedTracks.matchedTracks.map(
          (track) => ({
            ...track,
            spotifyCreatedAt: parseISO(track.spotifyCreatedAt),
            spotifyUpdatedAt: parseISO(track.spotifyUpdatedAt),
            localTrack: {
              ...track.localTrack,
              createdAt: parseISO(track.localTrack.createdAt),
            },
          }),
        ) as MatchedTrack[],
        totalCount: result.spotifyMatchedTracks.totalCount,
        page: result.spotifyMatchedTracks.page,
        pageSize: result.spotifyMatchedTracks.pageSize,
      };
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

  const totalPages = data ? Math.ceil(data.totalCount / data.pageSize) : 0;

  const columns: ColumnDef<MatchedTrack>[] = [
    {
      header: "Spotify Track",
      cell: ({ row }) => (
        <div className="space-y-1">
          <div className="font-medium">{row.original.spotifyTitle}</div>
          <div className="text-muted-foreground text-sm">
            {formatArtists(row.original.spotifyArtists)}
          </div>
          <div className="text-muted-foreground text-xs">
            {row.original.spotifyAlbum}
          </div>
        </div>
      ),
    },
    {
      header: "Spotify ISRC",
      cell: ({ row }) => row.original.spotifyIsrc || "-",
    },
    {
      header: "Spotify Duration",
      cell: ({ row }) => formatDuration(row.original.spotifyDuration),
    },
    {
      header: "Local Track",
      cell: ({ row }) => (
        <div className="space-y-1">
          <Link
            to={`/tracks`}
            className="font-medium hover:underline"
            onClick={(e) => {
              // Could navigate to track detail if we had a route
              e.preventDefault();
            }}
          >
            {row.original.localTrack.title}
          </Link>
          <div className="text-muted-foreground text-sm">
            {formatArtists(row.original.localTrack.artists)}
          </div>
          <div className="text-muted-foreground text-xs">
            {row.original.localTrack.album.title}
            {row.original.localTrack.album.year &&
              ` (${row.original.localTrack.album.year})`}
          </div>
        </div>
      ),
    },
    {
      header: "Local Track #",
      cell: ({ row }) =>
        row.original.localTrack.trackNumber
          ? `#${row.original.localTrack.trackNumber}`
          : "-",
    },
    {
      header: "Local Duration",
      cell: ({ row }) => formatDuration(row.original.localTrack.duration),
    },
  ];

  const table = useReactTable({
    data: data?.matchedTracks || [],
    columns,
    getCoreRowModel: getCoreRowModel(),
  });

  return (
    <div className="container mx-auto p-8 space-y-6">
      <div className="flex items-center justify-between">
        <h1 className="text-3xl font-bold">Matched Spotify Tracks</h1>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>Matched Tracks</CardTitle>
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
                  if (e.key === "Enter") {
                    handleSearch();
                  }
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

          {/* Table */}
          {isLoading ? (
            <div className="flex items-center justify-center p-8">
              <Music className="h-6 w-6 animate-pulse" />
            </div>
          ) : data?.matchedTracks.length === 0 ? (
            <div className="flex flex-col items-center justify-center p-8 text-center">
              <Music className="mb-4 h-12 w-12 text-muted-foreground" />
              <p className="text-muted-foreground">
                {search
                  ? "No matched tracks found matching your search."
                  : "No matched tracks found. Run the matching operation first."}
              </p>
            </div>
          ) : (
            <>
              <Table>
                <TableHeader>
                  {table.getHeaderGroups().map((headerGroup) => (
                    <TableRow key={headerGroup.id}>
                      {headerGroup.headers.map((header) => (
                        <TableHead key={header.id}>
                          {header.isPlaceholder
                            ? null
                            : flexRender(
                                header.column.columnDef.header,
                                header.getContext(),
                              )}
                        </TableHead>
                      ))}
                    </TableRow>
                  ))}
                </TableHeader>
                <TableBody>
                  {table.getRowModel().rows.map((row) => (
                    <TableRow key={row.id}>
                      {row.getVisibleCells().map((cell) => (
                        <TableCell key={cell.id}>
                          {flexRender(
                            cell.column.columnDef.cell,
                            cell.getContext(),
                          )}
                        </TableCell>
                      ))}
                    </TableRow>
                  ))}
                </TableBody>
              </Table>

              {/* Pagination */}
              {totalPages > 1 && data && (
                <div className="flex items-center justify-between">
                  <div className="text-muted-foreground text-sm">
                    Showing {(data.page - 1) * data.pageSize + 1} to{" "}
                    {Math.min(data.page * data.pageSize, data.totalCount)} of{" "}
                    {data.totalCount} matched tracks
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
                      Page {data.page} of {totalPages}
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
                    setPageSize(parseInt(value, 10));
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
