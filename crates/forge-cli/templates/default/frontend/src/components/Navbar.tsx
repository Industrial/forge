import type { ReactNode } from "react";
import { ActionButton, Divider, SearchField } from "@react-spectrum/s2";
import { style } from "@react-spectrum/s2/style" with { type: "macro" };

type NavbarProps = {
	appName?: string;
	colorScheme?: "light" | "dark";
	onToggleTheme?: () => void;
};

/** ActionButton isQuiet that navigates to href on press (same look as docs navbar). */
function NavActionButton({
	href,
	children,
	"aria-label": ariaLabel,
}: {
	href: string;
	children: ReactNode;
	"aria-label"?: string;
}) {
	return (
		<ActionButton
			isQuiet
			aria-label={ariaLabel}
			onPress={() => {
				window.location.href = href;
			}}
		>
			{children}
		</ActionButton>
	);
}

function LogoIcon() {
	return (
		<svg
			width={24}
			height={24}
			viewBox="0 0 24 24"
			fill="none"
			xmlns="http://www.w3.org/2000/svg"
			aria-hidden
		>
			<path d="M12 2L2 22h20L12 2z" fill="var(--spectrum-red-600)" />
		</svg>
	);
}

function GitHubIcon() {
	return (
		<svg
			width={20}
			height={20}
			viewBox="0 0 24 24"
			fill="currentColor"
			aria-hidden
		>
			<path d="M12 0c-6.626 0-12 5.373-12 12 0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23.957-.266 1.983-.399 3.003-.404 1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576 4.765-1.589 8.199-6.086 8.199-11.386 0-6.627-5.373-12-12-12z" />
		</svg>
	);
}

function ThemeIcon({ isDark }: { isDark: boolean }) {
	return (
		<svg
			width={20}
			height={20}
			viewBox="0 0 24 24"
			fill="none"
			stroke="currentColor"
			strokeWidth={1.5}
			strokeLinecap="round"
			strokeLinejoin="round"
			aria-hidden
		>
			{isDark ? (
				<>
					<circle cx="12" cy="12" r="5" />
					<path d="M12 1v2M12 21v2M4.22 4.22l1.42 1.42M18.36 18.36l1.42 1.42M1 12h2M21 12h2M4.22 19.78l1.42-1.42M18.36 5.64l1.42-1.42" />
				</>
			) : (
				<>
					<path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z" />
				</>
			)}
		</svg>
	);
}

export default function Navbar({
	appName = "React Spectrum",
	colorScheme = "dark",
	onToggleTheme,
}: NavbarProps) {
	return (
		<div
			className={style({
				display: "flex",
				flexDirection: "row",
				alignItems: "center",
				gap: 16,
				paddingInline: 16,
				paddingBlock: 12,
				minHeight: 72,
				backgroundColor: "layer-1",
			})}
			style={{
				borderBottom: "1px solid var(--spectrum-gray-200)",
			}}
		>
			{/* Logo + app name (left) - link to home */}
			<NavActionButton href="/">
				<div
					className={style({
						display: "flex",
						flexDirection: "row",
						alignItems: "center",
						gap: 8,
						flexShrink: 0,
					})}
				>
					<LogoIcon />
					<span className={style({ font: "title" })}>{appName}</span>
				</div>
			</NavActionButton>

			{/* Center: search bar (flex grows, search centered within) */}
			<div
				className={style({
					flexGrow: 1,
					minWidth: 0,
					display: "flex",
					justifyContent: "center",
					marginInline: 16,
				})}
			>
				<div
					className={style({
						display: "flex",
						flexDirection: "row",
						alignItems: "center",
						gap: 8,
						width: "100%",
						maxWidth: 400,
						backgroundColor: "layer-2",
						borderRadius: "default",
						paddingInline: 12,
						paddingBlock: 8,
					})}
				>
					<div className={style({ flexGrow: 1, minWidth: 0 })}>
						<SearchField
							aria-label="Search"
							placeholder="Search React Spectrum"
							styles={style({ width: "100%" })}
						/>
					</div>
					<kbd
						className={style({
							font: "detail-sm",
							paddingInline: 8,
							paddingBlock: 4,
							backgroundColor: "layer-1",
							borderRadius: "sm",
							flexShrink: 0,
						})}
						style={{ border: "1px solid var(--spectrum-gray-400)" }}
						title="Keyboard shortcut"
					>
						⌘K
					</kbd>
				</div>
			</div>

			{/* Right: nav buttons + theme toggle */}
			<div
				className={style({
					display: "flex",
					flexDirection: "row",
					alignItems: "center",
					gap: 8,
					flexShrink: 0,
				})}
			>
				<NavActionButton href="https://react-spectrum.adobe.com/docs">
					Docs
				</NavActionButton>
				<NavActionButton href="https://react-spectrum.adobe.com/releases">
					Releases
				</NavActionButton>
				<NavActionButton href="https://react-spectrum.adobe.com/blog">
					Blog
				</NavActionButton>
				<NavActionButton
					href="https://github.com/adobe/react-spectrum"
					aria-label="GitHub"
				>
					<GitHubIcon />
				</NavActionButton>
				<NavActionButton href="https://www.npmjs.com/package/@react-spectrum/s2">
					npm
				</NavActionButton>
				<Divider orientation="vertical" />
				<ActionButton
					aria-label={
						colorScheme === "dark"
							? "Switch to light mode"
							: "Switch to dark mode"
					}
					isQuiet
					onPress={onToggleTheme}
				>
					<ThemeIcon isDark={colorScheme === "dark"} />
				</ActionButton>
			</div>
		</div>
	);
}
