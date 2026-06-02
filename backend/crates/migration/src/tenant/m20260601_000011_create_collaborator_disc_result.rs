//! Tenant-schema `collaborator_disc_result` table — a collaborator's DISC
//! assessment result: the four dimension scores (executor = D, communicator = I,
//! planner = S, analyst = C). Results are kept as history (no soft-delete `active`
//! flag); reads return the most recent. The primary/secondary profile is derived
//! at read time (not stored).

use sea_orm_migration::prelude::*;

use super::m20260601_000006_create_collaborator::Collaborator;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(CollaboratorDiscResult::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(CollaboratorDiscResult::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(CollaboratorDiscResult::CollaboratorId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(CollaboratorDiscResult::Executor)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(CollaboratorDiscResult::Communicator)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(CollaboratorDiscResult::Planner)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(CollaboratorDiscResult::Analyst)
                            .integer()
                            .not_null(),
                    )
                    .col(timestamp(CollaboratorDiscResult::CreatedAt))
                    .col(timestamp(CollaboratorDiscResult::UpdatedAt))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_collaborator_disc_result_collaborator")
                            .from(
                                CollaboratorDiscResult::Table,
                                CollaboratorDiscResult::CollaboratorId,
                            )
                            .to(Collaborator::Table, Collaborator::Id),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .table(CollaboratorDiscResult::Table)
                    .to_owned(),
            )
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
pub enum CollaboratorDiscResult {
    Table,
    Id,
    CollaboratorId,
    Executor,
    Communicator,
    Planner,
    Analyst,
    CreatedAt,
    UpdatedAt,
}
