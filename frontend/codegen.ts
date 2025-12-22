import type { CodegenConfig } from "@graphql-codegen/cli";
import { env } from "./src/env";

const config: CodegenConfig = {
  schema: env.GRAPHQL_URL,
  documents: ["src/**/*.{ts,tsx}"],
  ignoreNoDocuments: true,
  generates: {
    "./src/graphql/": {
      preset: "client",
      config: {
        documentMode: "string",
      },
    },
    "./schema.graphql": {
      plugins: ["schema-ast"],
      config: {
        includeDirectives: true,
      },
    },
  },
};

export default config;
