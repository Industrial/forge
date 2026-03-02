import { NavLink } from "react-router-dom";
import { ActionButton } from "@react-spectrum/s2";
import { style } from "@react-spectrum/s2/style" with { type: "macro" };
import ChevronLeftIcon from "@react-spectrum/s2/icons/ChevronLeft";
import ChevronRightIcon from "@react-spectrum/s2/icons/ChevronRight";
import ChartBarVertIcon from "@react-spectrum/s2/icons/ChartBarVert";
import BuildingsIcon from "@react-spectrum/s2/icons/Buildings";
import LockIcon from "@react-spectrum/s2/icons/Lock";
import UserIcon from "@react-spectrum/s2/icons/User";

const navItems = [
	{ to: "/dashboard", label: "Dashboard", end: true, icon: ChartBarVertIcon },
	{
		to: "/dashboard/organizations",
		label: "Organizations",
		end: false,
		icon: BuildingsIcon,
	},
	{ to: "/dashboard/users", label: "Users", end: false, icon: UserIcon },
	{
		to: "/dashboard/roles-and-permissions",
		label: "Permissions",
		end: false,
		icon: LockIcon,
	},
] as const;

const linkClassName = style({
	paddingBlock: 8,
	paddingInline: 12,
	borderRadius: "default",
	font: "body",
	textDecoration: "none",
	cursor: "pointer",
	display: "flex",
	alignItems: "center",
	gap: 12,
});

const iconOnlyClassName = style({
	paddingBlock: 8,
	paddingInline: 12,
	borderRadius: "default",
	font: "body",
	textDecoration: "none",
	cursor: "pointer",
	display: "flex",
	alignItems: "center",
	justifyContent: "center",
});

const asideClassName = style({
	backgroundColor: "layer-1",
	borderColor: "gray-400",
	borderEndWidth: 1,
	borderStyle: "solid",
	borderWidth: 0,
	display: "flex",
	flexDirection: "column",
	flexShrink: 0,
	gap: 4,
	height: "100vh",
	overflow: "hidden",
	padding: 8,
});

export type SidebarProps = {
	expanded: boolean;
	onToggle: () => void;
};

export default function Sidebar({ expanded, onToggle }: SidebarProps) {
	const asideClassNames = [
		asideClassName,
		expanded
			? style({
					width: 240,
				})
			: style({
					width: 52,
				}),
		"dashboard-sidebar",
	].join(" ");

	return (
		<aside className={asideClassNames} aria-label="Dashboard navigation">
			<div
				className={[
					style({
						display: "flex",
						alignItems: "center",
						paddingBlockEnd: 8,
						marginBlockEnd: 8,
					}),
					expanded
						? "dashboard-toggle-bar--expanded"
						: "dashboard-toggle-bar--collapsed",
				].join(" ")}
			>
				<ActionButton
					isQuiet
					aria-label={expanded ? "Collapse sidebar" : "Expand sidebar"}
					onPress={onToggle}
				>
					{expanded ? <ChevronLeftIcon /> : <ChevronRightIcon />}
				</ActionButton>
			</div>
			<nav
				className={style({
					display: "flex",
					flexDirection: "column",
					gap: 4,
				})}
			>
				{navItems.map(({ to, label, end, icon: Icon }) => (
					<NavLink
						key={to}
						to={to}
						end={end}
						className={({ isActive }) =>
							[
								expanded ? linkClassName : iconOnlyClassName,
								isActive ? "dashboard-sidebar-link--active" : "",
							]
								.filter(Boolean)
								.join(" ")
						}
						title={!expanded ? label : undefined}
					>
						<Icon />
						{expanded ? <span>{label}</span> : null}
					</NavLink>
				))}
			</nav>
		</aside>
	);
}
