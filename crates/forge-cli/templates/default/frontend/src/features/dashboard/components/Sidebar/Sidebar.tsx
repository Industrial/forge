import { NavLink } from "react-router-dom";
import Box from "@mui/material/Box";
import IconButton from "@mui/material/IconButton";
import List from "@mui/material/List";
import ListItemButton from "@mui/material/ListItemButton";
import ListItemIcon from "@mui/material/ListItemIcon";
import ListItemText from "@mui/material/ListItemText";
import ChevronLeft from "@mui/icons-material/ChevronLeft";
import ChevronRight from "@mui/icons-material/ChevronRight";
import Dashboard from "@mui/icons-material/Dashboard";
import Business from "@mui/icons-material/Business";
import People from "@mui/icons-material/People";
import Lock from "@mui/icons-material/Lock";
import { useSession } from "../../../../context/Session";

const NAV_ITEMS = [
	{
		to: "/dashboard",
		label: "Dashboard",
		end: true,
		icon: Dashboard,
		permission: "dashboard",
	},
	{
		to: "/dashboard/organizations",
		label: "Organizations",
		end: false,
		icon: Business,
		permission: "dashboard.organizations",
	},
	{
		to: "/dashboard/users",
		label: "Users",
		end: false,
		icon: People,
		permission: "dashboard.users",
	},
	{
		to: "/dashboard/roles-and-permissions",
		label: "Permissions",
		end: false,
		icon: Lock,
		permission: "dashboard.permissions.manage",
	},
] as const;

export type SidebarProps = {
	expanded: boolean;
	onToggle: () => void;
};

export default function Sidebar({ expanded, onToggle }: SidebarProps) {
	const width = expanded ? 240 : 72;
	const { permissions } = useSession();
	const navItems = NAV_ITEMS.filter((item) =>
		permissions.includes(item.permission),
	);

	return (
		<Box
			component="aside"
			className="dashboard-sidebar"
			aria-label="Dashboard navigation"
			sx={{
				width,
				flexShrink: 0,
				borderRight: 1,
				borderColor: "divider",
				bgcolor: "background.paper",
				display: "flex",
				flexDirection: "column",
				gap: 0.5,
				height: "100vh",
				overflow: "hidden",
				p: 1,
				transition: "width 0.2s ease",
			}}
		>
			<List sx={{ flexGrow: 1, minHeight: 0, py: 0 }}>
				{navItems.map(({ to, label, end, icon: Icon }) => (
					<NavLink
						key={to}
						to={to}
						end={end}
						style={{ textDecoration: "none", color: "inherit" }}
					>
						{({ isActive }) => (
							<ListItemButton
								title={!expanded ? label : undefined}
								selected={isActive}
								sx={{
									borderRadius: 1,
									justifyContent: expanded ? "flex-start" : "center",
									px: 1.5,
									py: 1,
									"&.Mui-selected": {
										bgcolor: "primary.main",
										color: "primary.contrastText",
										"&:hover": { bgcolor: "primary.dark" },
									},
								}}
							>
								<ListItemIcon
									sx={{
										minWidth: expanded ? 56 : "auto",
										color: "inherit",
									}}
								>
									<Icon />
								</ListItemIcon>
								{expanded && <ListItemText primary={label} />}
							</ListItemButton>
						)}
					</NavLink>
				))}
			</List>
			<Box
				sx={{
					display: "flex",
					alignItems: "center",
					justifyContent: expanded ? "flex-end" : "center",
					py: 1,
				}}
			>
				<IconButton
					aria-label={expanded ? "Collapse sidebar" : "Expand sidebar"}
					onClick={onToggle}
					size="small"
				>
					{expanded ? <ChevronLeft /> : <ChevronRight />}
				</IconButton>
			</Box>
		</Box>
	);
}
