import type { SoulSeekFileAttributeValue } from "@/graphql/graphql";

/**
 * Format bytes to human-readable file size
 */
export function formatFileSize(bytes: number): string {
  if (bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${(bytes / k ** i).toFixed(1)} ${sizes[i]}`;
}

/**
 * Format file attributes for display
 */
export function formatAttributes(
  attributes: Array<SoulSeekFileAttributeValue>,
): string {
  const parts: string[] = [];

  for (const attr of attributes) {
    switch (attr.attribute) {
      case "BITRATE":
        parts.push(`${attr.value} kbps`);
        break;
      case "DURATION":
        parts.push(formatDuration(attr.value));
        break;
      case "SAMPLE_RATE":
        parts.push(`${attr.value} Hz`);
        break;
      case "BIT_DEPTH":
        parts.push(`${attr.value}-bit`);
        break;
      case "VARIABLE_BIT_RATE":
        if (attr.value === 1) {
          parts.push("VBR");
        }
        break;
      case "ENCODER":
        parts.push(`Encoder: ${attr.value}`);
        break;
    }
  }

  return parts.join(" • ") || "No attributes";
}

/**
 * Format duration in seconds to MM:SS format
 */
function formatDuration(seconds: number): string {
  const minutes = Math.floor(seconds / 60);
  const remainingSeconds = seconds % 60;
  return `${minutes}:${remainingSeconds.toString().padStart(2, "0")}`;
}

/**
 * Parse comma-separated artists string into array
 */
export function parseArtistsInput(input: string): string[] {
  return input
    .split(",")
    .map((artist) => artist.trim())
    .filter((artist) => artist.length > 0);
}
