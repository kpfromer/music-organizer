import { useMutation } from "@tanstack/react-query";
import {
  type ColumnDef,
  flexRender,
  getCoreRowModel,
  useReactTable,
} from "@tanstack/react-table";
import { Download as DownloadIcon, Loader2, Search } from "lucide-react";
import { useState } from "react";
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
  MutationDownloadSoulseekFileArgs,
  MutationSearchSoulseekArgs,
  SoulSeekSearchResult,
} from "@/graphql/graphql";
import { execute } from "@/lib/execute-graphql";
import {
  formatAttributes,
  formatFileSize,
  parseArtistsInput,
} from "@/lib/formatters";

const SearchSoulseekMutation = graphql(`
	mutation SearchSoulseek($trackTitle: String!, $albumName: String, $artists: [String!], $duration: Int) {
		searchSoulseek(
			trackTitle: $trackTitle
			albumName: $albumName
			artists: $artists
			duration: $duration
		) {
			username
			token
			filename
			size
			avgSpeed
			queueLength
			slotsFree
			attributes {
				attribute
				value
			}
		}
	}
`);

const DownloadSoulseekFileMutation = graphql(`
	mutation DownloadFromSoulseek($username: String!, $filename: String!, $size: Int!, $token: String!) {
		downloadSoulseekFile(
			username: $username
			filename: $filename
			size: $size
			token: $token
		) {
			success
			message
		}
	}
`);

type SearchFormData = {
  trackTitle: string;
  albumName: string;
  artists: string;
  duration: string;
};

export function Download() {
  const [downloadingIds, setDownloadingIds] = useState<Set<string>>(new Set());

  const searchSoulseek = useMutation({
    mutationFn: async (variables: MutationSearchSoulseekArgs) =>
      execute(SearchSoulseekMutation, variables),
  });
  const searchResults = searchSoulseek.data?.searchSoulseek ?? [];

  const downloadSoulseekFile = useMutation({
    mutationFn: async (variables: MutationDownloadSoulseekFileArgs) =>
      execute(DownloadSoulseekFileMutation, variables),
  });

  const form = useAppForm({
    defaultValues: {
      trackTitle: "",
      albumName: "",
      artists: "",
      duration: "",
    },
    onSubmit: async ({ value }: { value: SearchFormData }) => {
      console.log("onSubmit", value);
      const artistsArray =
        value.artists.trim() !== ""
          ? parseArtistsInput(value.artists)
          : undefined;
      const duration =
        value.duration.trim() !== "" ? parseInt(value.duration, 10) : undefined;

      const variables: MutationSearchSoulseekArgs = {
        trackTitle: value.trackTitle,
        albumName: value.albumName.trim() !== "" ? value.albumName : undefined,
        artists: artistsArray,
        duration: duration,
      };

      await searchSoulseek.mutateAsync(variables);
    },
  });

  const handleDownload = async (result: SoulSeekSearchResult) => {
    const downloadId = `${result.username}-${result.filename}`;
    setDownloadingIds((prev) => new Set(prev).add(downloadId));

    try {
      const variables: MutationDownloadSoulseekFileArgs = {
        username: result.username,
        filename: result.filename,
        size: result.size,
        token: result.token,
      };

      await downloadSoulseekFile.mutateAsync(variables);
    } catch (error) {
      console.error("Download failed:", error);
    } finally {
      setDownloadingIds((prev) => {
        const next = new Set(prev);
        next.delete(downloadId);
        return next;
      });
    }
  };

  const columns: ColumnDef<SoulSeekSearchResult>[] = [
    {
      accessorKey: "filename",
      header: "Filename",
      cell: ({ row }) => {
        return (
          <div
            className="font-medium max-w-md truncate"
            title={row.original.filename}
          >
            {row.original.filename}
          </div>
        );
      },
    },
    {
      accessorKey: "size",
      header: "Size",
      cell: ({ row }) => {
        return (
          <div className="text-muted-foreground">
            {formatFileSize(row.original.size)}
          </div>
        );
      },
    },
    {
      accessorKey: "attributes",
      header: "Attributes",
      cell: ({ row }) => {
        return (
          <div className="text-muted-foreground text-sm">
            {formatAttributes(row.original.attributes)}
          </div>
        );
      },
    },
    {
      id: "download",
      header: "",
      cell: ({ row }) => {
        const result = row.original;
        const downloadId = `${result.username}-${result.filename}`;
        const isDownloading = downloadingIds.has(downloadId);

        return (
          <div className="flex justify-end">
            <Button
              variant="outline"
              size="sm"
              onClick={() => handleDownload(result)}
              disabled={isDownloading}
            >
              {isDownloading ? (
                <>
                  <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                  Downloading...
                </>
              ) : (
                <>
                  <DownloadIcon className="mr-2 h-4 w-4" />
                  Download
                </>
              )}
            </Button>
          </div>
        );
      },
    },
  ];

  const table = useReactTable({
    data: searchResults ?? [],
    columns,
    getCoreRowModel: getCoreRowModel(),
  });

  return (
    <div className="container mx-auto p-8">
      <div className="mb-8">
        <h1 className="text-3xl font-bold mb-2">Search & Download</h1>
        <p className="text-muted-foreground">
          Search for tracks on Soulseek and download them directly
        </p>
      </div>

      <Card className="mb-8">
        <CardHeader>
          <CardTitle>Search</CardTitle>
        </CardHeader>
        <CardContent>
          <form.AppForm>
            <FieldSet>
              <form.AppField
                name="trackTitle"
                validators={{
                  onChange: ({ value }: { value: string }) =>
                    value.trim() === "" ? "Track title is required" : undefined,
                }}
              >
                {() => (
                  <FormFieldContainer label="Track Title *">
                    <FormTextField placeholder="Enter track title" />
                  </FormFieldContainer>
                )}
              </form.AppField>

              <form.AppField name="albumName">
                {() => (
                  <FormFieldContainer label="Album Name">
                    <FormTextField placeholder="Enter album name (optional)" />
                  </FormFieldContainer>
                )}
              </form.AppField>

              <form.AppField name="artists">
                {() => (
                  <FormFieldContainer label="Artists">
                    <FormTextField placeholder="Enter artists, comma-separated (optional)" />
                  </FormFieldContainer>
                )}
              </form.AppField>

              <form.AppField name="duration">
                {() => (
                  <FormFieldContainer label="Duration (seconds)">
                    <FormTextField
                      type="number"
                      placeholder="Enter duration in seconds (optional)"
                    />
                  </FormFieldContainer>
                )}
              </form.AppField>
              <form.FormSubmitButton
                label="Search"
                loadingLabel="Searching..."
                errorLabel="Search failed. Please try again."
                icon={Search}
              />
            </FieldSet>
          </form.AppForm>
        </CardContent>
      </Card>

      {searchResults !== null && (
        <Card>
          <CardHeader>
            <CardTitle>
              Results {searchResults.length > 0 && `(${searchResults.length})`}
            </CardTitle>
          </CardHeader>
          <CardContent>
            {searchResults.length === 0 ? (
              <div className="text-center py-8 text-muted-foreground">
                No results found. Try adjusting your search criteria.
              </div>
            ) : (
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
                          No results found.
                        </TableCell>
                      </TableRow>
                    )}
                  </TableBody>
                </Table>
              </div>
            )}
          </CardContent>
        </Card>
      )}
    </div>
  );
}
