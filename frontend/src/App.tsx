import "./index.css";

import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { Home } from "./pages/home";

const queryClient = new QueryClient();

function Providers({ children }: { children: React.ReactNode }) {
  return (
    <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
  );
}

export function App() {
  return (
    <Providers>
      <Home />
    </Providers>
  );
}

export default App;
