import { Outlet } from "react-router-dom";
import { AppSidebar } from "@/components/app-sidebar";
import { AudioPlayer } from "@/components/audio-player";
import { SidebarProvider, SidebarTrigger } from "@/components/ui/sidebar";

const SIDEBAR_STATE_KEY = "sidebar_state";

export function Page() {
  const defaultOpen =
    (localStorage.getItem(SIDEBAR_STATE_KEY) ?? "true") === "true";

  return (
    <SidebarProvider defaultOpen={defaultOpen}>
      <AppSidebar />
      <main className="pb-20">
        <SidebarTrigger />
        <Outlet />
      </main>
      <AudioPlayer />
    </SidebarProvider>
  );
}
