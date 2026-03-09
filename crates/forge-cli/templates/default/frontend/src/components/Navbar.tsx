import AppBar from '@mui/material/AppBar'
import Box from '@mui/material/Box'
import Button from '@mui/material/Button'
import Divider from '@mui/material/Divider'
import IconButton from '@mui/material/IconButton'
import MenuIcon from '@mui/icons-material/Menu'
import Toolbar from '@mui/material/Toolbar'
import { useNavigate } from 'react-router-dom'
import { Effect } from 'effect'

import NavbarUserMenu from './NavbarUserMenu'
import { Authentication } from '@/features/authentication/services/Authentication'
import { useComponentLogger } from '@/hooks'
import { getApplicationLayer } from '@/lib/appLayer'

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
  useComponentLogger('Navbar')
  const navigate = useNavigate()
  const canAccessDashboard = false

  const handleLogout = () => {
    Effect.runPromise(
      Effect.gen(function* () {
        const auth = yield* Authentication
        yield* auth.logout()
      }).pipe(Effect.provide(getApplicationLayer())),
    ).then(() => {
      navigate('/authentication/login', { replace: true })
    })
  }

  return (
    <AppBar
      position="static"
      color="default"
      elevation={0}
      data-testid="dashboard-navbar"
    >
      <Toolbar sx={{ minHeight: { xs: 56, sm: 64 }, gap: 1 }}>
        {onOpenSidebar != null && (
          <IconButton
            color="inherit"
            aria-label="Open dashboard menu"
            onClick={onOpenSidebar}
            sx={{ mr: 0.5 }}
            data-testid="navbar-menu-button"
          >
            <MenuIcon />
          </IconButton>
        )}
        <Button
          color="inherit"
          onClick={() => navigate('/')}
          startIcon={<LogoIcon />}
          sx={{ textTransform: 'none', fontSize: '1.125rem', mr: 1 }}
          data-testid="navbar-logo-button"
        >
          {appName}
        </Button>

        <Box sx={{ flexGrow: 1 }} />

        {canAccessDashboard && (
          <>
            <Button
              color="inherit"
              onClick={() => navigate('/dashboard')}
              data-testid="navbar-dashboard-button"
            >
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
          data-testid="theme-toggle-button"
        >
          <ThemeIcon isDark={colorScheme === 'dark'} />
        </IconButton>

        <NavbarUserMenu onLogout={handleLogout} />
      </Toolbar>
    </AppBar>
  )
}
