import React from "react";
import { Navigate } from "react-router-dom";
import { useSession } from "../context/Session";

type DashboardPermissionGuardProps = {
	permission: string;
	children: React.ReactNode;
};

/**
 * Renders children only if the current session has the required permission.
 * Otherwise redirects to /dashboard. Use for dashboard sub-routes.
 */
export default function DashboardPermissionGuard({
	permission,
	children,
}: DashboardPermissionGuardProps) {
	const { permissions, loading } = useSession();

	if (loading) {
		return null;
	}

	if (!permissions.includes(permission)) {
		return (
			<Navigate
				to={permission === "dashboard" ? "/" : "/dashboard"}
				replace
			/>
		);
	}

	return <>{children}</>;
}
