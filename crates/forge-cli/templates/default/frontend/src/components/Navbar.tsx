import { useState } from 'react'
import AppBar from '@mui/material/AppBar'
import Avatar from '@mui/material/Avatar'
import Box from '@mui/material/Box'
import Button from '@mui/material/Button'
import Divider from '@mui/material/Divider'
import IconButton from '@mui/material/IconButton'
import ListItemIcon from '@mui/material/ListItemIcon'
import ListItemText from '@mui/material/ListItemText'
import Menu from '@mui/material/Menu'
import MenuItem from '@mui/material/MenuItem'
import Toolbar from '@mui/material/Toolbar'
import MenuIcon from '@mui/icons-material/Menu'
import Person from '@mui/icons-material/Person'
import Logout from '@mui/icons-material/Logout'
import Business from '@mui/icons-material/Business'
import { useNavigate } from 'react-router-dom'
import { useAuthentication } from '../context/AuthenticationContext'
import { usePermission } from '../hooks/usePermission'

type NavbarProps = {
  appName?: string
  colorScheme?: 'light' | 'dark'
  onToggleTheme?: () => void
  onOpenSidebar?: () => void
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
      <path d="M12 2L2 22h20L12 2z" fill="currentColor" />
    </svg>
  )
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
        <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z" />
      )}
    </svg>
  )
}

export default function Navbar({
  appName = 'App',
  colorScheme = 'dark',
  onToggleTheme,
  onOpenSidebar,
}: NavbarProps) {
  const navigate = useNavigate()
  const {
    user,
    scopes,
    currentOrgId,
    currentRoleId,
    logout,
    switchScope,
  } = useAuthentication()
  const canAccessDashboard = usePermission('dashboard')
  const [anchorEl, setAnchorEl] = useState<null | HTMLElement>(null)
  const open = Boolean(anchorEl)

  const handleUserClick = (event: React.MouseEvent<HTMLElement>) => {
    setAnchorEl(event.currentTarget)
  }
  const handleClose = () => {
    setAnchorEl(null)
  }
  const handleScope = () => {
    handleClose()
    navigate('/scope')
  }
  const handleSwitchScope = async (
    orgId: string,
    roleId: string,
    roleName: string,
  ) => {
    handleClose()
    await switchScope(orgId, roleId, roleName)
  }
  const handleLogout = async () => {
    handleClose()
    await logout()
    navigate('/authentication/login', { replace: true })
  }

  return (
    <AppBar position="static" color="default" elevation={0}>
      <Toolbar sx={{ minHeight: { xs: 56, sm: 64 }, gap: 1 }}>
        {onOpenSidebar != null && (
          <IconButton
            color="inherit"
            aria-label="Open dashboard menu"
            onClick={onOpenSidebar}
            sx={{ mr: 0.5 }}
          >
            <MenuIcon />
          </IconButton>
        )}
        <Button
          color="inherit"
          onClick={() => navigate('/')}
          startIcon={<LogoIcon />}
          sx={{ textTransform: 'none', fontSize: '1.125rem', mr: 1 }}
        >
          {appName}
        </Button>

        <Box sx={{ flexGrow: 1 }} />

        {canAccessDashboard && (
          <>
            <Button color="inherit" onClick={() => navigate('/dashboard')}>
              Dashboard
            </Button>
            <Divider orientation="vertical" flexItem sx={{ mx: 0.5 }} />
          </>
        )}
        <IconButton
          color="inherit"
          aria-label={
            colorScheme === 'dark'
              ? 'Switch to light mode'
              : 'Switch to dark mode'
          }
          onClick={onToggleTheme}
        >
          <ThemeIcon isDark={colorScheme === 'dark'} />
        </IconButton>

        <IconButton
          color="inherit"
          aria-label="User menu"
          aria-controls={open ? 'user-menu' : undefined}
          aria-haspopup="true"
          aria-expanded={open ? 'true' : undefined}
          onClick={handleUserClick}
          sx={{ ml: 0.5 }}
        >
          <Avatar
            sx={{ width: 32, height: 32, bgcolor: 'primary.main' }}
            alt={user?.email ?? 'User'}
          >
            {user?.email?.charAt(0)?.toUpperCase() ?? 'U'}
          </Avatar>
        </IconButton>
        <Menu
          id="user-menu"
          anchorEl={anchorEl}
          open={open}
          onClose={handleClose}
          anchorOrigin={{ vertical: 'bottom', horizontal: 'right' }}
          transformOrigin={{ vertical: 'top', horizontal: 'right' }}
          slotProps={{ paper: { sx: { minWidth: 220 } } }}
        >
          <MenuItem onClick={handleScope}>
            <ListItemIcon>
              <Person fontSize="small" />
            </ListItemIcon>
            Scope
          </MenuItem>
          <Divider />
          {scopes.map((p) => {
            const isCurrentScope =
              currentOrgId != null &&
              currentRoleId != null &&
              p.org_id === currentOrgId &&
              (p.role_id ?? '') === currentRoleId
            return (
              <MenuItem
                key={p.role_id ?? p.org_id}
                selected={isCurrentScope}
                onClick={() =>
                  handleSwitchScope(
                    p.org_id,
                    p.role_id ?? '',
                    p.role ?? '',
                  )
                }
              >
                <ListItemIcon>
                  <Business fontSize="small" />
                </ListItemIcon>
                <ListItemText
                  primary={`${(p.role ?? '').charAt(0).toUpperCase()}${(p.role ?? '').slice(1)} · ${p.org_name}`}
                />
              </MenuItem>
            )
          })}
          <Divider />
          <MenuItem onClick={handleLogout}>
            <ListItemIcon>
              <Logout fontSize="small" />
            </ListItemIcon>
            Log out
          </MenuItem>
        </Menu>
      </Toolbar>
    </AppBar>
  )
}
