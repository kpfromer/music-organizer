import { useForm } from "@tanstack/react-form";
import { useMutation } from "@tanstack/react-query";
import {
	type ColumnDef,
	flexRender,
	getCoreRowModel,
	useReactTable,
} from "@tanstack/react-table";
import { Download as DownloadIcon, Loader2, Search } from "lucide-react";
import { useState } from "react";
import { FormField } from "@/components/form/form-field";
import { FormInput } from "@/components/form/form-input";
import { Button } from "@/components/ui/button";
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
	const [searchResults, setSearchResults] =
		useState<Array<SoulSeekSearchResult> | null>(null);
	const [downloadingIds, setDownloadingIds] = useState<Set<string>>(new Set());

	const searchSoulseek = useMutation({
		mutationFn: async (variables: MutationSearchSoulseekArgs) =>
			execute(SearchSoulseekMutation, variables),
	});

	const downloadSoulseekFile = useMutation({
		mutationFn: async (variables: MutationDownloadSoulseekFileArgs) =>
			execute(DownloadSoulseekFileMutation, variables),
	});

	const form = useForm({
		defaultValues: {
			trackTitle: "",
			albumName: "",
			artists: "",
			duration: "",
		},
		onSubmit: async ({ value }: { value: SearchFormData }) => {
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

			try {
				const result = await searchSoulseek.mutateAsync(variables);
				setSearchResults(result.searchSoulseek);
			} catch (error) {
				console.error("Search failed:", error);
			}
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
					<form
						onSubmit={(e) => {
							e.preventDefault();
							form.handleSubmit();
						}}
						className="space-y-4"
					>
						<form.Field
							name="trackTitle"
							validators={{
								onChange: ({ value }: { value: string }) =>
									value.trim() === "" ? "Track title is required" : undefined,
							}}
						>
							{(field) => (
								<FormField field={field} label="Track Title *">
									{(f) => (
										<FormInput field={f} placeholder="Enter track title" />
									)}
								</FormField>
							)}
						</form.Field>

						<form.Field name="albumName">
							{(field) => (
								<FormField field={field} label="Album Name">
									{(f) => (
										<FormInput
											field={f}
											placeholder="Enter album name (optional)"
										/>
									)}
								</FormField>
							)}
						</form.Field>

						<form.Field name="artists">
							{(field) => (
								<FormField field={field} label="Artists">
									{(f) => (
										<FormInput
											field={f}
											placeholder="Enter artists, comma-separated (optional)"
										/>
									)}
								</FormField>
							)}
						</form.Field>

						<form.Field name="duration">
							{(field) => (
								<FormField field={field} label="Duration (seconds)">
									{(f) => (
										<FormInput
											field={f}
											type="number"
											placeholder="Enter duration in seconds (optional)"
										/>
									)}
								</FormField>
							)}
						</form.Field>

						<Button
							type="submit"
							disabled={searchSoulseek.isPending}
							className="w-full"
						>
							{searchSoulseek.isPending ? (
								<>
									<Loader2 className="mr-2 h-4 w-4 animate-spin" />
									Searching...
								</>
							) : (
								<>
									<Search className="mr-2 h-4 w-4" />
									Search
								</>
							)}
						</Button>

						{searchSoulseek.isError && (
							<div className="text-sm text-destructive">
								Search failed. Please try again.
							</div>
						)}
					</form>
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
