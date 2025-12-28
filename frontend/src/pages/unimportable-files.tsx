import { useQuery } from "@tanstack/react-query";
import {
	type ColumnDef,
	flexRender,
	getCoreRowModel,
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

const UnimportableFilesQuery = graphql(`
  query UnimportableFiles($page: Int, $pageSize: Int) {
    unimportableFiles(page: $page, pageSize: $pageSize) {
      files {
        id
        filePath
        reason
        createdAt
        sha256
      }
      totalCount
      page
      pageSize
    }
  }
`);

type FileWithDate = {
	id: number;
	filePath: string;
	reason: string;
	createdAt: Date;
	sha256: string;
};

function formatReason(reason: string): string {
	// Convert snake_case to Title Case
	return reason
		.split("_")
		.map((word) => word.charAt(0).toUpperCase() + word.slice(1))
		.join(" ");
}

export function UnimportableFiles() {
	const [page, setPage] = useState(1);
	const [pageSize, setPageSize] = useState(25);
	const [sorting, setSorting] = useState<SortingState>([
		{ id: "createdAt", desc: true },
	]);

	const { data, isLoading } = useQuery({
		queryKey: ["unimportableFiles", page, pageSize],
		queryFn: async () => {
			const data = await execute(UnimportableFilesQuery, { page, pageSize });
			return {
				...data.unimportableFiles,
				files: data.unimportableFiles.files.map((file) => ({
					...file,
					createdAt: parseISO(file.createdAt),
				})),
			};
		},
	});

	const columns: ColumnDef<FileWithDate>[] = [
		{
			accessorKey: "filePath",
			header: "File Path",
			cell: ({ row }) => {
				return (
					<div className="font-mono text-sm">{row.getValue("filePath")}</div>
				);
			},
		},
		{
			accessorKey: "reason",
			header: "Reason",
			cell: ({ row }) => {
				const reason = row.getValue("reason") as string;
				return (
					<div className="font-medium text-destructive">
						{formatReason(reason)}
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
						Date
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
			accessorKey: "sha256",
			header: "SHA256",
			cell: ({ row }) => {
				const sha256 = row.getValue("sha256") as string;
				return (
					<div className="font-mono text-xs text-muted-foreground">
						{sha256.substring(0, 16)}...
					</div>
				);
			},
		},
	];

	const tableData: FileWithDate[] = data?.files ?? [];

	const table = useReactTable({
		data: tableData,
		columns,
		getCoreRowModel: getCoreRowModel(),
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
				<div className="text-muted-foreground">
					Loading unimportable files...
				</div>
			</div>
		);
	}

	return (
		<div className="container mx-auto p-8">
			<div className="mb-4 flex items-center justify-between">
				<h1 className="text-2xl font-bold">Unimportable Files</h1>
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
									No unimportable files found.
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
					{data?.totalCount ?? 0} files
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
