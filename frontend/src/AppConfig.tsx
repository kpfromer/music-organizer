import type { LucideIcon } from "lucide-react";
import { Home as HomeIcon } from "lucide-react";
import type { ComponentType } from "react";
import { Page } from "@/components/page";
import { Home } from "@/pages/home";

export interface AppPage {
	id: string;
	path: string;
	title: string;
	component: ComponentType;
}

export interface SidebarPageItem {
	type: "page";
	title: string;
	icon: LucideIcon;
	page: AppPage;
}

export interface SidebarGroupItem {
	type: "group";
	title: string;
	icon: LucideIcon;
	items: AppPage[];
}

export type SidebarItem = SidebarPageItem | SidebarGroupItem;

export const sidebarConfig: SidebarItem[] = [
	{
		type: "page",
		title: "Home",
		icon: HomeIcon,
		page: {
			id: "home",
			path: "/",
			title: "Home",
			component: () => {
				return (
					<Page>
						<Home />
					</Page>
				);
			},
		},
	},
	{
		type: "page",
		title: "Albums",
		icon: HomeIcon, // TODO: Replace with appropriate icon
		page: {
			id: "albums",
			path: "/albums",
			title: "Albums",
			component: () => (
				<Page>
					<div>Albums Page</div>
				</Page>
			),
		},
	},
];

// Helper to get all pages flattened for routing
export function getAllPages(): AppPage[] {
	const pages: AppPage[] = [];

	sidebarConfig.forEach((item) => {
		if (item.type === "page") {
			pages.push(item.page);
		} else {
			pages.push(...item.items);
		}
	});

	return pages;
}
