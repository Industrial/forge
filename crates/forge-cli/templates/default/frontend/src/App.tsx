import React from 'react'
import { Outlet, Route, Routes } from 'react-router-dom'
import { useComponentLogger } from '@/hooks'
import { createTheme } from '@mui/material/styles'
import Box from '@mui/material/Box'

import Layout from '@/layouts/Layout'
import DashboardLayout from '@/features/dashboard/layouts/DashboardLayout/DashboardLayout'
import AuthenticationLayout from '@/features/authentication/layouts/AuthenticationLayout/AuthenticationLayout'
import ProtectedRoute from '@/components/ProtectedRoute'
import GuestRoute from '@/components/GuestRoute'
import DashboardScopeGuard from '@/features/dashboard/components/DashboardScopeGuard'
import SelectScopeOnlyGuard from '@/features/authentication/components/SelectScopeOnlyGuard'
import { PermissionGuard } from '@/components/PermissionGuard'
import LoginPage from '@/features/authentication/pages/LoginPage/LoginPage'
import SelectScopePage from '@/features/authentication/pages/SelectScopePage/SelectScopePage'
import RegisterPage from '@/features/authentication/pages/RegisterPage/RegisterPage'
import HomePage from '@/features/home/pages/HomePage/HomePage'
import DashboardPage from '@/features/dashboard/pages/DashboardPage/DashboardPage'
import OrganizationsPage from '@/features/dashboard/pages/OrganizationsPage/OrganizationsPage'
import UsersPage from '@/features/dashboard/pages/UsersPage/UsersPage'
import RolesPage from '@/features/dashboard/pages/RolesPage/RolesPage'
import PermissionsPage from '@/features/dashboard/pages/PermissionsPage/PermissionsPage'
import AuditLogPage from '@/features/dashboard/pages/AuditLogPage/AuditLogPage'
import { useColorSchemeMode } from '@/hooks/useColorScheme'
import { Providers } from '@/Providers'
import { SubscriptionStreamRunner } from '@/components/SubscriptionStreamRunner'

function App() {
  useComponentLogger('App')
  const [colorSchemeMode, setColorSchemeMode] = useColorSchemeMode()

  const theme = createTheme({
    palette: {
      mode: colorSchemeMode,
    },
  })

  // Set data-theme attribute on body for theme toggle test
  React.useEffect(() => {
    if (typeof document !== 'undefined') {
      document.body.setAttribute('data-theme', colorSchemeMode)
    }
  }, [colorSchemeMode])

  const layoutProps = {
    colorScheme: colorSchemeMode,
    onToggleTheme: () =>
      setColorSchemeMode((m) => (m === 'dark' ? 'light' : 'dark')),
  }

  return (
    <Providers theme={theme}>
      <Box
        component="main"
        sx={{
          height: '100vh',
          minHeight: '100vh',
          display: 'flex',
          flexDirection: 'column',
        }}
      >
        <Routes>
          <Route
            path="/authentication/login"
            element={
              <GuestRoute>
                <AuthenticationLayout>
                  <LoginPage />
                </AuthenticationLayout>
              </GuestRoute>
            }
          />
          <Route
            path="/authentication/register"
            element={
              <GuestRoute>
                <AuthenticationLayout>
                  <RegisterPage />
                </AuthenticationLayout>
              </GuestRoute>
            }
          />
          <Route
            path="/authentication/select-scope"
            element={
              <ProtectedRoute>
                <SelectScopeOnlyGuard>
                  <AuthenticationLayout>
                    <SelectScopePage />
                  </AuthenticationLayout>
                </SelectScopeOnlyGuard>
              </ProtectedRoute>
            }
          />
          <Route
            path="/"
            element={
              <ProtectedRoute>
                <Layout {...layoutProps}>
                  <HomePage />
                </Layout>
              </ProtectedRoute>
            }
          />
          <Route
            path="/dashboard"
            element={
              <ProtectedRoute>
                <DashboardScopeGuard>
                  <>
                    <SubscriptionStreamRunner />
                    <DashboardLayout {...layoutProps}>
                      <Outlet />
                    </DashboardLayout>
                  </>
                </DashboardScopeGuard>
              </ProtectedRoute>
            }
          >
            <Route index element={<DashboardPage />} />
            <Route
              path="organizations"
              element={
                <PermissionGuard
                  permissions={['organization.read', 'organization.create']}
                >
                  <OrganizationsPage />
                </PermissionGuard>
              }
            />
            <Route
              path="users"
              element={
                <PermissionGuard permissions={['user.read', 'user.create']}>
                  <UsersPage />
                </PermissionGuard>
              }
            />
            <Route
              path="roles"
              element={
                <PermissionGuard permissions={['role.read', 'role.create']}>
                  <RolesPage />
                </PermissionGuard>
              }
            />
            <Route
              path="roles-and-permissions"
              element={
                <PermissionGuard
                  permissions={['permission.read', 'permission.create']}
                >
                  <PermissionsPage />
                </PermissionGuard>
              }
            />
            <Route
              path="audit-log"
              element={
                <PermissionGuard permissions={['audit.read']}>
                  <AuditLogPage />
                </PermissionGuard>
              }
            />
          </Route>
        </Routes>
      </Box>
    </Providers>
  )
}

export default App
