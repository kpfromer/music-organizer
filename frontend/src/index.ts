import { serve } from "bun";
import index from "./index.html";

console.log("process.env", process.env.PUBLIC_GRAPHQL_URL);

const server = serve({
  routes: {
    // Serve index.html for all unmatched routes.
    "/*": index,
  },

  development: process.env.NODE_ENV !== "production" && {
    // Enable browser hot reloading in development
    hmr: true,

    // Echo console logs from the browser to the server
    console: true,
  },
  // Default to port 3001 for development
  port: 3001,
});

console.log(`🚀 Server running at ${server.url}`);
