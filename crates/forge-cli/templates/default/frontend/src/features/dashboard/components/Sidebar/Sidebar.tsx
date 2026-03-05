import { NavLink } from 'react-router-dom'
import Box from '@mui/material/Box'
import IconButton from '@mui/material/IconButton'
import List from '@mui/material/List'
import ListItemButton from '@mui/material/ListItemButton'
import ListItemIcon from '@mui/material/ListItemIcon'
import ListItemText from '@mui/material/ListItemText'
import ChevronLeft from '@mui/icons-material/ChevronLeft'
import ChevronRight from '@mui/icons-material/ChevronRight'
import Dashboard from '@mui/icons-material/Dashboard'
import Business from '@mui/icons-material/Business'
import People from '@mui/icons-material/People'
import Badge from '@mui/icons-material/Badge'
import Lock from '@mui/icons-material/Lock'
import History from '@mui/icons-material/History'
import { useSession } from '../../../../context/Session'

/** Show nav item if user has any of these permissions (.read = view, .write = modify). */
const NAV_ITEMS = [
  {
    to: '/dashboard',
    label: 'Dashboard',
    end: true,
    icon: Dashboard,
    permissions: ['dashboard'],
  },
  {
    to: '/dashboard/organizations',
    label: 'Organizations',
    end: false,
    icon: Business,
    permissions: [
      'dashboard.organizations.read',
      'dashboard.organizations.write',
    ],
  },
  {
    to: '/dashboard/users',
    label: 'Users',
    end: false,
    icon: People,
    permissions: ['dashboard.users.read', 'dashboard.users.write'],
  },
  {
    to: '/dashboard/roles',
    label: 'Roles',
    end: false,
    icon: Badge,
    permissions: ['dashboard.roles.read', 'dashboard.roles.write'],
  },
  {
    to: '/dashboard/roles-and-permissions',
    label: 'Permissions',
    end: false,
    icon: Lock,
    permissions: ['dashboard.permissions.read', 'dashboard.permissions.write'],
  },
  {
    to: '/dashboard/audit-log',
    label: 'Audit log',
    end: false,
    icon: History,
    permissions: ['dashboard.audit.read'],
  },
] as const

export type SidebarProps = {
  expanded: boolean
  onToggle: () => void
  hideToggle?: boolean
  disableBorder?: boolean
  /** When true, sidebar fills container width (e.g. inside mobile drawer). */
  fullWidth?: boolean
  /** @deprecated Legacy API; layout still passes for outlet context. Prefer Effect HttpClient. */
  api?: (url: string, options?: RequestInit) => Promise<Response>
}

export default function Sidebar({
  expanded,
  onToggle,
  hideToggle = false,
  disableBorder = false,
  fullWidth = false,
}: SidebarProps) {
  const width = fullWidth ? '100%' : expanded ? 240 : 72
  const { permissions } = useSession()
  const navItems = NAV_ITEMS.filter((item) =>
    item.permissions.some((p) => permissions.includes(p)),
  )

  return (
    <Box
      component="aside"
      className="dashboard-sidebar"
      aria-label="Dashboard navigation"
      sx={{
        width,
        flexShrink: 0,
        ...(disableBorder ? {} : { borderRight: 1, borderColor: 'divider' }),
        bgcolor: 'background.paper',
        display: 'flex',
        flexDirection: 'column',
        gap: 0.5,
        height: '100vh',
        overflow: 'hidden',
        p: 1,
        transition: 'width 0.2s ease',
      }}
    >
      <List sx={{ flexGrow: 1, minHeight: 0, py: 0 }}>
        {navItems.map(({ to, label, end, icon: Icon }) => (
          <NavLink
            key={to}
            to={to}
            end={end}
            style={{ textDecoration: 'none', color: 'inherit' }}
          >
            {({ isActive }) => (
              <ListItemButton
                title={!expanded ? label : undefined}
                selected={isActive}
                sx={{
                  borderRadius: 1,
                  justifyContent: expanded ? 'flex-start' : 'center',
                  px: 1.5,
                  py: 1,
                  '&.Mui-selected': {
                    bgcolor: 'primary.main',
                    color: 'primary.contrastText',
                    '&:hover': { bgcolor: 'primary.dark' },
                  },
                }}
              >
                <ListItemIcon
                  sx={{
                    minWidth: expanded ? 56 : 'auto',
                    color: 'inherit',
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
      {!hideToggle && (
        <Box
          sx={{
            display: 'flex',
            alignItems: 'center',
            justifyContent: expanded ? 'flex-end' : 'center',
            py: 1,
          }}
        >
          <IconButton
            aria-label={expanded ? 'Collapse sidebar' : 'Expand sidebar'}
            onClick={onToggle}
            size="small"
          >
            {expanded ? <ChevronLeft /> : <ChevronRight />}
          </IconButton>
        </Box>
      )}
    </Box>
  )
}
