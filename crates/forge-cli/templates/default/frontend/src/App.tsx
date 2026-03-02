/*
 * Copyright 2024 Adobe. All rights reserved.
 * This file is licensed to you under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License. You may obtain a copy
 * of the License at http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software distributed under
 * the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR REPRESENTATIONS
 * OF ANY KIND, either express or implied. See the License for the specific language
 * governing permissions and limitations under the License.
 */

import { useEffect, useState } from "react";
import { Outlet, Route, Routes, useNavigate } from "react-router-dom";
import { Provider } from "@react-spectrum/s2";
import type { NavigateOptions } from "react-router-dom";
import { SessionProvider } from "./context/Session";
import Layout from "./layouts/Layout";
import DashboardLayout from "./features/dashboard/layouts/DashboardLayout/DashboardLayout";
import AuthenticationLayout from "./features/authentication/layouts/AuthenticationLayout/AuthenticationLayout";
import ProtectedRoute from "./components/ProtectedRoute";
import LoginPage from "./features/authentication/pages/LoginPage/LoginPage";
import RegisterPage from "./features/authentication/pages/RegisterPage/RegisterPage";
import HomePage from "./features/home/pages/HomePage/HomePage";
import DashboardPage from "./features/dashboard/pages/DashboardPage/DashboardPage";
import OrganizationsPage from "./features/dashboard/pages/OrganizationsPage/OrganizationsPage";
import UsersPage from "./features/dashboard/pages/UsersPage/UsersPage";
import PermissionsPage from "./features/dashboard/pages/PermissionsPage/PermissionsPage";
import ProfilePage from "./features/profile/pages/ProfilePage/ProfilePage";
import WebsocketsDemoPage from "./features/websockets-demo/pages/WebsocketsDemoPage/WebsocketsDemoPage";

declare module "@react-spectrum/s2" {
	interface RouterConfig {
		routerOptions: NavigateOptions;
	}
}

const STORAGE_KEY = "spectrum-color-scheme";

function App() {
	const navigate = useNavigate();
	const [colorScheme, setColorScheme] = useState<"light" | "dark">(() => {
		if (typeof window !== "undefined") {
			const stored = localStorage.getItem(STORAGE_KEY);
			if (stored === "light" || stored === "dark") return stored;
		}
		return "dark";
	});

	useEffect(() => {
		document.documentElement.setAttribute("data-color-scheme", colorScheme);
		if (typeof window !== "undefined") {
			localStorage.setItem(STORAGE_KEY, colorScheme);
		}
	}, [colorScheme]);

	const layoutProps = {
		colorScheme,
		onToggleTheme: () =>
			setColorScheme((s) => (s === "dark" ? "light" : "dark")),
	};

	return (
		<SessionProvider>
			<Provider
				elementType="main"
				locale="en-US"
				colorScheme={colorScheme}
				router={{
					navigate: (url: string) => navigate(url),
					useHref: (to: string) => to,
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
								<DashboardLayout {...layoutProps}>
									<Outlet />
								</DashboardLayout>
							</ProtectedRoute>
						}
					>
						<Route index element={<DashboardPage />} />
						<Route path="organizations" element={<OrganizationsPage />} />
						<Route path="users" element={<UsersPage />} />
						<Route path="roles-and-permissions" element={<PermissionsPage />} />
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
			</Provider>
		</SessionProvider>
	);
}

export default App;
