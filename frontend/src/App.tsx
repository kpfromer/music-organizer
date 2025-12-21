import {
	Card,
	CardContent,
	CardDescription,
	CardHeader,
	CardTitle,
} from "@/components/ui/card";
import { APITester } from "./APITester";
import "./index.css";

import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import logo from "./logo.svg";
import { Home } from "./pages/home";
import reactLogo from "./react.svg";

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
