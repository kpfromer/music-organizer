import { useQuery } from "@tanstack/react-query";
import {
  type ColumnDef,
  flexRender,
  getCoreRowModel,
  useReactTable,
} from "@tanstack/react-table";
import { AlertCircle, Loader2 } from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
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

const PlexTracksQuery = graphql(`
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
`);

type PlexTrack = {
  title: string;
  album: string | null;
  artist: string | null;
};

export function PlexTracks() {
  const { data, isLoading, error } = useQuery({
    queryKey: ["plexTracks"],
    queryFn: () => execute(PlexTracksQuery),
  });

  // Define columns outside conditional logic
  const columns: ColumnDef<PlexTrack>[] = [
    {
      accessorKey: "title",
      header: "Track Name",
    },
    {
      accessorKey: "album",
      header: "Album Name",
      cell: ({ row }) => row.original.album ?? "-",
    },
    {
      accessorKey: "artist",
      header: "Artist Name",
      cell: ({ row }) => row.original.artist ?? "-",
    },
  ];

  // Process tracks data - use empty array if no tracks
  const result = data?.plexTracks;
  const tracks: PlexTrack[] =
    result && "tracks" in result
      ? result.tracks.map((track) => ({
          title: track.title,
          album: track.album ?? null,
          artist: track.artist ?? null,
        }))
      : [];

  // Always call useReactTable hook (Rules of Hooks)
  const table = useReactTable({
    data: tracks,
    columns,
    getCoreRowModel: getCoreRowModel(),
  });

  if (isLoading) {
    return (
      <div className="flex items-center justify-center p-8">
        <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
      </div>
    );
  }

  if (error) {
    return (
      <div className="p-8">
        <Card className="border-destructive">
          <CardHeader>
            <CardTitle className="flex items-center gap-2 text-destructive">
              <AlertCircle className="h-4 w-4" />
              Error
            </CardTitle>
          </CardHeader>
          <CardContent>
            <p>Failed to load Plex tracks: {error.message}</p>
          </CardContent>
        </Card>
      </div>
    );
  }

  // Handle union type - check which variant we have
  if (!result) {
    return (
      <div className="p-8">
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <AlertCircle className="h-4 w-4" />
              No Data
            </CardTitle>
          </CardHeader>
          <CardContent>
            <p>No data returned from server.</p>
          </CardContent>
        </Card>
      </div>
    );
  }

  // Check for error variants
  if ("message" in result && !("tracks" in result)) {
    // It's an error variant
    const errorMessage = result.message;
    const serverCount =
      "serverCount" in result ? result.serverCount : undefined;

    return (
      <div className="p-8">
        <Card className="border-destructive">
          <CardHeader>
            <CardTitle className="flex items-center gap-2 text-destructive">
              <AlertCircle className="h-4 w-4" />
              Error
            </CardTitle>
          </CardHeader>
          <CardContent>
            <p>{errorMessage}</p>
            {serverCount !== undefined && (
              <p className="mt-2">
                Found {serverCount} server{serverCount !== 1 ? "s" : ""}.
              </p>
            )}
          </CardContent>
        </Card>
      </div>
    );
  }

  // It's a success variant - render table
  if ("tracks" in result) {
    return (
      <div className="p-8">
        <h1 className="text-2xl font-bold mb-4">Plex Tracks</h1>
        <div className="rounded-md border">
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
      </div>
    );
  }

  // Fallback - should not reach here, but handle just in case

  // Fallback
  return (
    <div className="p-8">
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <AlertCircle className="h-4 w-4" />
            Unknown State
          </CardTitle>
        </CardHeader>
        <CardContent>
          <p>Unable to determine the state of the response.</p>
        </CardContent>
      </Card>
    </div>
  );
}
