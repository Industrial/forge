/**
 * User dropdown in the navbar: avatar button and menu with scopes list and logout.
 * Fetches scopes when the menu opens and shows the current scope.
 */
import Avatar from '@mui/material/Avatar'
import Divider from '@mui/material/Divider'
import IconButton from '@mui/material/IconButton'
import ListItemIcon from '@mui/material/ListItemIcon'
import ListItemText from '@mui/material/ListItemText'
import Logout from '@mui/icons-material/Logout'
import Menu from '@mui/material/Menu'
import MenuItem from '@mui/material/MenuItem'
import Business from '@mui/icons-material/Business'
import Check from '@mui/icons-material/Check'
import { useNavigate } from 'react-router-dom'
import { useState, useEffect } from 'react'
import { Effect, Option } from 'effect'

import { useAuthStore } from '@/features/authentication/stores'
import { Authentication } from '@/features/authentication/services/Authentication'
import type { Scope } from '@/features/authentication/domain/Scope'
import { getApplicationLayer } from '@/lib/appLayer'

export type NavbarUserMenuProps = {
  /** Called when user chooses "Log out". */
  onLogout?: () => void
}

export default function NavbarUserMenu({ onLogout }: NavbarUserMenuProps) {
  const navigate = useNavigate()
  const authentication = useAuthStore()
  const [anchorEl, setAnchorEl] = useState<null | HTMLElement>(null)
  const [scopes, setScopes] = useState<readonly Scope[]>([])
  const [scopesLoading, setScopesLoading] = useState(false)
  const open = Boolean(anchorEl)

  const user = Option.getOrElse(authentication.user, () => null)
  const currentScope = Option.getOrElse(authentication.currentScope, () => null)

  useEffect(() => {
    if (!open || !user) {
      return
    }
    setScopesLoading(true)
    Effect.runPromise(
      Effect.gen(function* () {
        const auth = yield* Authentication
        const list = yield* auth.getScopes()
        return list
      }).pipe(
        Effect.provide(getApplicationLayer()),
        Effect.catchAll(() => Effect.succeed([])),
      ),
    )
      .then(setScopes)
      .finally(() => setScopesLoading(false))
  }, [open, user])

  const handleUserClick = (event: React.MouseEvent<HTMLElement>) => {
    setAnchorEl(event.currentTarget)
  }

  const handleClose = () => {
    setAnchorEl(null)
  }

  const handleSwitchScope = (orgId: string, roleId: string) => {
    handleClose()
    Effect.runPromise(
      Effect.gen(function* () {
        const auth = yield* Authentication
        yield* auth.selectScope(orgId, roleId)
      }).pipe(Effect.provide(getApplicationLayer())),
    ).then(() => {
      // Optionally refresh or stay; permissions will update via store
    })
  }

  const handleLogout = () => {
    handleClose()
    if (onLogout) {
      onLogout()
    } else {
      Effect.runPromise(
        Effect.gen(function* () {
          const auth = yield* Authentication
          yield* auth.logout()
        }).pipe(Effect.provide(getApplicationLayer())),
      ).then(() => {
        navigate('/authentication/login', { replace: true })
      })
    }
  }

  if (!user) {
    return null
  }

  const displayName = user.email ?? 'User'
  const initial = (user.email?.charAt(0) ?? 'U').toUpperCase()

  return (
    <>
      <IconButton
        color="inherit"
        aria-label="User menu"
        aria-controls={open ? 'navbar-user-menu' : undefined}
        aria-haspopup="true"
        aria-expanded={open ? 'true' : undefined}
        onClick={handleUserClick}
        sx={{ ml: 0.5 }}
        data-testid="navbar-user-menu-button"
      >
        <Avatar
          sx={{ width: 32, height: 32, bgcolor: 'primary.main' }}
          alt={displayName}
        >
          {initial}
        </Avatar>
      </IconButton>
      <Menu
        id="navbar-user-menu"
        anchorEl={anchorEl}
        open={open}
        onClose={handleClose}
        anchorOrigin={{ vertical: 'bottom', horizontal: 'right' }}
        transformOrigin={{ vertical: 'top', horizontal: 'right' }}
        slotProps={{ paper: { sx: { minWidth: 260 } } }}
        data-testid="navbar-user-menu"
      >
        <MenuItem disabled sx={{ opacity: 1 }}>
          <ListItemText primary={displayName} secondary="Signed in" />
        </MenuItem>
        <Divider />
        {scopesLoading ? (
          <MenuItem disabled data-testid="navbar-user-menu-scopes-loading">
            <ListItemText primary="Loading scopes…" />
          </MenuItem>
        ) : scopes.length > 0 ? (
          scopes.map((scope) => {
            const roleId = scope.role_id ?? ''
            const isCurrent =
              currentScope != null &&
              currentScope.organizationId === scope.org_id &&
              currentScope.roleId === roleId
            const label = `${(scope.role ?? '').charAt(0).toUpperCase()}${(scope.role ?? '').slice(1)} · ${scope.org_name}`
            return (
              <MenuItem
                key={`${scope.org_id}-${roleId}`}
                selected={isCurrent}
                onClick={() => handleSwitchScope(scope.org_id, roleId)}
                data-testid={`navbar-user-menu-scope-${scope.org_id}-${roleId}`}
              >
                <ListItemIcon>
                  <Business fontSize="small" />
                </ListItemIcon>
                <ListItemText
                  primary={label}
                  secondary={isCurrent ? 'Current' : undefined}
                />
                {isCurrent && (
                  <Check color="primary" fontSize="small" sx={{ ml: 1 }} />
                )}
              </MenuItem>
            )
          })
        ) : null}
        <Divider />
        <MenuItem onClick={handleLogout} data-testid="navbar-user-menu-logout">
          <ListItemIcon>
            <Logout fontSize="small" />
          </ListItemIcon>
          Log out
        </MenuItem>
      </Menu>
    </>
  )
}
