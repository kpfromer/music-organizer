import type { CodegenConfig } from "@graphql-codegen/cli";

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
		"./schema.graphql": {
			plugins: ["schema-ast"],
			config: {
				includeDirectives: true,
			},
		},
	},
};

export default config;
