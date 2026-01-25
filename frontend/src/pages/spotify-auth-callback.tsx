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
import type { MutationCompleteSpotifyAuthArgs } from "@/graphql/graphql";
import { execute } from "@/lib/execute-graphql";

const CompleteSpotifyAuthMutation = graphql(`
  mutation CompleteSpotifyAuth($authCode: String!, $csrfState: String!) {
    completeSpotifyAuth(authCode: $authCode, csrfState: $csrfState) {
      id
      userId
      displayName
      createdAt
      updatedAt
    }
  }
`);

export function SpotifyAuthCallback() {
  const navigate = useNavigate();
  const [searchParams] = useSearchParams();
  const [status, setStatus] = useState<
    "loading" | "success" | "error" | "missing-params"
  >("loading");

  const completeAuth = useMutation({
    mutationFn: async (variables: MutationCompleteSpotifyAuthArgs) =>
      execute(CompleteSpotifyAuthMutation, variables),
    onSuccess: () => {
      setStatus("success");
      // Redirect after 2 seconds
      setTimeout(() => {
        navigate("/spotify");
      }, 2000);
    },
    onError: () => {
      setStatus("error");
    },
  });

  const authCode = searchParams.get("code");
  const csrfState = searchParams.get("state");

  useEffect(() => {
    if (!authCode || !csrfState) {
      setStatus("missing-params");
      return;
    }

    completeAuth.mutate({ authCode: authCode, csrfState: csrfState });
  }, [authCode, csrfState, completeAuth.mutate]);

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
              Missing required parameters. Please try authenticating again.
            </CardDescription>
          </CardHeader>
          <CardContent>
            <Button onClick={() => navigate("/spotify")} className="w-full">
              Go to Spotify
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
                <li>The authorization code expired</li>
                <li>There was an error connecting to Spotify</li>
              </ul>
            </CardDescription>
          </CardHeader>
          <CardContent className="flex gap-2">
            <Button
              onClick={() => {
                const code = searchParams.get("code");
                const state = searchParams.get("state");
                if (code && state) {
                  completeAuth.mutate({ authCode: code, csrfState: state });
                  setStatus("loading");
                }
              }}
              className="flex-1"
            >
              Retry
            </Button>
            <Button
              onClick={() => navigate("/spotify")}
              variant="outline"
              className="flex-1"
            >
              Go to Spotify
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
              Your Spotify account has been successfully authenticated. If this
              account already existed, your tokens have been updated.
              Redirecting to Spotify page...
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
