export function getUrl(path: string) {
  if (!path.startsWith("/")) {
    throw new Error("Path must start with a slash");
  }
  const publicGraphqlUrl = process.env.PUBLIC_GRAPHQL_URL;
  if (!publicGraphqlUrl) {
    console.error("PUBLIC_GRAPHQL_URL is not set");
    throw new Error("PUBLIC_GRAPHQL_URL is not set");
  }
  // Remove trailing slash from publicGraphqlUrl (if present)
  const publicGraphqlUrlNoSlash = publicGraphqlUrl.endsWith("/")
    ? publicGraphqlUrl.slice(0, -1)
    : publicGraphqlUrl;
  const url = publicGraphqlUrlNoSlash.replace("/graphql", "");
  return `${url}${path}`;
}
