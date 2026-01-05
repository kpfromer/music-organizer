import { useMutation, useQuery } from "@tanstack/react-query";
import {
	type ColumnDef,
	flexRender,
	getCoreRowModel,
	useReactTable,
} from "@tanstack/react-table";
import {
	Check,
	Download as DownloadIcon,
	Loader2,
	Search,
	X,
} from "lucide-react";
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
	MutationSearchSoulseekArgs,
	SoulSeekSearchResult,
} from "@/graphql/graphql";
import {
	type DownloadFileInput,
	downloadFileQuery,
} from "@/lib/download-file-query";
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

type SearchFormData = {
	trackTitle: string;
	albumName: string;
	artists: string;
	duration: string;
};

function DownloadButton({
	result,
	onDownload,
	isActive,
}: {
	result: SoulSeekSearchResult;
	onDownload: (result: SoulSeekSearchResult) => void;
	isActive: boolean;
}) {
	const downloadInput: DownloadFileInput = {
		username: result.username,
		token: result.token,
		filename: result.filename,
		size: result.size,
	};

	const {
		data: downloadState,
		isLoading,
		error,
	} = useQuery({
		...downloadFileQuery(downloadInput),
		enabled: isActive,
	});

	const isDownloading = downloadState?.status === "downloading" || isLoading;
	const isCompleted = downloadState?.status === "completed";
	const isFailed = downloadState?.status === "failed" || error !== null;

	return (
		<div className="flex flex-col items-end gap-2">
			<Button
				variant="outline"
				size="sm"
				onClick={() => onDownload(result)}
				disabled={isDownloading || isCompleted}
			>
				{isDownloading ? (
					<>
						<Loader2 className="mr-2 h-4 w-4 animate-spin" />
						Downloading...
					</>
				) : isCompleted ? (
					<>
						<Check className="mr-2 h-4 w-4" />
						Completed
					</>
				) : isFailed ? (
					<>
						<X className="mr-2 h-4 w-4" />
						Failed
					</>
				) : (
					<>
						<DownloadIcon className="mr-2 h-4 w-4" />
						Download
					</>
				)}
			</Button>
			{downloadState && downloadState.status === "downloading" && (
				<div className="w-32">
					<div className="text-xs text-muted-foreground mb-1">
						{downloadState.progress}% (
						{formatFileSize(downloadState.bytesDownloaded)} /{" "}
						{formatFileSize(downloadState.totalBytes)})
					</div>
					<div className="h-2 bg-muted rounded-full overflow-hidden">
						<div
							className="h-full bg-primary transition-all duration-300"
							style={{ width: `${downloadState.progress}%` }}
						/>
					</div>
				</div>
			)}
			{downloadState?.error && (
				<div className="text-xs text-destructive max-w-32 text-right">
					{downloadState.error}
				</div>
			)}
			{error && (
				<div className="text-xs text-destructive max-w-32 text-right">
					{error instanceof Error ? error.message : "Download failed"}
				</div>
			)}
		</div>
	);
}

export function Download() {
	const [activeDownloads, setActiveDownloads] = useState<Set<string>>(
		new Set(),
	);

	const searchSoulseek = useMutation({
		mutationFn: async (variables: MutationSearchSoulseekArgs) =>
			execute(SearchSoulseekMutation, variables),
	});
	const searchResults = searchSoulseek.data?.searchSoulseek ?? [];

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

	const handleDownload = (result: SoulSeekSearchResult) => {
		const downloadId = `${result.username}-${result.filename}`;
		setActiveDownloads((prev) => new Set(prev).add(downloadId));
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
				const isActive = activeDownloads.has(downloadId);

				return (
					<DownloadButton
						result={result}
						onDownload={handleDownload}
						isActive={isActive}
					/>
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
