import { z } from "zod";

const envSchema = z.object({
  PORT: z.coerce.number().default(3001),
  NODE_ENV: z.enum(["development", "production"]).default("development"),
  GRAPHQL_URL: z.string().default("http://localhost:3000/graphql"),
});

export const env = envSchema.parse(process.env);
