import { useMutation } from "@tanstack/react-query";
import { CheckCircle, Loader2, XCircle } from "lucide-react";
import { useEffect, useState } from "react";
import { useNavigate, useSearchParams } from "react-router-dom";
import { Button } from "@/components/ui/button";
import {
	Card,
	CardContent,
	CardDescription,
	CardHeader,
	CardTitle,
} from "@/components/ui/card";
import { graphql } from "@/graphql";
import type { MutationCompletePlexServerAuthenticationArgs } from "@/graphql/graphql";
import { execute } from "@/lib/execute-graphql";

const CompletePlexServerAuthenticationMutation = graphql(`
  mutation CompletePlexServerAuthentication($serverId: Int!, $pinId: Int!) {
    completePlexServerAuthentication(serverId: $serverId, pinId: $pinId) {
      id
      name
      serverUrl
      hasAccessToken
      createdAt
      updatedAt
    }
  }
`);

export function PlexAuthCallback() {
	const navigate = useNavigate();
	const [searchParams] = useSearchParams();
	const [status, setStatus] = useState<
		"loading" | "success" | "error" | "missing-params"
	>("loading");

	const completeAuth = useMutation({
		mutationFn: async (
			variables: MutationCompletePlexServerAuthenticationArgs,
		) => execute(CompletePlexServerAuthenticationMutation, variables),
		onSuccess: (_, variables) => {
			// Clean up localStorage
			const pinIdKey = Object.keys(localStorage).find((key) =>
				key.startsWith("plex_auth_"),
			);
			if (pinIdKey) {
				const authData = JSON.parse(localStorage.getItem(pinIdKey) || "{}");
				if (authData.pinId === variables.pinId) {
					localStorage.removeItem(pinIdKey);
				}
			}
			setStatus("success");
			// Redirect after 2 seconds
			setTimeout(() => {
				navigate("/plex-servers");
			}, 2000);
		},
		onError: () => {
			setStatus("error");
		},
	});

	useEffect(() => {
		// Try to get serverId and pinId from URL params first
		let serverIdParam = searchParams.get("serverId");
		let pinIdParam = searchParams.get("pinId");

		// If not in URL, try to get from localStorage (stored when auth was initiated)
		if (!serverIdParam || !pinIdParam) {
			const pinIdKey = Object.keys(localStorage).find((key) =>
				key.startsWith("plex_auth_"),
			);
			if (pinIdKey) {
				try {
					const authData = JSON.parse(localStorage.getItem(pinIdKey) || "{}");
					serverIdParam = authData.serverId?.toString();
					pinIdParam = authData.pinId?.toString();
				} catch {
					// Ignore parse errors
				}
			}
		}

		if (!serverIdParam || !pinIdParam) {
			setStatus("missing-params");
			return;
		}

		const serverId = parseInt(serverIdParam, 10);
		const pinId = parseInt(pinIdParam, 10);

		if (Number.isNaN(serverId) || Number.isNaN(pinId)) {
			setStatus("missing-params");
			return;
		}

		completeAuth.mutate({ serverId, pinId });
	}, [searchParams, completeAuth.mutate]);

	if (status === "missing-params") {
		return (
			<div className="container mx-auto p-8 flex items-center justify-center min-h-screen">
				<Card className="w-full max-w-md">
					<CardHeader>
						<CardTitle className="flex items-center gap-2">
							<XCircle className="h-5 w-5 text-red-500" />
							Invalid Request
						</CardTitle>
						<CardDescription>
							Missing required parameters. Please try authenticating again from
							the Plex Servers page.
						</CardDescription>
					</CardHeader>
					<CardContent>
						<Button
							onClick={() => navigate("/plex-servers")}
							className="w-full"
						>
							Go to Plex Servers
						</Button>
					</CardContent>
				</Card>
			</div>
		);
	}

	if (status === "error") {
		return (
			<div className="container mx-auto p-8 flex items-center justify-center min-h-screen">
				<Card className="w-full max-w-md">
					<CardHeader>
						<CardTitle className="flex items-center gap-2">
							<XCircle className="h-5 w-5 text-red-500" />
							Authentication Failed
						</CardTitle>
						<CardDescription>
							Failed to complete authentication. This could be because:
							<ul className="list-disc list-inside mt-2 space-y-1">
								<li>The authentication window was closed too early</li>
								<li>The PIN expired</li>
								<li>There was an error connecting to the Plex server</li>
							</ul>
						</CardDescription>
					</CardHeader>
					<CardContent className="flex gap-2">
						<Button
							onClick={() => {
								// Try to get from URL params or localStorage
								let serverIdParam = searchParams.get("serverId");
								let pinIdParam = searchParams.get("pinId");

								if (!serverIdParam || !pinIdParam) {
									const pinIdKey = Object.keys(localStorage).find((key) =>
										key.startsWith("plex_auth_"),
									);
									if (pinIdKey) {
										try {
											const authData = JSON.parse(
												localStorage.getItem(pinIdKey) || "{}",
											);
											serverIdParam = authData.serverId?.toString();
											pinIdParam = authData.pinId?.toString();
										} catch {
											// Ignore parse errors
										}
									}
								}

								if (serverIdParam && pinIdParam) {
									const serverId = parseInt(serverIdParam, 10);
									const pinId = parseInt(pinIdParam, 10);
									if (!Number.isNaN(serverId) && !Number.isNaN(pinId)) {
										completeAuth.mutate({ serverId, pinId });
										setStatus("loading");
									}
								}
							}}
							className="flex-1"
						>
							Retry
						</Button>
						<Button
							onClick={() => navigate("/plex-servers")}
							variant="outline"
							className="flex-1"
						>
							Go to Plex Servers
						</Button>
					</CardContent>
				</Card>
			</div>
		);
	}

	if (status === "success") {
		return (
			<div className="container mx-auto p-8 flex items-center justify-center min-h-screen">
				<Card className="w-full max-w-md">
					<CardHeader>
						<CardTitle className="flex items-center gap-2">
							<CheckCircle className="h-5 w-5 text-green-500" />
							Authentication Successful
						</CardTitle>
						<CardDescription>
							Your Plex server has been successfully authenticated. Redirecting
							to Plex Servers page...
						</CardDescription>
					</CardHeader>
				</Card>
			</div>
		);
	}

	// Loading state
	return (
		<div className="container mx-auto p-8 flex items-center justify-center min-h-screen">
			<Card className="w-full max-w-md">
				<CardHeader>
					<CardTitle className="flex items-center gap-2">
						<Loader2 className="h-5 w-5 animate-spin" />
						Completing Authentication
					</CardTitle>
					<CardDescription>
						Please wait while we complete the authentication process...
					</CardDescription>
				</CardHeader>
			</Card>
		</div>
	);
}
