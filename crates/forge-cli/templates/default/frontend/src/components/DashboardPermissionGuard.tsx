import React from "react";
import { Navigate } from "react-router-dom";
import { useSession } from "../context/Session";

type DashboardPermissionGuardProps = {
	/** Single permission or list; access allowed if user has any of them (e.g. .read or .write). */
	permission: string | string[];
	children: React.ReactNode;
};

function hasAny(
	userPermissions: string[],
	required: string | string[],
): boolean {
	const list = Array.isArray(required) ? required : [required];
	return list.some((p) => userPermissions.includes(p));
}

/**
 * Renders children only if the current session has at least one of the required permissions.
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

	if (!hasAny(permissions, permission)) {
		const to =
			(Array.isArray(permission) ? permission[0] : permission) === "dashboard"
				? "/"
				: "/dashboard";
		return <Navigate to={to} replace />;
	}

	return <>{children}</>;
}
