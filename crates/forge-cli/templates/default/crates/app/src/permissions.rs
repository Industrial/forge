//! Code-defined dashboard permission keys. Used for resolution and for listing in APIs.
//! Resources have .read (view/list) and .write (create/update/delete) where applicable.
//! Admin org role may have "all.read" and "all.write" (grants all read/write respectively).

pub const DASHBOARD_PERMISSIONS: &[&str] = &[
  "dashboard",
  "dashboard.organizations.read",
  "dashboard.organizations.write",
  "dashboard.users.read",
  "dashboard.users.write",
  "dashboard.roles.read",
  "dashboard.roles.write",
  "dashboard.permissions.read",
  "dashboard.permissions.write",
  "dashboard.audit.read",
  "all.read",
  "all.write",
];
