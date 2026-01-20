import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import {
  type ColumnDef,
  flexRender,
  getCoreRowModel,
  useReactTable,
} from "@tanstack/react-table";
import { formatDistanceToNow, parseISO } from "date-fns";
import {
  AlertCircle,
  CheckCircle,
  Loader2,
  Music,
  RefreshCw,
  XCircle,
} from "lucide-react";
import { useState } from "react";
import { z } from "zod";
import { FormFieldContainer } from "@/components/form/FormFieldContainer";
import { FormTextField } from "@/components/form/FormTextField";
import { useAppForm } from "@/components/form/form-hooks";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { FieldSet } from "@/components/ui/field";
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
import type {
  MutationSyncSpotifyPlaylistToLocalLibraryArgs,
  SpotifyAccount,
  SpotifyPlaylist,
  SpotifyTrackDownloadFailure,
  SyncSpotifyPlaylistsMutationVariables,
} from "@/graphql/graphql";
import { execute } from "@/lib/execute-graphql";

const SpotifyAccountsQuery = graphql(`
  query SpotifyAccounts {
    spotifyAccounts {
      id
      userId
      displayName
      createdAt
      updatedAt
    }
  }
`);

const SpotifyPlaylistsQuery = graphql(`
  query SpotifyPlaylists($accountId: Int!) {
    spotifyPlaylists(accountId: $accountId) {
      id
      spotifyId
      name
      description
      trackCount
      createdAt
      updatedAt
    }
  }
`);

const SpotifyPlaylistSyncStateQuery = graphql(`
  query SpotifyPlaylistSyncState($spotifyPlaylistId: Int!) {
    spotifyPlaylistSyncState(spotifyPlaylistId: $spotifyPlaylistId) {
      id
      spotifyPlaylistId
      localPlaylistId
      lastSyncAt
      syncStatus
      tracksDownloaded
      tracksFailed
      errorLog
    }
  }
`);

const SpotifyTrackDownloadFailuresQuery = graphql(`
  query SpotifyTrackDownloadFailures($spotifyPlaylistId: Int!) {
    spotifyTrackDownloadFailures(spotifyPlaylistId: $spotifyPlaylistId) {
      id
      spotifyPlaylistId
      spotifyTrackId
      trackName
      artistName
      albumName
      isrc
      reason
      attemptsCount
      createdAt
      updatedAt
    }
  }
`);

const SyncSpotifyPlaylistsMutation = graphql(`
  mutation SyncSpotifyPlaylists($accountId: Int!) {
    syncSpotifyAccountPlaylistsToDb(accountId: $accountId)
  }
`);

const MatchTracksMutation = graphql(`
  mutation MatchTracks {
    matchExistingSpotifyTracksWithLocalTracks
  }
`);

const SyncPlaylistToLocalLibraryMutation = graphql(`
  mutation SyncPlaylistToLocalLibrary(
    $spotifyAccountId: Int!
    $spotifyPlaylistId: Int!
    $localPlaylistName: String!
  ) {
    syncSpotifyPlaylistToLocalLibrary(
      spotifyAccountId: $spotifyAccountId
      spotifyPlaylistId: $spotifyPlaylistId
      localPlaylistName: $localPlaylistName
    )
  }
`);

const syncPlaylistSchema = z.object({
  spotifyPlaylistId: z.string().min(1, "Playlist is required"),
  localPlaylistName: z.string().min(1, "Local playlist name is required"),
});

type SyncPlaylistFormData = z.infer<typeof syncPlaylistSchema>;

type SpotifyAccountWithDates = Omit<
  SpotifyAccount,
  "createdAt" | "updatedAt"
> & {
  createdAt: Date;
  updatedAt: Date;
};

type SpotifyPlaylistWithDates = Omit<
  SpotifyPlaylist,
  "createdAt" | "updatedAt"
> & {
  createdAt: Date;
  updatedAt: Date;
};

type SpotifyTrackDownloadFailureWithDates = Omit<
  SpotifyTrackDownloadFailure,
  "createdAt" | "updatedAt"
> & {
  createdAt: Date;
  updatedAt: Date;
};

export function Spotify() {
  const queryClient = useQueryClient();
  const [selectedAccountId, setSelectedAccountId] = useState<number | null>(
    null,
  );
  const [selectedPlaylistId, setSelectedPlaylistId] = useState<number | null>(
    null,
  );

  const { data: accountsData, isLoading: accountsLoading } = useQuery({
    queryKey: ["spotifyAccounts"],
    queryFn: async () => {
      const result = await execute(SpotifyAccountsQuery);
      return {
        accounts: result.spotifyAccounts.map((account) => ({
          ...account,
          createdAt: parseISO(account.createdAt),
          updatedAt: parseISO(account.updatedAt),
        })) as SpotifyAccountWithDates[],
      };
    },
  });

  const { data: playlistsData, isLoading: playlistsLoading } = useQuery({
    queryKey: ["spotifyPlaylists", selectedAccountId],
    queryFn: async () => {
      if (!selectedAccountId) return { playlists: [] };
      const result = await execute(SpotifyPlaylistsQuery, {
        accountId: selectedAccountId,
      });
      return {
        playlists: result.spotifyPlaylists.map((playlist) => ({
          ...playlist,
          createdAt: parseISO(playlist.createdAt),
          updatedAt: parseISO(playlist.updatedAt),
        })) as SpotifyPlaylistWithDates[],
      };
    },
    enabled: !!selectedAccountId,
  });

  const { data: syncStateData } = useQuery({
    queryKey: ["spotifyPlaylistSyncState", selectedPlaylistId],
    queryFn: async () => {
      if (!selectedPlaylistId) return null;
      const result = await execute(SpotifyPlaylistSyncStateQuery, {
        spotifyPlaylistId: selectedPlaylistId,
      });
      return result.spotifyPlaylistSyncState;
    },
    enabled: !!selectedPlaylistId,
  });

  const syncState = syncStateData;
  const shouldPoll =
    syncState?.syncStatus === "pending" ||
    syncState?.syncStatus === "in_progress";

  // Poll for sync state when active
  useQuery({
    queryKey: ["spotifyPlaylistSyncState", selectedPlaylistId, "poll"],
    queryFn: async () => {
      if (!selectedPlaylistId) return null;
      const result = await execute(SpotifyPlaylistSyncStateQuery, {
        spotifyPlaylistId: selectedPlaylistId,
      });
      return result.spotifyPlaylistSyncState;
    },
    enabled: !!selectedPlaylistId && shouldPoll,
    refetchInterval: shouldPoll ? 2000 : false,
  });

  const { data: failuresData } = useQuery({
    queryKey: ["spotifyTrackDownloadFailures", selectedPlaylistId],
    queryFn: async () => {
      if (!selectedPlaylistId) return { failures: [] };
      const result = await execute(SpotifyTrackDownloadFailuresQuery, {
        spotifyPlaylistId: selectedPlaylistId,
      });
      return {
        failures: result.spotifyTrackDownloadFailures.map((failure) => ({
          ...failure,
          createdAt: parseISO(failure.createdAt),
          updatedAt: parseISO(failure.updatedAt),
        })) as SpotifyTrackDownloadFailureWithDates[],
      };
    },
    enabled: !!selectedPlaylistId,
  });

  const syncPlaylists = useMutation({
    mutationFn: async (variables: SyncSpotifyPlaylistsMutationVariables) =>
      execute(SyncSpotifyPlaylistsMutation, variables),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["spotifyPlaylists"] });
    },
  });

  const matchTracks = useMutation({
    mutationFn: async () => execute(MatchTracksMutation),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["spotifyPlaylists"] });
    },
  });

  const syncPlaylistToLocal = useMutation({
    mutationFn: async (
      variables: MutationSyncSpotifyPlaylistToLocalLibraryArgs,
    ) => execute(SyncPlaylistToLocalLibraryMutation, variables),
    onSuccess: () => {
      queryClient.invalidateQueries({
        queryKey: ["spotifyPlaylistSyncState"],
      });
      queryClient.invalidateQueries({
        queryKey: ["spotifyTrackDownloadFailures"],
      });
    },
  });

  const syncForm = useAppForm({
    defaultValues: {
      spotifyPlaylistId: "",
      localPlaylistName: "",
    },
    validators: {
      onSubmit: syncPlaylistSchema,
    },
    onSubmit: async ({ value }: { value: SyncPlaylistFormData }) => {
      if (!selectedAccountId) return;
      await syncPlaylistToLocal.mutateAsync({
        spotifyAccountId: selectedAccountId,
        spotifyPlaylistId: parseInt(value.spotifyPlaylistId, 10),
        localPlaylistName: value.localPlaylistName,
      });
      syncForm.reset();
    },
  });

  const playlistColumns: ColumnDef<SpotifyPlaylistWithDates>[] = [
    {
      accessorKey: "name",
      header: "Name",
    },
    {
      accessorKey: "description",
      header: "Description",
      cell: ({ row }) => row.original.description || "-",
    },
    {
      accessorKey: "trackCount",
      header: "Tracks",
    },
    {
      accessorKey: "updatedAt",
      header: "Updated",
      cell: ({ row }) =>
        formatDistanceToNow(row.original.updatedAt, { addSuffix: true }),
    },
  ];

  const playlistTable = useReactTable({
    data: playlistsData?.playlists || [],
    columns: playlistColumns,
    getCoreRowModel: getCoreRowModel(),
  });

  const failureColumns: ColumnDef<SpotifyTrackDownloadFailureWithDates>[] = [
    {
      accessorKey: "trackName",
      header: "Track",
    },
    {
      accessorKey: "artistName",
      header: "Artist",
    },
    {
      accessorKey: "albumName",
      header: "Album",
      cell: ({ row }) => row.original.albumName || "-",
    },
    {
      accessorKey: "isrc",
      header: "ISRC",
      cell: ({ row }) => row.original.isrc || "-",
    },
    {
      accessorKey: "reason",
      header: "Reason",
    },
    {
      accessorKey: "attemptsCount",
      header: "Attempts",
    },
    {
      accessorKey: "createdAt",
      header: "Failed At",
      cell: ({ row }) =>
        formatDistanceToNow(row.original.createdAt, { addSuffix: true }),
    },
  ];

  const failureTable = useReactTable({
    data: failuresData?.failures || [],
    columns: failureColumns,
    getCoreRowModel: getCoreRowModel(),
  });

  const getStatusBadge = (status: string) => {
    switch (status) {
      case "completed":
        return (
          <Badge variant="default" className="bg-green-500">
            <CheckCircle className="mr-1 h-3 w-3" />
            Completed
          </Badge>
        );
      case "in_progress":
        return (
          <Badge variant="default" className="bg-blue-500">
            <Loader2 className="mr-1 h-3 w-3 animate-spin" />
            In Progress
          </Badge>
        );
      case "error":
        return (
          <Badge variant="destructive">
            <XCircle className="mr-1 h-3 w-3" />
            Error
          </Badge>
        );
      case "pending":
        return (
          <Badge variant="secondary">
            <Loader2 className="mr-1 h-3 w-3 animate-spin" />
            Pending
          </Badge>
        );
      default:
        return <Badge variant="secondary">{status}</Badge>;
    }
  };

  return (
    <div className="container mx-auto p-8 space-y-6">
      <h1 className="text-3xl font-bold">Spotify Management</h1>

      {/* Account Selection */}
      <Card>
        <CardHeader>
          <CardTitle>Account Selection</CardTitle>
          <CardDescription>
            Select a Spotify account to manage playlists
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <Select
            value={selectedAccountId?.toString() || ""}
            onValueChange={(value) => {
              setSelectedAccountId(value ? parseInt(value, 10) : null);
              setSelectedPlaylistId(null);
            }}
          >
            <SelectTrigger className="w-full max-w-md">
              <SelectValue placeholder="Select an account" />
            </SelectTrigger>
            <SelectContent>
              {accountsData?.accounts.map((account) => (
                <SelectItem key={account.id} value={account.id.toString()}>
                  {account.displayName || account.userId}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>

          {selectedAccountId && (
            <Button
              onClick={() => {
                syncPlaylists.mutate({ accountId: selectedAccountId });
              }}
              disabled={syncPlaylists.isPending}
            >
              {syncPlaylists.isPending ? (
                <>
                  <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                  Syncing...
                </>
              ) : (
                <>
                  <RefreshCw className="mr-2 h-4 w-4" />
                  Sync Playlists from Spotify
                </>
              )}
            </Button>
          )}
        </CardContent>
      </Card>

      {/* Playlists */}
      {selectedAccountId && (
        <Card>
          <CardHeader>
            <CardTitle>Playlists</CardTitle>
            <CardDescription>
              Select a playlist to view sync status and failures
            </CardDescription>
          </CardHeader>
          <CardContent>
            {playlistsLoading ? (
              <div className="flex items-center justify-center p-8">
                <Loader2 className="h-6 w-6 animate-spin" />
              </div>
            ) : playlistsData?.playlists.length === 0 ? (
              <p className="text-muted-foreground text-center p-8">
                No playlists found. Sync playlists from Spotify first.
              </p>
            ) : (
              <Table>
                <TableHeader>
                  {playlistTable.getHeaderGroups().map((headerGroup) => (
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
                  {playlistTable.getRowModel().rows.map((row) => (
                    <TableRow
                      key={row.id}
                      className={
                        selectedPlaylistId === row.original.id
                          ? "bg-muted"
                          : "cursor-pointer"
                      }
                      onClick={() => setSelectedPlaylistId(row.original.id)}
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
                  ))}
                </TableBody>
              </Table>
            )}
          </CardContent>
        </Card>
      )}

      {/* Operations */}
      <Card>
        <CardHeader>
          <CardTitle>Operations</CardTitle>
          <CardDescription>
            Perform matching and syncing operations
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <Button
            onClick={() => matchTracks.mutate()}
            disabled={matchTracks.isPending}
          >
            {matchTracks.isPending ? (
              <>
                <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                Matching...
              </>
            ) : (
              <>
                <Music className="mr-2 h-4 w-4" />
                Match Existing Tracks
              </>
            )}
          </Button>

          {selectedAccountId && (
            <syncForm.AppForm>
              <FieldSet className="space-y-4">
                <syncForm.AppField name="spotifyPlaylistId">
                  {() => (
                    <FormFieldContainer label="Playlist">
                      <Select
                        value={syncForm.state.values.spotifyPlaylistId}
                        onValueChange={(value) => {
                          syncForm.setFieldValue("spotifyPlaylistId", value);
                        }}
                      >
                        <SelectTrigger>
                          <SelectValue placeholder="Select a playlist" />
                        </SelectTrigger>
                        <SelectContent>
                          {playlistsData?.playlists.map((playlist) => (
                            <SelectItem
                              key={playlist.id}
                              value={playlist.id.toString()}
                            >
                              {playlist.name}
                            </SelectItem>
                          ))}
                        </SelectContent>
                      </Select>
                    </FormFieldContainer>
                  )}
                </syncForm.AppField>

                <syncForm.AppField name="localPlaylistName">
                  {() => (
                    <FormFieldContainer label="Local Playlist Name *">
                      <FormTextField placeholder="Enter local playlist name" />
                    </FormFieldContainer>
                  )}
                </syncForm.AppField>

                <syncForm.FormSubmitButton
                  label="Sync Playlist to Local Library"
                  loadingLabel="Syncing..."
                  errorLabel="Sync failed"
                  icon={RefreshCw}
                />
              </FieldSet>
            </syncForm.AppForm>
          )}
        </CardContent>
      </Card>

      {/* Sync State */}
      {selectedPlaylistId && syncState && (
        <Card>
          <CardHeader>
            <CardTitle>Sync State</CardTitle>
            <CardDescription>
              Current sync status for selected playlist
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="flex items-center gap-2">
              <span className="font-medium">Status:</span>
              {getStatusBadge(syncState.syncStatus)}
            </div>
            <div className="grid grid-cols-2 gap-4">
              <div>
                <span className="text-muted-foreground text-sm">
                  Tracks Downloaded:
                </span>
                <p className="text-lg font-semibold">
                  {syncState.tracksDownloaded}
                </p>
              </div>
              <div>
                <span className="text-muted-foreground text-sm">
                  Tracks Failed:
                </span>
                <p className="text-lg font-semibold">
                  {syncState.tracksFailed}
                </p>
              </div>
            </div>
            {syncState.errorLog && (
              <div>
                <span className="text-muted-foreground text-sm">
                  Error Log:
                </span>
                <pre className="mt-2 rounded bg-destructive/10 p-2 text-sm">
                  {syncState.errorLog}
                </pre>
              </div>
            )}
          </CardContent>
        </Card>
      )}

      {/* Download Failures */}
      {selectedPlaylistId &&
        failuresData &&
        failuresData.failures.length > 0 && (
          <Card>
            <CardHeader>
              <CardTitle>Download Failures</CardTitle>
              <CardDescription>
                Tracks that failed to download during sync
              </CardDescription>
            </CardHeader>
            <CardContent>
              <Table>
                <TableHeader>
                  {failureTable.getHeaderGroups().map((headerGroup) => (
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
                  {failureTable.getRowModel().rows.map((row) => (
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
            </CardContent>
          </Card>
        )}

      {selectedAccountId &&
        !accountsLoading &&
        accountsData?.accounts.length === 0 && (
          <Card>
            <CardContent className="flex flex-col items-center justify-center p-8">
              <AlertCircle className="mb-4 h-12 w-12 text-muted-foreground" />
              <p className="text-muted-foreground text-center">
                No Spotify accounts found. Please authenticate a Spotify account
                first.
              </p>
            </CardContent>
          </Card>
        )}
    </div>
  );
}
