import { z } from "zod";
import type { TypedDocumentString } from "../graphql/graphql";

const GraphQLErrorSchema = z.object({
	message: z.string(),
	locations: z
		.array(
			z.object({
				line: z.number(),
				column: z.number(),
			}),
		)
		.optional(),
	path: z.array(z.union([z.string(), z.number()])).optional(),
	extensions: z.record(z.string(), z.unknown()).optional(),
});

const GraphQLSuccessResponseSchema = z.object({
	data: z.unknown().optional(),
	errors: z.array(GraphQLErrorSchema).optional(),
});

export async function execute<TResult, TVariables>(
	query: TypedDocumentString<TResult, TVariables>,
	...[variables]: TVariables extends Record<string, never> ? [] : [TVariables]
) {
	// biome-ignore lint/style/noNonNullAssertion: we know that the environment variable is set
	const response = await fetch(process.env.PUBLIC_GRAPHQL_URL!, {
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

	const errorResponse = GraphQLErrorSchema.safeParse(res);
	if (errorResponse.success) {
		throw new Error(errorResponse.data.message);
	}

	const successResponse = GraphQLSuccessResponseSchema.parse(res);
	if ("errors" in successResponse) {
		throw new Error(
			successResponse.errors?.[0]?.message ?? "Unknown graphql error",
		);
	}

	return successResponse.data as TResult;
}
