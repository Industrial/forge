import { useState } from "react";
import { style } from "@react-spectrum/s2/style" with { type: "macro" };
import Sidebar from "../../components/Sidebar/Sidebar";
import Navbar from "../../../../components/Navbar";

export type DashboardLayoutProps = {
	children: React.ReactNode;
	colorScheme: "light" | "dark";
	onToggleTheme: () => void;
};

export default function DashboardLayout({
	children,
	colorScheme,
	onToggleTheme,
}: DashboardLayoutProps) {
	const [sidebarExpanded, setSidebarExpanded] = useState(true);

	return (
		<>
			<Navbar
				appName="App"
				colorScheme={colorScheme}
				onToggleTheme={onToggleTheme}
			/>

			<div
				className={[
					style({
						display: "flex",
						flexDirection: "row",
						flexGrow: 1,
						minHeight: 0,
						overflow: "hidden",
					}),
					"dashboard-page",
				].join(" ")}
			>
				<Sidebar
					expanded={sidebarExpanded}
					onToggle={() => setSidebarExpanded((e) => !e)}
				/>
				<div
					className={[
						style({
							flexGrow: 1,
							minWidth: 0,
							overflow: "auto",
							paddingBlock: 16,
							paddingInline: 16,
						}),
						"dashboard-content",
					].join(" ")}
				>
					{children}
				</div>
			</div>
		</>
	);
}
