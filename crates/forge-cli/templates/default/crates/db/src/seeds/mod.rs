//! Seeds for template app. Every user has at least one organization (membership).
//! "Personal org" = first org by membership insert order (e.g. Default for most users).
//! Active profile is stored in session, not on user row.

pub mod s20220101_000001_seed_users;
pub mod s20220101_000002_seed_role_permissions;
pub mod s20220101_000003_seed_user_global_roles;
