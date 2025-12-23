import "./index.css";

import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { BrowserRouter, Route, Routes } from "react-router-dom";
import { getAllPages } from "./AppConfig";
import { Page } from "./components/page";
import { Home } from "./pages/home";

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
					{getAllPages().map((page) => (
						<Route
							key={page.id}
							path={page.path}
							element={<page.component />}
						/>
					))}
				</Routes>
			</Providers>
		</BrowserRouter>
	);
}

export default App;
