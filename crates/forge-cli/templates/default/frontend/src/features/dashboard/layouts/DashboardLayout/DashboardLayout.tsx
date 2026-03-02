import { useState } from "react";
import Box from "@mui/material/Box";
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
			<Box
				className="dashboard-page"
				sx={{
					display: "flex",
					flexDirection: "row",
					flexGrow: 1,
					minHeight: 0,
					overflow: "hidden",
				}}
			>
				<Sidebar
					expanded={sidebarExpanded}
					onToggle={() => setSidebarExpanded((e) => !e)}
				/>
				<Box
					className="dashboard-content"
					sx={{
						flexGrow: 1,
						minWidth: 0,
						overflow: "auto",
						py: 2,
						px: 2,
					}}
				>
					{children}
				</Box>
			</Box>
		</>
	);
}
