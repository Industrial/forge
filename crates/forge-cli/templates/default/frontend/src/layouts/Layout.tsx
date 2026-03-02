import { style } from "@react-spectrum/s2/style" with { type: "macro" };
import React from "react";
import Navbar from "../components/Navbar";

export type LayoutProps = {
	children: React.ReactNode;
	colorScheme: "light" | "dark";
	onToggleTheme: () => void;
};

export default function Layout({
	children,
	colorScheme,
	onToggleTheme,
}: LayoutProps) {
	return (
		<div
			className={style({
				display: "flex",
				flexDirection: "column",
				minHeight: "full",
				height: "full",
				overflow: "auto",
				flexGrow: 1,
			})}
		>
			<Navbar
				appName="App"
				colorScheme={colorScheme}
				onToggleTheme={onToggleTheme}
			/>
			<div
				className={style({
					display: "flex",
					flexDirection: "column",
					gap: 16,
					margin: 16,
					backgroundColor: "layer-1",
					padding: 16,
					borderRadius: "default",
					flexGrow: 1,
				})}
			>
				{children}
			</div>
		</div>
	);
}
