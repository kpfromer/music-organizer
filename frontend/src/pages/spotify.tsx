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
  Plus,
  RefreshCw,
} from "lucide-react";
import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { z } from "zod";
import { FormFieldContainer } from "@/components/form/FormFieldContainer";
import { FormTextField } from "@/components/form/FormTextField";
import { useAppForm } from "@/components/form/form-hooks";
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
  SpotifyAccount,
  SpotifyPlaylist,
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

const SyncSpotifyPlaylistToLocalMutation = graphql(`
  mutation SyncSpotifyPlaylistToLocal(
    $spotifyPlaylistId: Int!
    $localPlaylistName: String!
  ) {
    syncSpotifyPlaylistToLocal(
      spotifyPlaylistId: $spotifyPlaylistId
      localPlaylistName: $localPlaylistName
    ) {
      totalTracks
      matchedTracks
      unmatchedTracks
      newMatchesFound
    }
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

interface SyncResult {
  totalTracks: number;
  matchedTracks: number;
  unmatchedTracks: number;
  newMatchesFound: number;
}

export function Spotify() {
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const [selectedAccountId, setSelectedAccountId] = useState<number | null>(
    null,
  );
  const [lastSyncResult, setLastSyncResult] = useState<SyncResult | null>(null);

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
    mutationFn: async (variables: {
      spotifyPlaylistId: number;
      localPlaylistName: string;
    }) => execute(SyncSpotifyPlaylistToLocalMutation, variables),
    onSuccess: (data) => {
      setLastSyncResult(data.syncSpotifyPlaylistToLocal);
      queryClient.invalidateQueries({ queryKey: ["playlists"] });
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
      await syncPlaylistToLocal.mutateAsync({
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
          <div className="flex gap-2 items-center">
            <Select
              value={selectedAccountId?.toString() || ""}
              onValueChange={(value) => {
                setSelectedAccountId(value ? parseInt(value, 10) : null);
              }}
            >
              <SelectTrigger className="flex-1 max-w-md">
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
            <Button
              onClick={() => navigate("/spotify/login")}
              variant="outline"
            >
              <Plus className="mr-2 h-4 w-4" />
              Add Account
            </Button>
          </div>

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
              Spotify playlists for the selected account
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
                  label="Sync to Local"
                  loadingLabel="Syncing..."
                  errorLabel="Sync failed"
                  icon={RefreshCw}
                />
              </FieldSet>
            </syncForm.AppForm>
          )}

          {/* Sync Result */}
          {lastSyncResult && (
            <div className="rounded-lg border p-4 space-y-2">
              <div className="flex items-center gap-2">
                <CheckCircle className="h-4 w-4 text-green-500" />
                <span className="font-medium">Sync Complete</span>
              </div>
              <div className="grid grid-cols-2 gap-4 text-sm">
                <div>
                  <span className="text-muted-foreground">Total Tracks:</span>{" "}
                  {lastSyncResult.totalTracks}
                </div>
                <div>
                  <span className="text-muted-foreground">Matched:</span>{" "}
                  <span className="text-green-600">
                    {lastSyncResult.matchedTracks}
                  </span>
                </div>
                <div>
                  <span className="text-muted-foreground">Unmatched:</span>{" "}
                  <span className="text-orange-600">
                    {lastSyncResult.unmatchedTracks}
                  </span>
                </div>
                <div>
                  <span className="text-muted-foreground">New Matches:</span>{" "}
                  <span className="text-blue-600">
                    {lastSyncResult.newMatchesFound}
                  </span>
                </div>
              </div>
            </div>
          )}
        </CardContent>
      </Card>

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
