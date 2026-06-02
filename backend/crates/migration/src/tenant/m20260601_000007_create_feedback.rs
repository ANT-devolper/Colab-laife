//! Tenant-schema `feedback` table — a structured feedback event about a
//! collaborator (the manager↔report conversation, with the expectation-contract
//! observations). Redesigned from the legacy `feedback` model: manager/sector are
//! **not** stored here (they are derived from the collaborator at read time);
//! AI/transcription (`openai_counter`, records) is out of scope. `active` carries
//! the soft-delete. Audit columns (`created_by`/`updated_by`) are deferred; only
//! `created_at`/`updated_at` are tracked for now.

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
                    .table(Feedback::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Feedback::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Feedback::CollaboratorId).uuid().not_null())
                    .col(
                        ColumnDef::new(Feedback::FeedbackDate)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Feedback::NextFeedbackDate).timestamp_with_time_zone())
                    .col(ColumnDef::new(Feedback::ExpectationContractObservation).text())
                    .col(ColumnDef::new(Feedback::ExpectationContractObservationPrivate).text())
                    .col(ColumnDef::new(Feedback::Status).text())
                    .col(
                        ColumnDef::new(Feedback::Active)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(timestamp(Feedback::CreatedAt))
                    .col(timestamp(Feedback::UpdatedAt))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_feedback_collaborator")
                            .from(Feedback::Table, Feedback::CollaboratorId)
                            .to(Collaborator::Table, Collaborator::Id),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Feedback::Table).to_owned())
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
pub enum Feedback {
    Table,
    Id,
    CollaboratorId,
    FeedbackDate,
    NextFeedbackDate,
    ExpectationContractObservation,
    ExpectationContractObservationPrivate,
    Status,
    Active,
    CreatedAt,
    UpdatedAt,
}
