//! Tenant-schema `role` table — a job title with the legacy set of descriptive
//! fields (profile suggestion, objective, the requirement breakdown and a free
//! observation). All description fields are optional text; only `name` is
//! required. `active` carries the soft-delete (a removed role is deactivated,
//! not dropped). Audit columns (`created_by`/`updated_by`) are deferred; only
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
                    .table(Role::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Role::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Role::Name).string().not_null())
                    .col(ColumnDef::new(Role::ProfileSuggestion).text())
                    .col(ColumnDef::new(Role::Objective).text())
                    .col(ColumnDef::new(Role::RequirementEducation).text())
                    .col(ColumnDef::new(Role::RequirementExperience).text())
                    .col(ColumnDef::new(Role::RequirementAttention).text())
                    .col(ColumnDef::new(Role::RequirementKnowledge).text())
                    .col(ColumnDef::new(Role::RequirementSkill).text())
                    .col(ColumnDef::new(Role::RequirementAttitude).text())
                    .col(ColumnDef::new(Role::RequirementDelivery).text())
                    .col(ColumnDef::new(Role::Observation).text())
                    .col(
                        ColumnDef::new(Role::Active)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(timestamp(Role::CreatedAt))
                    .col(timestamp(Role::UpdatedAt))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Role::Table).to_owned())
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
enum Role {
    Table,
    Id,
    Name,
    ProfileSuggestion,
    Objective,
    RequirementEducation,
    RequirementExperience,
    RequirementAttention,
    RequirementKnowledge,
    RequirementSkill,
    RequirementAttitude,
    RequirementDelivery,
    Observation,
    Active,
    CreatedAt,
    UpdatedAt,
}
