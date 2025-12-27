import { useQuery } from "@tanstack/react-query";
import {
	Card,
	CardDescription,
	CardHeader,
	CardTitle,
} from "@/components/ui/card";
import { graphql } from "@/graphql";
import { execute } from "@/lib/execute-graphql";

const TestQuery = graphql(`
  query Test {
    howdy
  }
`);

export function Home() {
	const { data } = useQuery({
		queryKey: ["test"],
		queryFn: async () => execute(TestQuery),
	});
	return (
		<div className="container mx-auto p-8 text-center relative z-10">
			<h1>Test Query: {data?.howdy}</h1>
			<div className="flex justify-center items-center gap-8 mb-8"></div>
			<Card>
				<CardHeader className="gap-4">
					<CardTitle className="text-3xl font-bold">Bun + React</CardTitle>
					<CardDescription>
						Edit{" "}
						<code className="rounded bg-muted px-[0.3rem] py-[0.2rem] font-mono">
							src/App.tsx
						</code>{" "}
						and save to test HMR
					</CardDescription>
				</CardHeader>
			</Card>
		</div>
	);
}
