//! Migrations applied inside each tenant's dedicated schema (RBAC, and the
//! domain tables added as the model grows).

pub mod m20260601_000003_create_permissions;
pub mod m20260601_000004_create_sector;
pub mod m20260601_000005_create_role;
pub mod m20260601_000006_create_collaborator;
pub mod m20260601_000007_create_feedback;
pub mod m20260601_000008_create_expectation_contract_item;
pub mod m20260601_000009_create_feedback_behavior;
pub mod m20260601_000010_create_annotation;
