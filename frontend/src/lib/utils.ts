import { type ClassValue, clsx } from "clsx";
import { twMerge } from "tailwind-merge";

export function cn(...inputs: ClassValue[]): string {
  return twMerge(clsx(inputs));
}

/**
 * Sets a cookie using the Cookie Store API.
 * Falls back to document.cookie if Cookie Store API is not available.
 */
export async function setCookie(
  name: string,
  value: string,
  options?: {
    maxAge?: number;
    path?: string;
    domain?: string;
    secure?: boolean;
    sameSite?: "strict" | "lax" | "none";
  },
): Promise<void> {
  if (typeof cookieStore !== "undefined") {
    await cookieStore.set({
      name,
      value,
      expires: options?.maxAge ? Date.now() + options.maxAge * 1000 : undefined,
      path: options?.path ?? "/",
      domain: options?.domain,
      // secure: options?.secure,
      sameSite: options?.sameSite,
    });
  } else {
    // Fallback for browsers that don't support Cookie Store API
    const cookieString = `${name}=${value}; path=${options?.path ?? "/"}${
      options?.maxAge ? `; max-age=${options.maxAge}` : ""
    }${options?.domain ? `; domain=${options.domain}` : ""}${
      options?.secure ? "; secure" : ""
    }${options?.sameSite ? `; samesite=${options.sameSite}` : ""}`;
    // biome-ignore lint/suspicious/noDocumentCookie: fallback for browsers that don't support Cookie Store API
    document.cookie = cookieString;
  }
}
