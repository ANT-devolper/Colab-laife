//! RBAC entities. These live in each tenant's schema (not `public`), so they
//! are used through a tenant-scoped connection. A user reaches a resource via
//! profile → task → resource (see the `TenantMigrator` migration).

pub mod profile;
pub mod profile_task;
pub mod profile_user;
pub mod resource;
pub mod task;
pub mod task_resource;
