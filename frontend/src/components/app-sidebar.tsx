import {
  AlertCircle,
  Download,
  Heart,
  Home,
  ListMusic,
  Music,
  Server,
  Video,
} from "lucide-react";
import { Link } from "react-router-dom";
import {
  Sidebar,
  SidebarContent,
  SidebarGroup,
  SidebarGroupContent,
  SidebarGroupLabel,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
} from "@/components/ui/sidebar";

// Menu items.
const items = [
  {
    title: "Home",
    url: "/",
    icon: Home,
  },
  {
    title: "Tracks",
    url: "/tracks",
    icon: Music,
  },
  {
    title: "Playlists",
    url: "/playlists",
    icon: ListMusic,
  },
  {
    title: "Download",
    url: "/download",
    icon: Download,
  },
  {
    title: "Wishlist",
    url: "/wishlist",
    icon: Heart,
  },
  {
    title: "Unimportable Files",
    url: "/unimportable-files",
    icon: AlertCircle,
  },
  {
    title: "Plex Servers",
    url: "/plex-servers",
    icon: Server,
  },
  {
    title: "Plex Tracks",
    url: "/plex-tracks",
    icon: Music,
  },
  {
    title: "Spotify",
    url: "/spotify",
    icon: Music,
  },
  {
    title: "Matched Tracks",
    url: "/spotify/matched-tracks",
    icon: Music,
  },
  {
    title: "Unmatched Tracks",
    url: "/spotify/unmatched-tracks",
    icon: Music,
  },
  {
    title: "Youtube",
    url: "/youtube",
    icon: Video,
  },
  {
    title: "Youtube Subscriptions",
    url: "/youtube-subscriptions",
    icon: Video,
  },
];

export function AppSidebar() {
  return (
    <Sidebar>
      <SidebarContent>
        <SidebarGroup>
          <SidebarGroupLabel>Menu</SidebarGroupLabel>
          <SidebarGroupContent>
            <SidebarMenu>
              {items.map((item) => (
                <SidebarMenuItem key={item.title}>
                  <SidebarMenuButton asChild>
                    <Link to={item.url}>
                      <item.icon />
                      <span>{item.title}</span>
                    </Link>
                  </SidebarMenuButton>
                </SidebarMenuItem>
              ))}
            </SidebarMenu>
          </SidebarGroupContent>
        </SidebarGroup>
      </SidebarContent>
    </Sidebar>
  );
}
