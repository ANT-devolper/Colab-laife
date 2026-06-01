//! Tenant-schema `collaborator` table — a person managed inside a tenant (the
//! corporate record: sector, role, manager). Distinct from `public.users` (the
//! login identity): the optional `user_id` links the two **by value**, with no
//! cross-schema foreign key (the same approach as `permission_profile_users`).
//!
//! `sector_id`/`role_id` are nullable FKs into the tenant's `sector`/`role`;
//! `manager_id` is a nullable self-FK (the org hierarchy — only the column lands
//! now, the "accessible collaborators" service is deferred). `active` carries the
//! soft-delete. Audit columns are deferred; only `created_at`/`updated_at` exist.

use sea_orm_migration::prelude::*;

use super::m20260601_000004_create_sector::Sector;
use super::m20260601_000005_create_role::Role;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Collaborator::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Collaborator::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Collaborator::Name).string().not_null())
                    .col(ColumnDef::new(Collaborator::SectorId).uuid())
                    .col(ColumnDef::new(Collaborator::RoleId).uuid())
                    .col(ColumnDef::new(Collaborator::ManagerId).uuid())
                    .col(ColumnDef::new(Collaborator::Whatsapp).text())
                    .col(ColumnDef::new(Collaborator::Email).text())
                    .col(
                        ColumnDef::new(Collaborator::IsManager)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(ColumnDef::new(Collaborator::UserId).uuid())
                    .col(ColumnDef::new(Collaborator::DateOfHire).date())
                    .col(
                        ColumnDef::new(Collaborator::Active)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(timestamp(Collaborator::CreatedAt))
                    .col(timestamp(Collaborator::UpdatedAt))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_collaborator_sector")
                            .from(Collaborator::Table, Collaborator::SectorId)
                            .to(Sector::Table, Sector::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_collaborator_role")
                            .from(Collaborator::Table, Collaborator::RoleId)
                            .to(Role::Table, Role::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_collaborator_manager")
                            .from(Collaborator::Table, Collaborator::ManagerId)
                            .to(Collaborator::Table, Collaborator::Id),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Collaborator::Table).to_owned())
            .await
    }
}

/// A non-null timestamp column defaulting to now.
fn timestamp<T: IntoIden>(name: T) -> ColumnDef {
    ColumnDef::new(name)
        .timestamp_with_time_zone()
        .not_null()
        .default(Expr::current_timestamp())
        .to_owned()
}

#[derive(DeriveIden)]
enum Collaborator {
    Table,
    Id,
    Name,
    SectorId,
    RoleId,
    ManagerId,
    Whatsapp,
    Email,
    IsManager,
    UserId,
    DateOfHire,
    Active,
    CreatedAt,
    UpdatedAt,
}
