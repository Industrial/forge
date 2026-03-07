import { Outlet, Route, Routes } from 'react-router-dom'
import { createTheme } from '@mui/material/styles'
import Box from '@mui/material/Box'

import Layout from '@/layouts/Layout'
import DashboardLayout from '@/features/dashboard/layouts/DashboardLayout/DashboardLayout'
import AuthenticationLayout from '@/features/authentication/layouts/AuthenticationLayout/AuthenticationLayout'
import ProtectedRoute from '@/components/ProtectedRoute'
import GuestRoute from '@/components/GuestRoute'
import DashboardScopeGuard from '@/features/dashboard/components/DashboardScopeGuard'
import SelectScopeOnlyGuard from '@/features/authentication/components/SelectScopeOnlyGuard'
import DashboardPermissionGuard from '@/features/dashboard/components/DashboardPermissionGuard'
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
import ScopePage from '@/features/profile/pages/ProfilePage/ProfilePage'
import { useColorSchemeMode } from '@/hooks/useColorScheme'
import { Providers } from '@/Providers'
import { SubscriptionStreamRunner } from '@/components/SubscriptionStreamRunner'

function App() {
  console.log('App')

  const [colorSchemeMode, setColorSchemeMode] = useColorSchemeMode()

  const theme = createTheme({
    palette: {
      mode: colorSchemeMode,
    },
  })

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
            <Route
              index
              element={
                <DashboardPermissionGuard permission="dashboard">
                  <DashboardPage />
                </DashboardPermissionGuard>
              }
            />
            <Route
              path="organizations"
              element={
                <DashboardPermissionGuard
                  permission={['organization.read', 'organization.create']}
                >
                  <OrganizationsPage />
                </DashboardPermissionGuard>
              }
            />
            <Route
              path="users"
              element={
                <DashboardPermissionGuard
                  permission={['user.read', 'user.create']}
                >
                  <UsersPage />
                </DashboardPermissionGuard>
              }
            />
            <Route
              path="roles"
              element={
                <DashboardPermissionGuard
                  permission={['role.read', 'role.create']}
                >
                  <RolesPage />
                </DashboardPermissionGuard>
              }
            />
            <Route
              path="roles-and-permissions"
              element={
                <DashboardPermissionGuard
                  permission={['permission.read', 'permission.create']}
                >
                  <PermissionsPage />
                </DashboardPermissionGuard>
              }
            />
            <Route
              path="audit-log"
              element={
                <DashboardPermissionGuard permission="audit.read">
                  <AuditLogPage />
                </DashboardPermissionGuard>
              }
            />
          </Route>
          <Route
            path="/scope"
            element={
              <ProtectedRoute>
                <Layout {...layoutProps}>
                  <ScopePage />
                </Layout>
              </ProtectedRoute>
            }
          />
        </Routes>
      </Box>
    </Providers>
  )
}

export default App
