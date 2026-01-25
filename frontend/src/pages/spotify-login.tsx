import { useMutation } from "@tanstack/react-query";
import { ExternalLink, Loader2, Music, XCircle } from "lucide-react";
import { useNavigate } from "react-router-dom";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { graphql } from "@/graphql";
import { execute } from "@/lib/execute-graphql";

const InitiateSpotifyAuthMutation = graphql(`
  mutation InitiateSpotifyAuth {
    initiateSpotifyAuth {
      redirectUrl
    }
  }
`);

export function SpotifyLogin() {
  const navigate = useNavigate();

  const initiateAuth = useMutation({
    mutationFn: async () => execute(InitiateSpotifyAuthMutation),
    onSuccess: (data) => {
      // Redirect to Spotify's authorization page
      window.location.href = data.initiateSpotifyAuth.redirectUrl;
    },
  });

  const handleLogin = () => {
    initiateAuth.mutate();
  };

  if (initiateAuth.isError) {
    return (
      <div className="container mx-auto p-8 flex items-center justify-center min-h-screen">
        <Card className="w-full max-w-md">
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <XCircle className="h-5 w-5 text-red-500" />
              Authentication Failed
            </CardTitle>
            <CardDescription>
              Failed to initiate Spotify authentication. This could be because:
              <ul className="list-disc list-inside mt-2 space-y-1">
                <li>Spotify credentials are not configured</li>
                <li>There was an error connecting to the server</li>
                <li>The authentication service is temporarily unavailable</li>
              </ul>
            </CardDescription>
          </CardHeader>
          <CardContent className="flex gap-2">
            <Button
              onClick={handleLogin}
              className="flex-1"
              disabled={initiateAuth.isPending}
            >
              {initiateAuth.isPending ? (
                <>
                  <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                  Retrying...
                </>
              ) : (
                "Retry"
              )}
            </Button>
            <Button
              onClick={() => navigate("/spotify")}
              variant="outline"
              className="flex-1"
            >
              Back to Spotify
            </Button>
          </CardContent>
        </Card>
      </div>
    );
  }

  if (initiateAuth.isPending) {
    return (
      <div className="container mx-auto p-8 flex items-center justify-center min-h-screen">
        <Card className="w-full max-w-md">
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Loader2 className="h-5 w-5 animate-spin" />
              Initiating Authentication
            </CardTitle>
            <CardDescription>
              Please wait while we prepare the authentication flow...
            </CardDescription>
          </CardHeader>
        </Card>
      </div>
    );
  }

  return (
    <div className="container mx-auto p-8 flex items-center justify-center min-h-screen">
      <Card className="w-full max-w-md">
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Music className="h-5 w-5" />
            Login with Spotify
          </CardTitle>
          <CardDescription>
            Connect your Spotify account to sync playlists and manage your music
            library.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <Button
            onClick={handleLogin}
            className="w-full"
            size="lg"
            disabled={initiateAuth.isPending}
          >
            {initiateAuth.isPending ? (
              <>
                <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                Connecting...
              </>
            ) : (
              <>
                <ExternalLink className="mr-2 h-4 w-4" />
                Login with Spotify
              </>
            )}
          </Button>
          <Button
            onClick={() => navigate("/spotify")}
            variant="outline"
            className="w-full"
          >
            Back to Spotify
          </Button>
        </CardContent>
      </Card>
    </div>
  );
}
