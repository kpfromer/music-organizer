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
    throw new Error("Network response was not ok");
  }

  const res = await response.json();
  return res.data as TResult;
}
