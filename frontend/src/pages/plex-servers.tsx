import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import {
  type ColumnDef,
  flexRender,
  getCoreRowModel,
  useReactTable,
} from "@tanstack/react-table";
import { formatDistanceToNow, parseISO } from "date-fns";
import {
  CheckCircle,
  ExternalLink,
  Loader2,
  Plus,
  RefreshCw,
  XCircle,
} from "lucide-react";
import { useEffect, useState } from "react";
import { z } from "zod";
import { FormFieldContainer } from "@/components/form/FormFieldContainer";
import { FormTextField } from "@/components/form/FormTextField";
import { useAppForm } from "@/components/form/form-hooks";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { FieldSet } from "@/components/ui/field";
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
  PlexServer as GraphQLPlexServer,
  MutationAuthenticatePlexServerArgs,
  MutationCreatePlexServerArgs,
  MutationRefreshMusicLibraryArgs,
} from "@/graphql/graphql";
import { execute } from "@/lib/execute-graphql";

const PlexServersQuery = graphql(`
  query PlexServers {
    plexServers {
      id
      name
      serverUrl
      hasAccessToken
      createdAt
      updatedAt
    }
  }
`);

const CreatePlexServerMutation = graphql(`
  mutation CreatePlexServer($name: String!, $serverUrl: String!) {
    createPlexServer(name: $name, serverUrl: $serverUrl) {
      id
      name
      serverUrl
      hasAccessToken
      createdAt
      updatedAt
    }
  }
`);

const AuthenticatePlexServerMutation = graphql(`
  mutation AuthenticatePlexServer($serverId: Int!) {
    authenticatePlexServer(serverId: $serverId) {
      authUrl
      pinId
    }
  }
`);

const RefreshMusicLibraryMutation = graphql(`
  mutation RefreshMusicLibrary($plexServerId: Int!) {
    refreshMusicLibrary(plexServerId: $plexServerId) {
      success
      message
      sectionId
    }
  }
`);

const MusicLibraryScanStatusQuery = graphql(`
  query MusicLibraryScanStatus($plexServerId: Int!) {
    musicLibraryScanStatus(plexServerId: $plexServerId) {
      isScanning
      progress
      title
      subtitle
    }
  }
`);

// Transform the generated PlexServer type to have createdAt/updatedAt as Date instead of DateTime scalar
type PlexServer = Omit<GraphQLPlexServer, "createdAt" | "updatedAt"> & {
  createdAt: Date;
  updatedAt: Date;
};

const createServerSchema = z.object({
  name: z.string().min(1, "Name is required"),
  serverUrl: z.string().url("Invalid URL format"),
});

type CreateServerFormData = z.infer<typeof createServerSchema>;

export function PlexServers() {
  const queryClient = useQueryClient();
  const [showCreateForm, setShowCreateForm] = useState(false);
  const [authenticatingServerId, setAuthenticatingServerId] = useState<
    number | null
  >(null);
  const [refreshingServerId, setRefreshingServerId] = useState<number | null>(
    null,
  );

  const { data, isLoading } = useQuery({
    queryKey: ["plexServers"],
    queryFn: async () => {
      const result = await execute(PlexServersQuery);
      return {
        servers: result.plexServers.map((server) => ({
          ...server,
          createdAt: parseISO(server.createdAt),
          updatedAt: parseISO(server.updatedAt),
        })),
      };
    },
  });

  const createServer = useMutation({
    mutationFn: async (variables: MutationCreatePlexServerArgs) =>
      execute(CreatePlexServerMutation, variables),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["plexServers"] });
      setShowCreateForm(false);
    },
  });

  const authenticateServer = useMutation({
    mutationFn: async (variables: MutationAuthenticatePlexServerArgs) =>
      execute(AuthenticatePlexServerMutation, variables),
    onSuccess: (data, variables) => {
      // Store pinId and serverId in localStorage for callback page
      localStorage.setItem(
        `plex_auth_${data.authenticatePlexServer.pinId}`,
        JSON.stringify({
          serverId: variables.serverId,
          pinId: data.authenticatePlexServer.pinId,
        }),
      );
      // Navigate to auth URL
      window.location.href = data.authenticatePlexServer.authUrl;
    },
  });

  const form = useAppForm({
    defaultValues: {
      name: "",
      serverUrl: "",
    },
    validators: {
      onSubmit: createServerSchema,
    },
    onSubmit: async ({ value }: { value: CreateServerFormData }) => {
      await createServer.mutateAsync({
        name: value.name,
        serverUrl: value.serverUrl,
      });
    },
  });

  const handleAuthenticate = async (serverId: number) => {
    setAuthenticatingServerId(serverId);
    try {
      await authenticateServer.mutateAsync({ serverId });
    } catch (error) {
      console.error("Failed to authenticate server:", error);
      setAuthenticatingServerId(null);
    }
  };

  const refreshMusicLibrary = useMutation({
    mutationFn: async (variables: MutationRefreshMusicLibraryArgs) =>
      execute(RefreshMusicLibraryMutation, variables),
    onSuccess: (_, variables) => {
      setRefreshingServerId(variables.plexServerId);
    },
  });

  // Poll for scan status when a refresh is in progress
  const scanStatus = useQuery({
    queryKey: ["musicLibraryScanStatus", refreshingServerId],
    queryFn: async () => {
      if (!refreshingServerId) return null;
      const result = await execute(MusicLibraryScanStatusQuery, {
        plexServerId: refreshingServerId,
      });
      return result.musicLibraryScanStatus;
    },
    enabled: refreshingServerId !== null,
    refetchInterval: (query) => {
      const data = query.state.data;
      // Poll every 1 second if scanning, stop if not scanning
      return data?.isScanning ? 1000 : false;
    },
  });

  // Stop polling when scan completes
  useEffect(() => {
    if (
      refreshingServerId !== null &&
      scanStatus.data &&
      !scanStatus.data.isScanning
    ) {
      const timer = setTimeout(() => {
        setRefreshingServerId(null);
      }, 2000);
      return () => clearTimeout(timer);
    }
  }, [refreshingServerId, scanStatus.data]);

  const handleRefreshLibrary = async (serverId: number) => {
    await refreshMusicLibrary.mutateAsync({ plexServerId: serverId });
  };

  const columns: ColumnDef<PlexServer>[] = [
    {
      accessorKey: "name",
      header: "Name",
      cell: ({ row }) => {
        const server = row.original;
        return <div className="font-medium">{server.name}</div>;
      },
    },
    {
      accessorKey: "serverUrl",
      header: "Server URL",
      cell: ({ row }) => {
        const url = row.getValue("serverUrl") as string;
        return (
          <div className="text-muted-foreground font-mono text-sm">{url}</div>
        );
      },
    },
    {
      accessorKey: "hasAccessToken",
      header: "Status",
      cell: ({ row }) => {
        const hasToken = row.getValue("hasAccessToken") as boolean;
        return (
          <div className="flex items-center gap-2">
            {hasToken ? (
              <>
                <CheckCircle className="h-4 w-4 text-green-500" />
                <span className="text-sm text-green-700 dark:text-green-400">
                  Authenticated
                </span>
              </>
            ) : (
              <>
                <XCircle className="h-4 w-4 text-red-500" />
                <span className="text-sm text-red-700 dark:text-red-400">
                  Not Authenticated
                </span>
              </>
            )}
          </div>
        );
      },
    },
    {
      accessorKey: "createdAt",
      header: "Created",
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
      accessorKey: "updatedAt",
      header: "Updated",
      cell: ({ row }) => {
        const date = row.getValue("updatedAt") as Date;
        return (
          <div className="text-muted-foreground">
            {formatDistanceToNow(date, { addSuffix: true })}
          </div>
        );
      },
    },
    {
      id: "actions",
      header: "Actions",
      cell: ({ row }) => {
        const server = row.original;
        const isAuthenticating = authenticatingServerId === server.id;
        const isAuthenticated = server.hasAccessToken;
        const isRefreshing = refreshingServerId === server.id;
        const status = isRefreshing ? scanStatus.data : null;

        return (
          <div className="flex items-center gap-2">
            {!isAuthenticated && (
              <Button
                variant="outline"
                size="sm"
                onClick={() => handleAuthenticate(server.id)}
                disabled={isAuthenticating || authenticateServer.isPending}
              >
                {isAuthenticating ? (
                  <>
                    <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                    Authenticating...
                  </>
                ) : (
                  <>
                    <ExternalLink className="mr-2 h-4 w-4" />
                    Authenticate
                  </>
                )}
              </Button>
            )}
            {isAuthenticated && (
              <Button
                variant="outline"
                size="sm"
                onClick={() => handleRefreshLibrary(server.id)}
                disabled={isRefreshing || refreshMusicLibrary.isPending}
              >
                {isRefreshing ? (
                  <>
                    <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                    {status?.isScanning
                      ? status.progress !== null
                        ? `Scanning ${Math.round((status.progress ?? 0) * 100)}%`
                        : "Scanning..."
                      : "Starting..."}
                  </>
                ) : (
                  <>
                    <RefreshCw className="mr-2 h-4 w-4" />
                    Refresh Library
                  </>
                )}
              </Button>
            )}
            {isRefreshing && status?.isScanning && (
              <div className="text-sm text-muted-foreground">
                {status.subtitle && <div>{status.subtitle}</div>}
              </div>
            )}
          </div>
        );
      },
    },
  ];

  const table = useReactTable({
    data: data?.servers ?? [],
    columns,
    getCoreRowModel: getCoreRowModel(),
  });

  if (isLoading) {
    return (
      <div className="container mx-auto p-8">
        <div className="text-muted-foreground">Loading Plex servers...</div>
      </div>
    );
  }

  return (
    <div className="container mx-auto p-8">
      <div className="mb-4 flex items-center justify-between">
        <h1 className="text-2xl font-bold">Plex Servers</h1>
        <Button
          onClick={() => setShowCreateForm(!showCreateForm)}
          variant="default"
        >
          <Plus className="mr-2 h-4 w-4" />
          Add Server
        </Button>
      </div>

      {showCreateForm && (
        <Card className="mb-6">
          <CardHeader>
            <CardTitle>Add New Plex Server</CardTitle>
          </CardHeader>
          <CardContent>
            <form.AppForm>
              <FieldSet>
                <form.AppField name="name">
                  {() => (
                    <FormFieldContainer label="Server Name *">
                      <FormTextField placeholder="Enter server name" />
                    </FormFieldContainer>
                  )}
                </form.AppField>

                <form.AppField name="serverUrl">
                  {() => (
                    <FormFieldContainer label="Server URL *">
                      <FormTextField
                        type="url"
                        placeholder="https://example.com:32400"
                      />
                    </FormFieldContainer>
                  )}
                </form.AppField>

                <div className="flex gap-2">
                  <form.FormSubmitButton
                    label="Create Server"
                    loadingLabel="Creating..."
                    errorLabel="Failed to create server"
                  />
                  <Button
                    type="button"
                    variant="outline"
                    onClick={() => {
                      setShowCreateForm(false);
                      form.reset();
                    }}
                  >
                    Cancel
                  </Button>
                </div>
              </FieldSet>
            </form.AppForm>
          </CardContent>
        </Card>
      )}

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
                  {showCreateForm
                    ? "No servers found. Create your first server above."
                    : "No Plex servers found. Click 'Add Server' to create one."}
                </TableCell>
              </TableRow>
            )}
          </TableBody>
        </Table>
      </div>
    </div>
  );
}
