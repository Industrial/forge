//! Seeds: 000001 admin (db-only), 000002 Default org+users, 000003 CoolOrg, 000004 multi-org user, 000005 audit log.
//! 002–004 use app handler impls so the same logic as the API runs during seed.

pub mod s20220101_000001_seed_admin;
pub mod s20220101_000002_seed_default_users;
pub mod s20220101_000003_seed_coolorg_users;
pub mod s20220101_000004_seed_multi_org_user;
pub mod s20220101_000005_seed_audit_log;
