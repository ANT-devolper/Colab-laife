//! Tenant-schema `sector` table — an organizational unit a collaborator belongs
//! to. `active` carries the soft-delete (a removed sector is deactivated, not
//! dropped). Audit columns (`created_by`/`updated_by`) are deferred; only
//! `created_at`/`updated_at` are tracked for now.

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Sector::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Sector::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Sector::Name).string().not_null())
                    .col(
                        ColumnDef::new(Sector::Active)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(timestamp(Sector::CreatedAt))
                    .col(timestamp(Sector::UpdatedAt))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Sector::Table).to_owned())
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
enum Sector {
    Table,
    Id,
    Name,
    Active,
    CreatedAt,
    UpdatedAt,
}
