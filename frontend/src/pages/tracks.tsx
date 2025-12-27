import { useQuery } from "@tanstack/react-query";
import {
	type ColumnDef,
	flexRender,
	getCoreRowModel,
	getSortedRowModel,
	type SortingState,
	useReactTable,
} from "@tanstack/react-table";
import { ArrowDown, ArrowUp, ArrowUpDown, Download } from "lucide-react";
import { useState } from "react";
import { Button } from "@/components/ui/button";
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
  query Tracks {
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
  }
`);

type Track = {
	id: number;
	title: string;
	trackNumber: number | null;
	duration: number | null;
	createdAt: number;
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

function formatDateAdded(timestamp: number): string {
	const now = Date.now() / 1000;
	const diff = now - timestamp;
	const days = Math.floor(diff / 86400);

	if (days === 0) return "Today";
	if (days === 1) return "1 day ago";
	if (days < 7) return `${days} days ago`;

	const weeks = Math.floor(days / 7);
	if (weeks === 1) return "1 week ago";
	if (weeks < 4) return `${weeks} weeks ago`;

	const months = Math.floor(days / 30);
	if (months === 1) return "1 month ago";
	if (months < 12) return `${months} months ago`;

	const years = Math.floor(days / 365);
	if (years === 1) return "1 year ago";
	return `${years} years ago`;
}

export function Tracks() {
	const [sorting, setSorting] = useState<SortingState>([
		{ id: "created_at", desc: true },
	]);

	const { data, isLoading } = useQuery({
		queryKey: ["tracks"],
		queryFn: () => execute(TracksQuery),
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
				const timestamp = row.getValue("created_at") as number;
				return (
					<div className="text-muted-foreground">
						{formatDateAdded(timestamp)}
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
	});

	if (isLoading) {
		return (
			<div className="container mx-auto p-8">
				<div className="text-muted-foreground">Loading tracks...</div>
			</div>
		);
	}

	return (
		<div className="container mx-auto p-8">
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
		</div>
	);
}
