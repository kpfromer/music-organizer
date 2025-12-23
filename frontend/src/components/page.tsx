import { Outlet } from "react-router-dom";
import { AppSidebar } from "@/components/app-sidebar";
import { SidebarProvider, SidebarTrigger } from "@/components/ui/sidebar";

const SIDEBAR_STATE_KEY = "sidebar_state";

export function Page() {
	const defaultOpen =
		(localStorage.getItem(SIDEBAR_STATE_KEY) ?? "true") === "true";

	return (
		<SidebarProvider defaultOpen={defaultOpen}>
			<AppSidebar />
			<main>
				<SidebarTrigger />
				<Outlet />
			</main>
		</SidebarProvider>
	);
}
