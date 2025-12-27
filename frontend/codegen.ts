import type { CodegenConfig } from "@graphql-codegen/cli";

const REQUIRED_ENV_VARIABLES = ["PUBLIC_GRAPHQL_URL"] as const;

for (const envVar of REQUIRED_ENV_VARIABLES) {
  if (!process.env[envVar]) {
    throw new Error(`Environment variable ${envVar} is not set`);
  }
}

const config: CodegenConfig = {
  // biome-ignore lint/style/noNonNullAssertion: we know that the environment variable is set
  schema: process.env.PUBLIC_GRAPHQL_URL!,
  documents: ["src/**/*.{ts,tsx}"],
  ignoreNoDocuments: true,
  generates: {
    "./src/graphql/": {
      preset: "client",
      config: {
        documentMode: "string",
      },
    },
  },
};

export default config;
