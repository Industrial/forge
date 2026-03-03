import { useEffect, useState } from "react";
import { Outlet, Route, Routes } from "react-router-dom";
import { createTheme, ThemeProvider } from "@mui/material/styles";
import CssBaseline from "@mui/material/CssBaseline";
import Box from "@mui/material/Box";
import { LiveWsProvider } from "./context/LiveWs";
import { SessionProvider } from "./context/Session";
import Layout from "./layouts/Layout";
import DashboardLayout from "./features/dashboard/layouts/DashboardLayout/DashboardLayout";
import AuthenticationLayout from "./features/authentication/layouts/AuthenticationLayout/AuthenticationLayout";
import ProtectedRoute from "./components/ProtectedRoute";
import DashboardProfileGuard from "./components/DashboardProfileGuard";
import DashboardPermissionGuard from "./components/DashboardPermissionGuard";
import LoginPage from "./features/authentication/pages/LoginPage/LoginPage";
import SelectProfilePage from "./features/authentication/pages/SelectProfilePage/SelectProfilePage";
import RegisterPage from "./features/authentication/pages/RegisterPage/RegisterPage";
import HomePage from "./features/home/pages/HomePage/HomePage";
import DashboardPage from "./features/dashboard/pages/DashboardPage/DashboardPage";
import OrganizationsPage from "./features/dashboard/pages/OrganizationsPage/OrganizationsPage";
import UsersPage from "./features/dashboard/pages/UsersPage/UsersPage";
import RolesPage from "./features/dashboard/pages/RolesPage/RolesPage";
import PermissionsPage from "./features/dashboard/pages/PermissionsPage/PermissionsPage";
import AuditLogPage from "./features/dashboard/pages/AuditLogPage/AuditLogPage";
import ProfilePage from "./features/profile/pages/ProfilePage/ProfilePage";
import WebsocketsDemoPage from "./features/websockets-demo/pages/WebsocketsDemoPage/WebsocketsDemoPage";

const STORAGE_KEY = "mui-color-scheme";

function App() {
	const [mode, setMode] = useState<"light" | "dark">(() => {
		if (typeof window !== "undefined") {
			const stored = localStorage.getItem(STORAGE_KEY);
			if (stored === "light" || stored === "dark") return stored;
		}
		return "dark";
	});

	useEffect(() => {
		if (typeof window !== "undefined") {
			localStorage.setItem(STORAGE_KEY, mode);
		}
	}, [mode]);

	const theme = createTheme({
		palette: { mode },
	});

	const layoutProps = {
		colorScheme: mode,
		onToggleTheme: () => setMode((m) => (m === "dark" ? "light" : "dark")),
	};

	return (
		<SessionProvider>
			<LiveWsProvider>
			<ThemeProvider theme={theme}>
				<CssBaseline />
				<Box
					component="main"
					sx={{
						height: "100vh",
						minHeight: "100vh",
						display: "flex",
						flexDirection: "column",
					}}
				>
					<Routes>
					<Route
						path="/login"
						element={
							<AuthenticationLayout>
								<LoginPage />
							</AuthenticationLayout>
						}
					/>
					<Route
						path="/register"
						element={
							<AuthenticationLayout>
								<RegisterPage />
							</AuthenticationLayout>
						}
					/>
					<Route
						path="/select-profile"
						element={
							<ProtectedRoute>
								<AuthenticationLayout>
									<SelectProfilePage />
								</AuthenticationLayout>
							</ProtectedRoute>
						}
					/>
					<Route path="/ws-demo" element={<WebsocketsDemoPage />} />
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
								<DashboardProfileGuard>
									<DashboardLayout {...layoutProps}>
										<Outlet />
									</DashboardLayout>
								</DashboardProfileGuard>
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
									permission={[
										"dashboard.organizations.read",
										"dashboard.organizations.write",
									]}
								>
									<OrganizationsPage />
								</DashboardPermissionGuard>
							}
						/>
						<Route
							path="users"
							element={
								<DashboardPermissionGuard
									permission={[
										"dashboard.users.read",
										"dashboard.users.write",
									]}
								>
									<UsersPage />
								</DashboardPermissionGuard>
							}
						/>
						<Route
							path="roles"
							element={
								<DashboardPermissionGuard
									permission={[
										"dashboard.roles.read",
										"dashboard.roles.write",
									]}
								>
									<RolesPage />
								</DashboardPermissionGuard>
							}
						/>
						<Route
							path="roles-and-permissions"
							element={
								<DashboardPermissionGuard
									permission={[
										"dashboard.permissions.read",
										"dashboard.permissions.write",
									]}
								>
									<PermissionsPage />
								</DashboardPermissionGuard>
							}
						/>
						<Route
							path="audit-log"
							element={
								<DashboardPermissionGuard permission="dashboard.audit.read">
									<AuditLogPage />
								</DashboardPermissionGuard>
							}
						/>
					</Route>
					<Route
						path="/profile"
						element={
							<ProtectedRoute>
								<Layout {...layoutProps}>
									<ProfilePage />
								</Layout>
							</ProtectedRoute>
						}
					/>
					</Routes>
				</Box>
			</ThemeProvider>
			</LiveWsProvider>
		</SessionProvider>
	);
}

export default App;
