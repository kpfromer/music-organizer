import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import {
  type ColumnDef,
  flexRender,
  getCoreRowModel,
  useReactTable,
} from "@tanstack/react-table";
import { formatDistanceToNow, parseISO } from "date-fns";
import {
  ChevronLeft,
  ChevronRight,
  Loader2,
  RefreshCw,
  Trash2,
} from "lucide-react";
import { useState } from "react";
import { Badge } from "@/components/ui/badge";
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

const WishlistItemsQuery = graphql(`
  query WishlistItems($page: Int, $pageSize: Int, $status: String) {
    wishlistItems(page: $page, pageSize: $pageSize, status: $status) {
      items {
        id
        spotifyTrackId
        status
        errorReason
        attemptsCount
        lastAttemptAt
        nextRetryAt
        createdAt
        updatedAt
        trackTitle
        trackArtists
        trackAlbum
      }
      totalCount
      page
      pageSize
    }
  }
`);

const WishlistStatsQuery = graphql(`
  query WishlistStats {
    wishlistStats {
      pending
      searching
      downloading
      importing
      completed
      failed
    }
  }
`);

const RemoveFromWishlistMutation = graphql(`
  mutation RemoveFromWishlist($id: Int!) {
    removeFromWishlist(id: $id)
  }
`);

const RetryWishlistItemMutation = graphql(`
  mutation RetryWishlistItem($id: Int!) {
    retryWishlistItem(id: $id) {
      id
      status
    }
  }
`);

type WishlistItemRow = {
  id: number;
  spotifyTrackId: string;
  status: string;
  errorReason?: string | null;
  attemptsCount: number;
  lastAttemptAt: Date | null;
  createdAt: Date;
  trackTitle: string;
  trackArtists: string[];
  trackAlbum: string;
};

function statusBadge(status: string) {
  switch (status) {
    case "pending":
      return (
        <Badge variant="secondary" className="bg-gray-100 text-gray-700">
          Pending
        </Badge>
      );
    case "searching":
      return (
        <Badge variant="secondary" className="bg-blue-100 text-blue-700">
          <Loader2 className="mr-1 h-3 w-3 animate-spin" />
          Searching
        </Badge>
      );
    case "downloading":
      return (
        <Badge variant="secondary" className="bg-blue-100 text-blue-700">
          <Loader2 className="mr-1 h-3 w-3 animate-spin" />
          Downloading
        </Badge>
      );
    case "importing":
      return (
        <Badge variant="secondary" className="bg-blue-100 text-blue-700">
          <Loader2 className="mr-1 h-3 w-3 animate-spin" />
          Importing
        </Badge>
      );
    case "completed":
      return (
        <Badge variant="secondary" className="bg-green-100 text-green-700">
          Completed
        </Badge>
      );
    case "failed":
      return <Badge variant="destructive">Failed</Badge>;
    default:
      return <Badge variant="outline">{status}</Badge>;
  }
}

export function Wishlist() {
  const queryClient = useQueryClient();
  const [page, setPage] = useState(1);
  const [pageSize, setPageSize] = useState(25);
  const [statusFilter, setStatusFilter] = useState<string | undefined>(
    undefined,
  );

  const hasInProgress = (stats: {
    pending: number;
    searching: number;
    downloading: number;
    importing: number;
  }) =>
    stats.pending > 0 ||
    stats.searching > 0 ||
    stats.downloading > 0 ||
    stats.importing > 0;

  const { data: statsData } = useQuery({
    queryKey: ["wishlistStats"],
    queryFn: () => execute(WishlistStatsQuery),
    refetchInterval: (query) => {
      const stats = query.state.data?.wishlistStats;
      return stats && hasInProgress(stats) ? 5000 : false;
    },
  });

  const { data, isLoading } = useQuery({
    queryKey: ["wishlistItems", page, pageSize, statusFilter],
    queryFn: async () => {
      const result = await execute(WishlistItemsQuery, {
        page,
        pageSize,
        status: statusFilter,
      });
      return {
        ...result.wishlistItems,
        items: result.wishlistItems.items.map((item) => ({
          ...item,
          lastAttemptAt: item.lastAttemptAt
            ? parseISO(item.lastAttemptAt)
            : null,
          createdAt: parseISO(item.createdAt),
        })),
      };
    },
    refetchInterval: () => {
      const stats = statsData?.wishlistStats;
      return stats && hasInProgress(stats) ? 5000 : false;
    },
  });

  const removeItem = useMutation({
    mutationFn: (id: number) => execute(RemoveFromWishlistMutation, { id }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["wishlistItems"] });
      queryClient.invalidateQueries({ queryKey: ["wishlistStats"] });
    },
  });

  const retryItem = useMutation({
    mutationFn: (id: number) => execute(RetryWishlistItemMutation, { id }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["wishlistItems"] });
      queryClient.invalidateQueries({ queryKey: ["wishlistStats"] });
    },
  });

  const columns: ColumnDef<WishlistItemRow>[] = [
    {
      accessorKey: "trackTitle",
      header: "Track",
      cell: ({ row }) => (
        <div>
          <div className="font-medium">{row.original.trackTitle}</div>
          <div className="text-sm text-muted-foreground">
            {row.original.trackArtists.join(", ")}
          </div>
        </div>
      ),
    },
    {
      accessorKey: "trackAlbum",
      header: "Album",
    },
    {
      accessorKey: "status",
      header: "Status",
      cell: ({ row }) => (
        <div className="space-y-1">
          {statusBadge(row.original.status)}
          {row.original.status === "failed" && row.original.errorReason && (
            <div className="text-xs text-destructive">
              {row.original.errorReason}
            </div>
          )}
        </div>
      ),
    },
    {
      accessorKey: "attemptsCount",
      header: "Attempts",
      cell: ({ row }) => (
        <span className="text-muted-foreground">
          {row.original.attemptsCount}
        </span>
      ),
    },
    {
      accessorKey: "lastAttemptAt",
      header: "Last Attempt",
      cell: ({ row }) => {
        const date = row.original.lastAttemptAt;
        return (
          <span className="text-muted-foreground">
            {date ? formatDistanceToNow(date, { addSuffix: true }) : "-"}
          </span>
        );
      },
    },
    {
      id: "actions",
      header: "Actions",
      cell: ({ row }) => (
        <div className="flex items-center gap-1">
          {row.original.status === "failed" && (
            <Button
              size="sm"
              variant="outline"
              onClick={() => retryItem.mutate(row.original.id)}
              disabled={retryItem.isPending}
            >
              <RefreshCw className="mr-1 h-3 w-3" />
              Retry
            </Button>
          )}
          <Button
            size="sm"
            variant="ghost"
            onClick={() => {
              if (window.confirm("Remove this item from the wishlist?")) {
                removeItem.mutate(row.original.id);
              }
            }}
            disabled={removeItem.isPending}
          >
            <Trash2 className="h-3 w-3" />
          </Button>
        </div>
      ),
    },
  ];

  const tableData: WishlistItemRow[] = data?.items ?? [];

  const table = useReactTable({
    data: tableData,
    columns,
    getCoreRowModel: getCoreRowModel(),
    manualPagination: true,
  });

  const totalPages = data ? Math.ceil(data.totalCount / data.pageSize) : 0;
  const stats = statsData?.wishlistStats;

  if (isLoading) {
    return (
      <div className="container mx-auto p-8">
        <div className="text-muted-foreground">Loading wishlist...</div>
      </div>
    );
  }

  return (
    <div className="container mx-auto p-8">
      <div className="mb-6">
        <h1 className="text-2xl font-bold mb-4">Wishlist</h1>

        {stats && (
          <div className="flex flex-wrap gap-3">
            <Badge variant="secondary" className="bg-gray-100 text-gray-700">
              Pending: {stats.pending}
            </Badge>
            <Badge variant="secondary" className="bg-blue-100 text-blue-700">
              Searching: {stats.searching}
            </Badge>
            <Badge variant="secondary" className="bg-blue-100 text-blue-700">
              Downloading: {stats.downloading}
            </Badge>
            <Badge variant="secondary" className="bg-blue-100 text-blue-700">
              Importing: {stats.importing}
            </Badge>
            <Badge variant="secondary" className="bg-green-100 text-green-700">
              Completed: {stats.completed}
            </Badge>
            <Badge variant="destructive">Failed: {stats.failed}</Badge>
          </div>
        )}
      </div>

      <div className="mb-4 flex items-center justify-between">
        <div className="flex items-center gap-2">
          <span className="text-sm text-muted-foreground">Status:</span>
          <Select
            value={statusFilter ?? "all"}
            onValueChange={(value) => {
              setStatusFilter(value === "all" ? undefined : value);
              setPage(1);
            }}
          >
            <SelectTrigger className="w-40">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="all">All</SelectItem>
              <SelectItem value="pending">Pending</SelectItem>
              <SelectItem value="searching">Searching</SelectItem>
              <SelectItem value="downloading">Downloading</SelectItem>
              <SelectItem value="importing">Importing</SelectItem>
              <SelectItem value="completed">Completed</SelectItem>
              <SelectItem value="failed">Failed</SelectItem>
            </SelectContent>
          </Select>
        </div>
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
              ))
            ) : (
              <TableRow>
                <TableCell
                  colSpan={columns.length}
                  className="h-24 text-center"
                >
                  No wishlist items found.
                </TableCell>
              </TableRow>
            )}
          </TableBody>
        </Table>
      </div>

      <div className="mt-4 flex items-center justify-between">
        <div className="text-sm text-muted-foreground">
          Showing {data?.totalCount ? (page - 1) * pageSize + 1 : 0} to{" "}
          {Math.min(page * pageSize, data?.totalCount ?? 0)} of{" "}
          {data?.totalCount ?? 0} items
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
