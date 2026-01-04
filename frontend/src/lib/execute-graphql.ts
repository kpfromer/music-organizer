import type { TypedDocumentString } from "../graphql/graphql";

export async function execute<TResult, TVariables>(
  query: TypedDocumentString<TResult, TVariables>,
  ...[variables]: TVariables extends Record<string, never> ? [] : [TVariables]
) {
  const publicGraphqlUrl = process.env.PUBLIC_GRAPHQL_URL;
  if (!publicGraphqlUrl) {
    console.error("PUBLIC_GRAPHQL_URL is not set");
    throw new Error("PUBLIC_GRAPHQL_URL is not set");
  }
  const response = await fetch(publicGraphqlUrl, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      Accept: "application/graphql-response+json",
    },
    body: JSON.stringify({
      query,
      variables,
    }),
  });

  if (!response.ok) {
    const errorText = await response.text();
    console.error("GraphQL request failed:", {
      status: response.status,
      statusText: response.statusText,
      body: errorText,
    });
    throw new Error(
      `Network response was not ok: ${response.status} ${response.statusText}`,
    );
  }

  const res = await response.json();

  // GraphQL returns errors in the response body even with 200 OK
  if (res.errors && res.errors.length > 0) {
    const errorMessages = res.errors
      .map((e: unknown) =>
        typeof e === "object" && e !== null && "message" in e
          ? e.message
          : JSON.stringify(e),
      )
      .join(", ");
    console.error("GraphQL errors:", res.errors);
    throw new Error(`GraphQL error: ${errorMessages}`);
  }

  return res.data as TResult;
}
