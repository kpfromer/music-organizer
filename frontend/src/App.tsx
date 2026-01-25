import "./index.css";

import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { BrowserRouter, Route, Routes } from "react-router-dom";
import { Page } from "./components/page";
import { Download } from "./pages/download";
import { Home } from "./pages/home";
import { Playlist } from "./pages/playlist";
import { Playlists } from "./pages/playlists";
import { PlexAuthCallback } from "./pages/plex-auth-callback";
import { PlexServers } from "./pages/plex-servers";
import { PlexTracks } from "./pages/plex-tracks";
import { Spotify } from "./pages/spotify";
import { SpotifyAuthCallback } from "./pages/spotify-auth-callback";
import { SpotifyLogin } from "./pages/spotify-login";
import { SpotifyMatchedTracks } from "./pages/spotify-matched-tracks";
import { Tracks } from "./pages/tracks";
import { UnimportableFiles } from "./pages/unimportable-files";

const queryClient = new QueryClient();

function Providers({ children }: { children: React.ReactNode }) {
  return (
    <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
  );
}

export function App() {
  return (
    <BrowserRouter>
      <Providers>
        <Routes>
          <Route element={<Page />}>
            <Route path="/" element={<Home />} />
            <Route path="/albums" element={<>Albums</>} />
            <Route path="/tracks" element={<Tracks />} />
            <Route path="/playlists" element={<Playlists />} />
            <Route path="/playlist/:id" element={<Playlist />} />
            <Route path="/download" element={<Download />} />
            <Route path="/unimportable-files" element={<UnimportableFiles />} />
            <Route path="/plex-servers" element={<PlexServers />} />
            <Route path="/plex-tracks" element={<PlexTracks />} />
            <Route path="/spotify" element={<Spotify />} />
            <Route path="/spotify/login" element={<SpotifyLogin />} />
            <Route
              path="/spotify/matched-tracks"
              element={<SpotifyMatchedTracks />}
            />
          </Route>
          <Route path="/plex-auth/callback" element={<PlexAuthCallback />} />
          <Route
            path="/spotify-auth/callback-frontend"
            element={<SpotifyAuthCallback />}
          />
        </Routes>
      </Providers>
    </BrowserRouter>
  );
}

export default App;
