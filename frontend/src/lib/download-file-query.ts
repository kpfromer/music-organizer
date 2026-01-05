import {
	queryOptions,
	experimental_streamedQuery as streamedQuery,
} from "@tanstack/react-query";
import { getUrl } from "./get-url";

export type DownloadFileInput = {
	username: string;
	token: string;
	filename: string;
	size: number;
};

export type DownloadEvent =
	| { type: "Started" }
	| { type: "Progress"; bytes_downloaded: number; total_bytes: number }
	| { type: "Completed" }
	| { type: "Failed"; message: string };

export type DownloadState = {
	status: "idle" | "downloading" | "completed" | "failed";
	progress: number; // 0-100
	bytesDownloaded: number;
	totalBytes: number;
	error?: string;
};

/**
 * Creates a query function that streams download progress from the backend.
 * The backend returns NDJSON (newline-delimited JSON) with DownloadEvent objects.
 */
async function* fetchDownloadStream(
	input: DownloadFileInput,
): AsyncIterable<DownloadEvent> {
	const url = getUrl("/download-file");
	const response = await fetch(url, {
		method: "POST",
		headers: {
			"Content-Type": "application/json",
		},
		body: JSON.stringify(input),
	});

	if (!response.ok) {
		const errorText = await response.text();
		throw new Error(
			`Download failed: ${response.status} ${response.statusText}. ${errorText}`,
		);
	}

	if (!response.body) {
		throw new Error("Response body is null");
	}

	const reader = response.body.getReader();
	const decoder = new TextDecoder();
	let buffer = "";

	try {
		while (true) {
			const { done, value } = await reader.read();
			if (done) break;

			buffer += decoder.decode(value, { stream: true });
			const lines = buffer.split("\n");
			buffer = lines.pop() || ""; // Keep incomplete line in buffer

			for (const line of lines) {
				const trimmed = line.trim();
				if (!trimmed) continue;

				try {
					const event = JSON.parse(trimmed) as DownloadEvent;
					yield event;
				} catch (e) {
					console.error("Failed to parse download event:", trimmed, e);
					// Continue processing other lines
				}
			}
		}

		// Process any remaining buffer
		if (buffer.trim()) {
			try {
				const event = JSON.parse(buffer.trim()) as DownloadEvent;
				yield event;
			} catch (e) {
				console.error("Failed to parse final download event:", buffer, e);
			}
		}
	} finally {
		reader.releaseLock();
	}
}

/**
 * Reducer function that accumulates download events into a DownloadState.
 */
function reduceDownloadEvents(
	accumulator: DownloadState,
	chunk: DownloadEvent,
): DownloadState {
	switch (chunk.type) {
		case "Started":
			return {
				...accumulator,
				status: "downloading",
			};

		case "Progress":
			return {
				...accumulator,
				status: "downloading",
				bytesDownloaded: chunk.bytes_downloaded,
				totalBytes: chunk.total_bytes,
				progress:
					chunk.total_bytes > 0
						? Math.round((chunk.bytes_downloaded / chunk.total_bytes) * 100)
						: 0,
			};

		case "Completed":
			return {
				...accumulator,
				status: "completed",
				progress: 100,
			};

		case "Failed":
			return {
				...accumulator,
				status: "failed",
				error: chunk.message,
			};

		default:
			return accumulator;
	}
}

/**
 * Query options for downloading a file with progress tracking.
 * Use this with useQuery to track download progress.
 *
 * @example
 * ```tsx
 * const { data, isLoading, error } = useQuery(
 *   downloadFileQuery({
 *     username: "user",
 *     token: "token",
 *     filename: "file.mp3",
 *     size: 1024,
 *   })
 * );
 * ```
 */
export function downloadFileQuery(input: DownloadFileInput) {
	return queryOptions({
		queryKey: ["download-file", input.username, input.filename, input.token],
		queryFn: streamedQuery({
			streamFn: () => fetchDownloadStream(input),
			initialValue: {
				status: "idle" as const,
				progress: 0,
				bytesDownloaded: 0,
				totalBytes: input.size,
			},
			reducer: reduceDownloadEvents,
			refetchMode: "reset",
		}),
		refetchOnWindowFocus: false,
		refetchOnMount: false,
		refetchOnReconnect: false,
	});
}
