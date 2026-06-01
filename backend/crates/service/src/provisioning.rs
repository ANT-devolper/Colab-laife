//! Tenant provisioning. Creates an organization in the `public` schema, its
//! dedicated PostgreSQL schema (migrated with `TenantMigrator`), and the admin
//! user. The organization `name` doubles as the schema slug, so it is validated
//! as a safe SQL identifier before being interpolated into DDL.

use crate::password::hash_password;
use crate::tenant::is_valid_schema_name;
use entity::{organization, user};
use migration::{MigratorTrait, TenantMigrator};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectOptions, ConnectionTrait, Database, DatabaseConnection,
    DbErr, EntityTrait, QueryFilter,
};
use sea_orm::{ActiveValue::NotSet, Set, TransactionTrait};
use uuid::Uuid;

/// Input for provisioning a new tenant and its first (admin) user.
pub struct NewOrganization {
    pub name: String,
    pub plan: Option<String>,
    pub admin_name: String,
    pub admin_email: String,
    pub admin_password: String,
}

/// What provisioning created, for the caller to return/inspect.
pub struct Provisioned {
    pub organization: organization::Model,
    pub admin: user::Model,
}

#[derive(Debug)]
pub enum ProvisionError {
    /// `name` is not a safe schema identifier (`^[a-z][a-z0-9_]{0,62}$`).
    InvalidName,
    /// An organization with this name already exists.
    NameTaken,
    /// Password hashing failed.
    Hash,
    /// A database/migration error.
    Db(DbErr),
}

impl std::fmt::Display for ProvisionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidName => write!(f, "invalid organization name"),
            Self::NameTaken => write!(f, "organization name already taken"),
            Self::Hash => write!(f, "failed to hash password"),
            Self::Db(e) => write!(f, "database error: {e}"),
        }
    }
}

impl std::error::Error for ProvisionError {}

impl From<DbErr> for ProvisionError {
    fn from(error: DbErr) -> Self {
        Self::Db(error)
    }
}

/// Provisions a tenant. `database_url` is needed to open a connection whose
/// `search_path` points at the new schema, so `TenantMigrator` runs inside it.
///
/// Atomicity across the create-schema / migrate / insert steps is best-effort:
/// a failed migration drops the freshly created schema; the organization and
/// admin rows are written in a single `public` transaction.
pub async fn provision_organization(
    db: &DatabaseConnection,
    database_url: &str,
    input: NewOrganization,
) -> Result<Provisioned, ProvisionError> {
    if !is_valid_schema_name(&input.name) {
        return Err(ProvisionError::InvalidName);
    }

    let already_exists = organization::Entity::find()
        .filter(organization::Column::Name.eq(&input.name))
        .one(db)
        .await?
        .is_some();
    if already_exists {
        return Err(ProvisionError::NameTaken);
    }

    // Create and migrate the tenant schema. `name` is validated above.
    db.execute_unprepared(&format!("CREATE SCHEMA IF NOT EXISTS \"{}\"", input.name))
        .await?;

    let mut options = ConnectOptions::new(database_url.to_owned());
    options.set_schema_search_path(input.name.clone());
    let tenant_conn = Database::connect(options).await?;
    if let Err(error) = TenantMigrator::up(&tenant_conn, None).await {
        let _ = db
            .execute_unprepared(&format!("DROP SCHEMA IF EXISTS \"{}\" CASCADE", input.name))
            .await;
        return Err(ProvisionError::Db(error));
    }

    let password_hash = hash_password(&input.admin_password).map_err(|_| ProvisionError::Hash)?;

    // Persist the organization and its admin atomically in the public schema.
    let txn = db.begin().await?;
    let organization = organization::ActiveModel {
        id: Set(Uuid::new_v4()),
        name: Set(input.name.clone()),
        plan: input.plan.map(Set).unwrap_or(NotSet),
        ..Default::default()
    }
    .insert(&txn)
    .await?;
    let admin = user::ActiveModel {
        id: Set(Uuid::new_v4()),
        name: Set(input.admin_name),
        email: Set(input.admin_email),
        password_hash: Set(password_hash),
        is_admin: Set(true),
        organization_id: Set(organization.id),
        ..Default::default()
    }
    .insert(&txn)
    .await?;
    txn.commit().await?;

    Ok(Provisioned {
        organization,
        admin,
    })
}
