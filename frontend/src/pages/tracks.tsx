import { useQuery } from "@tanstack/react-query";
import {
  type ColumnDef,
  flexRender,
  getCoreRowModel,
  getSortedRowModel,
  type SortingState,
  useReactTable,
} from "@tanstack/react-table";
import { formatDistanceToNow, parseISO } from "date-fns";
import {
  ArrowDown,
  ArrowUp,
  ArrowUpDown,
  ChevronLeft,
  ChevronRight,
  Download,
} from "lucide-react";
import { useState } from "react";
import { Button } from "@/components/ui/button";
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
import { execute } from "@/lib/execute-graphql";

const TracksQuery = graphql(`
  query Tracks($page: Int, $pageSize: Int) {
    tracks(page: $page, pageSize: $pageSize) {
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

type Track = {
  id: number;
  title: string;
  trackNumber: number | null;
  duration: number | null;
  createdAt: Date;
  album: {
    id: number;
    title: string;
    year: number | null;
    artworkUrl: string | null;
  };
  artists: Array<{
    id: number;
    name: string;
  }>;
};

function formatDuration(seconds: number | null): string {
  if (!seconds) return "0:00";
  const minutes = Math.floor(seconds / 60);
  const remainingSeconds = seconds % 60;
  return `${minutes}:${remainingSeconds.toString().padStart(2, "0")}`;
}

export function Tracks() {
  const [page, setPage] = useState(1);
  const [pageSize, setPageSize] = useState(25);
  const [sorting, setSorting] = useState<SortingState>([
    { id: "createdAt", desc: true },
  ]);

  const { data, isLoading } = useQuery({
    queryKey: ["tracks", page, pageSize],
    queryFn: async () => {
      const result = await execute(TracksQuery, {
        page,
        pageSize,
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

  const columns: ColumnDef<Track>[] = [
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
      header: "Title",
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
        return (
          <Button
            variant="ghost"
            onClick={() => column.toggleSorting(column.getIsSorted() === "asc")}
            className="h-8 px-2"
          >
            Date added
            {column.getIsSorted() === "desc" ? (
              <ArrowDown className="ml-2 h-4 w-4" />
            ) : column.getIsSorted() === "asc" ? (
              <ArrowUp className="ml-2 h-4 w-4" />
            ) : (
              <ArrowUpDown className="ml-2 h-4 w-4" />
            )}
          </Button>
        );
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
      header: "Duration",
      cell: ({ row }) => {
        const duration = row.getValue("duration") as number | null;
        return (
          <div className="text-muted-foreground">
            {formatDuration(duration)}
          </div>
        );
      },
    },
  ];

  const table = useReactTable({
    data: data?.tracks ?? [],
    columns,
    getCoreRowModel: getCoreRowModel(),
    getSortedRowModel: getSortedRowModel(),
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
              table.getRowModel().rows.map((row) => (
                <TableRow
                  key={row.id}
                  data-state={row.getIsSelected() && "selected"}
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
              ))
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

      {/* Pagination Controls */}
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
