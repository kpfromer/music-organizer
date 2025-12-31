import { useQuery } from "@tanstack/react-query";
import {
  type ColumnDef,
  flexRender,
  getCoreRowModel,
  type SortingState,
  useReactTable,
} from "@tanstack/react-table";
import { formatDistanceToNow, parseISO } from "date-fns";
import { ChevronLeft, ChevronRight, Download, Search, X } from "lucide-react";
import { useMemo, useState } from "react";
import { TrackContextMenu } from "@/components/track-context-menu";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { SortButton } from "@/components/ui/sort-button";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { graphql } from "@/graphql";
import type { Track as GraphQLTrack } from "@/graphql/graphql";
import { execute } from "@/lib/execute-graphql";
import {
  buildPaginationInput,
  buildTextSearchInput,
  buildTrackSortInput,
} from "@/lib/query-builder";
import { useAudioPlayerStore } from "@/stores/audio-player-store";

const TracksQuery = graphql(`
  query Tracks(
    $pagination: PaginationInput
    $search: TextSearchInput
    $sort: [TrackSortInput!]
  ) {
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
`);

// Transform the generated Track type to have createdAt as Date instead of DateTime scalar
type Track = Omit<GraphQLTrack, "createdAt"> & {
  createdAt: Date;
};

function formatDuration(seconds: number | null): string {
  if (!seconds) return "0:00";
  const minutes = Math.floor(seconds / 60);
  const remainingSeconds = seconds % 60;
  return `${minutes}:${remainingSeconds.toString().padStart(2, "0")}`;
}

export function Tracks() {
  const playTrack = useAudioPlayerStore((state) => state.playTrack);
  const [page, setPage] = useState(1);
  const [pageSize, setPageSize] = useState(25);
  const [search, setSearch] = useState("");
  const [searchInput, setSearchInput] = useState("");
  const [sorting, setSorting] = useState<SortingState>([
    { id: "createdAt", desc: true },
  ]);

  const { data, isLoading } = useQuery({
    queryKey: ["tracks", page, pageSize, search, sorting],
    queryFn: async () => {
      const result = await execute(TracksQuery, {
        pagination: buildPaginationInput(page, pageSize),
        search: buildTextSearchInput(search),
        sort: buildTrackSortInput(sorting),
      });
      return {
        ...result.tracks,
        tracks: result.tracks.tracks.map((track) => ({
          ...track,
          createdAt: parseISO(track.createdAt),
        })),
      };
    },
  });

  const handleSearch = () => {
    setSearch(searchInput);
    setPage(1);
  };

  const handleSearchKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === "Enter") {
      handleSearch();
    }
  };

  const columns: ColumnDef<Track>[] = useMemo(
    () => [
      {
        accessorKey: "trackNumber",
        header: "#",
        cell: ({ row }) => {
          const value = row.getValue("trackNumber") as number | null;
          return <div className="text-muted-foreground">{value ?? ""}</div>;
        },
      },
      {
        accessorKey: "title",
        header: ({ column }) => {
          return <SortButton column={column}>Title</SortButton>;
        },
        cell: ({ row }) => {
          const track = row.original;
          const primaryArtist = track.artists[0]?.name ?? "Unknown Artist";
          return (
            <div className="flex items-center gap-3">
              {track.album.artworkUrl ? (
                <img
                  src={track.album.artworkUrl}
                  alt={track.album.title}
                  className="h-10 w-10 rounded object-cover"
                />
              ) : (
                <div className="flex h-10 w-10 items-center justify-center rounded bg-muted text-xs text-muted-foreground">
                  {track.album.title.charAt(0).toUpperCase()}
                </div>
              )}
              <div className="flex flex-col">
                <div className="font-medium">{track.title}</div>
                <div className="flex items-center gap-1 text-sm text-muted-foreground">
                  <span>{primaryArtist}</span>
                  <Download className="h-3 w-3 text-green-500" />
                </div>
              </div>
            </div>
          );
        },
      },
      {
        accessorFn: (row) => row.album.title,
        id: "album",
        header: "Album",
        cell: ({ row }) => {
          return (
            <div className="text-muted-foreground">
              {row.original.album.title}
            </div>
          );
        },
      },
      {
        accessorKey: "createdAt",
        header: ({ column }) => {
          return <SortButton column={column}>Date added</SortButton>;
        },
        cell: ({ row }) => {
          const date = row.getValue("createdAt") as Date;
          return (
            <div className="text-muted-foreground">
              {formatDistanceToNow(date, { addSuffix: true })}
            </div>
          );
        },
      },
      {
        accessorKey: "duration",
        header: ({ column }) => {
          return <SortButton column={column}>Duration</SortButton>;
        },
        cell: ({ row }) => {
          const duration = row.getValue("duration") as number | null;
          return (
            <div className="text-muted-foreground">
              {formatDuration(duration)}
            </div>
          );
        },
      },
    ],
    [],
  );

  const table = useReactTable({
    data: data?.tracks ?? [],
    columns,
    getCoreRowModel: getCoreRowModel(),
    onSortingChange: setSorting,
    state: {
      sorting,
    },
    manualPagination: true,
  });

  const totalPages = data ? Math.ceil(data.totalCount / data.pageSize) : 0;

  if (isLoading) {
    return (
      <div className="container mx-auto p-8">
        <div className="text-muted-foreground">Loading tracks...</div>
      </div>
    );
  }

  return (
    <div className="container mx-auto p-8">
      <div className="mb-4 flex items-center justify-between">
        <h1 className="text-2xl font-bold">Tracks</h1>
        <div className="flex items-center gap-2">
          <span className="text-sm text-muted-foreground">Page size:</span>
          <Select
            value={pageSize.toString()}
            onValueChange={(value) => {
              setPageSize(Number(value));
              setPage(1);
            }}
          >
            <SelectTrigger className="w-20">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="10">10</SelectItem>
              <SelectItem value="25">25</SelectItem>
              <SelectItem value="50">50</SelectItem>
              <SelectItem value="100">100</SelectItem>
            </SelectContent>
          </Select>
        </div>
      </div>

      {/* Search Input */}
      <div className="mb-4 flex items-center gap-2">
        <div className="relative flex-1 max-w-sm">
          <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
          <Input
            type="text"
            placeholder="Search tracks by title..."
            value={searchInput}
            onChange={(e) => setSearchInput(e.target.value)}
            onKeyDown={handleSearchKeyDown}
            className="pl-9"
          />
        </div>
        <Button onClick={handleSearch} variant="outline">
          Search
        </Button>
        {search && (
          <Button
            onClick={() => {
              setSearch("");
              setSearchInput("");
              setPage(1);
            }}
            variant="ghost"
            size="icon"
          >
            <X className="h-4 w-4" />
          </Button>
        )}
      </div>

      <div className="rounded-lg border">
        <Table>
          <TableHeader>
            {table.getHeaderGroups().map((headerGroup) => (
              <TableRow key={headerGroup.id}>
                {headerGroup.headers.map((header) => {
                  return (
                    <TableHead key={header.id}>
                      {header.isPlaceholder
                        ? null
                        : flexRender(
                            header.column.columnDef.header,
                            header.getContext(),
                          )}
                    </TableHead>
                  );
                })}
              </TableRow>
            ))}
          </TableHeader>
          <TableBody>
            {table.getRowModel().rows?.length ? (
              table.getRowModel().rows.map((row) => {
                const track = row.original;
                return (
                  <TrackContextMenu
                    key={row.id}
                    trackId={track.id}
                    trackTitle={track.title}
                  >
                    <TableRow
                      data-state={row.getIsSelected() && "selected"}
                      onClick={() => playTrack(track)}
                      className="cursor-pointer hover:bg-muted/50"
                    >
                      {row.getVisibleCells().map((cell) => (
                        <TableCell key={cell.id}>
                          {flexRender(
                            cell.column.columnDef.cell,
                            cell.getContext(),
                          )}
                        </TableCell>
                      ))}
                    </TableRow>
                  </TrackContextMenu>
                );
              })
            ) : (
              <TableRow>
                <TableCell
                  colSpan={columns.length}
                  className="h-24 text-center"
                >
                  No tracks found.
                </TableCell>
              </TableRow>
            )}
          </TableBody>
        </Table>
      </div>
      <div className="mt-4 flex items-center justify-between">
        <div className="text-sm text-muted-foreground">
          Showing {(page - 1) * pageSize + 1} to{" "}
          {Math.min(page * pageSize, data?.totalCount ?? 0)} of{" "}
          {data?.totalCount ?? 0} tracks
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
          <div className="text-sm text-muted-foreground">
            Page {data?.page ?? 1} of {totalPages || 1}
          </div>
          <Button
            variant="outline"
            size="sm"
            onClick={() => setPage((p) => Math.min(totalPages, p + 1))}
            disabled={page >= totalPages}
          >
            Next
            <ChevronRight className="h-4 w-4" />
          </Button>
        </div>
      </div>
    </div>
  );
}
