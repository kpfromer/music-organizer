import "./index.css";

import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { BrowserRouter, Route, Routes } from "react-router-dom";
import { Page } from "./components/page";
import { Download } from "./pages/download";
import { Home } from "./pages/home";
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
            <Route path="/download" element={<Download />} />
            <Route path="/unimportable-files" element={<UnimportableFiles />} />
          </Route>
        </Routes>
      </Providers>
    </BrowserRouter>
  );
}

export default App;
