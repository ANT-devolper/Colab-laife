//! RBAC catalog and tenant seeding. `Resource` is the type-safe set of
//! protected actions; `seed_tenant_rbac` plants the minimal catalog and an
//! "administrator" profile (granting everything) into a freshly migrated tenant
//! schema, and links the tenant's admin user to it (see ADR 0010).

use entity::permission::{profile, profile_task, profile_user, resource, task, task_resource};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ConnectionTrait, DatabaseBackend, DatabaseConnection,
    DbErr, Statement, TransactionTrait,
};
use uuid::Uuid;

/// Name shared by the seeded "administrator" task and profile.
const ADMINISTRATOR: &str = "administrator";

/// A protected action, identified by a stable `domain.action` name. The guard
/// checks against these; the catalog is seeded into every tenant.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Resource {
    UserRead,
    UserCreate,
    UserUpdate,
    UserDelete,
    ProfileRead,
    ProfileManage,
    SectorRead,
    SectorCreate,
    SectorUpdate,
    SectorDelete,
    RoleRead,
    RoleCreate,
    RoleUpdate,
    RoleDelete,
    CollaboratorRead,
    CollaboratorCreate,
    CollaboratorUpdate,
    CollaboratorDelete,
    FeedbackRead,
    FeedbackCreate,
    FeedbackUpdate,
    FeedbackDelete,
    ExpectationRead,
    ExpectationCreate,
    ExpectationUpdate,
    ExpectationDelete,
    FeedbackBehaviorRead,
    FeedbackBehaviorCreate,
    FeedbackBehaviorUpdate,
    FeedbackBehaviorDelete,
    AnnotationRead,
    AnnotationCreate,
    AnnotationUpdate,
    AnnotationDelete,
}

impl Resource {
    /// The persisted identifier (`permission_resources.name`).
    pub const fn name(self) -> &'static str {
        match self {
            Self::UserRead => "user.read",
            Self::UserCreate => "user.create",
            Self::UserUpdate => "user.update",
            Self::UserDelete => "user.delete",
            Self::ProfileRead => "profile.read",
            Self::ProfileManage => "profile.manage",
            Self::SectorRead => "sector.read",
            Self::SectorCreate => "sector.create",
            Self::SectorUpdate => "sector.update",
            Self::SectorDelete => "sector.delete",
            Self::RoleRead => "role.read",
            Self::RoleCreate => "role.create",
            Self::RoleUpdate => "role.update",
            Self::RoleDelete => "role.delete",
            Self::CollaboratorRead => "collaborator.read",
            Self::CollaboratorCreate => "collaborator.create",
            Self::CollaboratorUpdate => "collaborator.update",
            Self::CollaboratorDelete => "collaborator.delete",
            Self::FeedbackRead => "feedback.read",
            Self::FeedbackCreate => "feedback.create",
            Self::FeedbackUpdate => "feedback.update",
            Self::FeedbackDelete => "feedback.delete",
            Self::ExpectationRead => "expectation.read",
            Self::ExpectationCreate => "expectation.create",
            Self::ExpectationUpdate => "expectation.update",
            Self::ExpectationDelete => "expectation.delete",
            Self::FeedbackBehaviorRead => "feedback_behavior.read",
            Self::FeedbackBehaviorCreate => "feedback_behavior.create",
            Self::FeedbackBehaviorUpdate => "feedback_behavior.update",
            Self::FeedbackBehaviorDelete => "feedback_behavior.delete",
            Self::AnnotationRead => "annotation.read",
            Self::AnnotationCreate => "annotation.create",
            Self::AnnotationUpdate => "annotation.update",
            Self::AnnotationDelete => "annotation.delete",
        }
    }

    /// A human-readable label stored alongside the resource.
    pub const fn label(self) -> &'static str {
        match self {
            Self::UserRead => "View users",
            Self::UserCreate => "Create users",
            Self::UserUpdate => "Update users",
            Self::UserDelete => "Remove users",
            Self::ProfileRead => "View profiles",
            Self::ProfileManage => "Manage profiles",
            Self::SectorRead => "View sectors",
            Self::SectorCreate => "Create sectors",
            Self::SectorUpdate => "Update sectors",
            Self::SectorDelete => "Remove sectors",
            Self::RoleRead => "View roles",
            Self::RoleCreate => "Create roles",
            Self::RoleUpdate => "Update roles",
            Self::RoleDelete => "Remove roles",
            Self::CollaboratorRead => "View collaborators",
            Self::CollaboratorCreate => "Create collaborators",
            Self::CollaboratorUpdate => "Update collaborators",
            Self::CollaboratorDelete => "Remove collaborators",
            Self::FeedbackRead => "View feedback",
            Self::FeedbackCreate => "Create feedback",
            Self::FeedbackUpdate => "Update feedback",
            Self::FeedbackDelete => "Remove feedback",
            Self::ExpectationRead => "View expectation contract",
            Self::ExpectationCreate => "Create expectation contract items",
            Self::ExpectationUpdate => "Update expectation contract items",
            Self::ExpectationDelete => "Remove expectation contract items",
            Self::FeedbackBehaviorRead => "View feedback behaviors",
            Self::FeedbackBehaviorCreate => "Create feedback behaviors",
            Self::FeedbackBehaviorUpdate => "Update feedback behaviors",
            Self::FeedbackBehaviorDelete => "Remove feedback behaviors",
            Self::AnnotationRead => "View annotations",
            Self::AnnotationCreate => "Create annotations",
            Self::AnnotationUpdate => "Update annotations",
            Self::AnnotationDelete => "Remove annotations",
        }
    }

    /// Every resource seeded into a new tenant. Grows as domains are added.
    pub const fn catalog() -> &'static [Resource] {
        &[
            Self::UserRead,
            Self::UserCreate,
            Self::UserUpdate,
            Self::UserDelete,
            Self::ProfileRead,
            Self::ProfileManage,
            Self::SectorRead,
            Self::SectorCreate,
            Self::SectorUpdate,
            Self::SectorDelete,
            Self::RoleRead,
            Self::RoleCreate,
            Self::RoleUpdate,
            Self::RoleDelete,
            Self::CollaboratorRead,
            Self::CollaboratorCreate,
            Self::CollaboratorUpdate,
            Self::CollaboratorDelete,
            Self::FeedbackRead,
            Self::FeedbackCreate,
            Self::FeedbackUpdate,
            Self::FeedbackDelete,
            Self::ExpectationRead,
            Self::ExpectationCreate,
            Self::ExpectationUpdate,
            Self::ExpectationDelete,
            Self::FeedbackBehaviorRead,
            Self::FeedbackBehaviorCreate,
            Self::FeedbackBehaviorUpdate,
            Self::FeedbackBehaviorDelete,
            Self::AnnotationRead,
            Self::AnnotationCreate,
            Self::AnnotationUpdate,
            Self::AnnotationDelete,
        ]
    }
}

/// Whether `user_id` is granted `resource`, by walking
/// `profile_users → profile_tasks → task_resources → resources` in the tenant
/// schema. `conn` must target the tenant's `search_path`. The admin bypass is
/// applied by the caller (the guard), not here.
pub async fn has_permission(
    conn: &impl ConnectionTrait,
    user_id: Uuid,
    resource: Resource,
) -> Result<bool, DbErr> {
    let stmt = Statement::from_sql_and_values(
        DatabaseBackend::Postgres,
        "SELECT 1 \
         FROM permission_profile_users pu \
         JOIN permission_profile_tasks pt ON pt.profile_id = pu.profile_id \
         JOIN permission_task_resources tr ON tr.task_id = pt.task_id \
         JOIN permission_resources r ON r.id = tr.resource_id \
         WHERE pu.user_id = $1 AND r.name = $2 \
         LIMIT 1",
        [user_id.into(), resource.name().to_owned().into()],
    );
    Ok(conn.query_one(stmt).await?.is_some())
}

/// Seeds the resource catalog plus an "administrator" profile that grants every
/// resource, and links `admin_user_id` to that profile. Runs against the
/// tenant's `search_path` connection, atomically in one transaction.
pub async fn seed_tenant_rbac(conn: &DatabaseConnection, admin_user_id: Uuid) -> Result<(), DbErr> {
    let txn = conn.begin().await?;

    // Resource catalog.
    let mut resource_ids = Vec::with_capacity(Resource::catalog().len());
    for entry in Resource::catalog() {
        let id = Uuid::new_v4();
        resource::ActiveModel {
            id: Set(id),
            name: Set(entry.name().to_owned()),
            label: Set(entry.label().to_owned()),
            ..Default::default()
        }
        .insert(&txn)
        .await?;
        resource_ids.push(id);
    }

    // A single task granting the whole catalog.
    let task_id = Uuid::new_v4();
    task::ActiveModel {
        id: Set(task_id),
        name: Set(ADMINISTRATOR.to_owned()),
        label: Set("All permissions".to_owned()),
        ..Default::default()
    }
    .insert(&txn)
    .await?;
    for resource_id in resource_ids {
        task_resource::ActiveModel {
            id: Set(Uuid::new_v4()),
            task_id: Set(task_id),
            resource_id: Set(resource_id),
            ..Default::default()
        }
        .insert(&txn)
        .await?;
    }

    // The "administrator" profile holds that task and is granted to the admin.
    let profile_id = Uuid::new_v4();
    profile::ActiveModel {
        id: Set(profile_id),
        name: Set(ADMINISTRATOR.to_owned()),
        label: Set("Administrator".to_owned()),
        ..Default::default()
    }
    .insert(&txn)
    .await?;
    profile_task::ActiveModel {
        id: Set(Uuid::new_v4()),
        profile_id: Set(profile_id),
        task_id: Set(task_id),
        ..Default::default()
    }
    .insert(&txn)
    .await?;
    profile_user::ActiveModel {
        id: Set(Uuid::new_v4()),
        user_id: Set(admin_user_id),
        profile_id: Set(profile_id),
        ..Default::default()
    }
    .insert(&txn)
    .await?;

    txn.commit().await
}

#[cfg(test)]
mod tests {
    use super::Resource;

    #[test]
    fn resource_names_are_domain_dot_action() {
        assert_eq!(Resource::UserRead.name(), "user.read");
        assert_eq!(Resource::ProfileManage.name(), "profile.manage");
    }

    #[test]
    fn catalog_names_are_unique() {
        let mut names: Vec<_> = Resource::catalog().iter().map(|r| r.name()).collect();
        let total = names.len();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), total, "resource names must be unique");
    }
}
