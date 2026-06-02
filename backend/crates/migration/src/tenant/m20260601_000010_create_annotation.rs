//! Tenant-schema `annotation` table — a quick note about a collaborator, with up
//! to two scores and a free main note. Redesigned from the legacy model:
//! `manager` is derived from the collaborator at read time (not stored), and the
//! deferred concerns are dropped — `company_id` (multi-company), attachments
//! (`is_attach`/`attach_file_name`, needs S3) and feedback messaging
//! (`feedback_message`/`feedback_sent`, needs notifications). `active` carries the
//! soft-delete; audit columns are deferred (only `created_at`/`updated_at`).

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
                    .table(Annotation::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Annotation::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Annotation::CollaboratorId).uuid().not_null())
                    .col(
                        ColumnDef::new(Annotation::NoteDate)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Annotation::Score1Number)
                            .integer()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Annotation::Score1Description).text())
                    .col(ColumnDef::new(Annotation::Score1Type).text().not_null())
                    .col(
                        ColumnDef::new(Annotation::AskAmountDays)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(ColumnDef::new(Annotation::Score2Number).integer())
                    .col(ColumnDef::new(Annotation::Score2Description).text())
                    .col(ColumnDef::new(Annotation::Score2Type).text())
                    .col(ColumnDef::new(Annotation::AmountDays).integer())
                    .col(ColumnDef::new(Annotation::MainNote).text())
                    .col(ColumnDef::new(Annotation::PeriodStartDate).date())
                    .col(ColumnDef::new(Annotation::Observation).text())
                    .col(
                        ColumnDef::new(Annotation::RecordedOnMobile)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(Annotation::Active)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(timestamp(Annotation::CreatedAt))
                    .col(timestamp(Annotation::UpdatedAt))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_annotation_collaborator")
                            .from(Annotation::Table, Annotation::CollaboratorId)
                            .to(Collaborator::Table, Collaborator::Id),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Annotation::Table).to_owned())
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
pub enum Annotation {
    Table,
    Id,
    CollaboratorId,
    NoteDate,
    Score1Number,
    Score1Description,
    Score1Type,
    AskAmountDays,
    Score2Number,
    Score2Description,
    Score2Type,
    AmountDays,
    MainNote,
    PeriodStartDate,
    Observation,
    RecordedOnMobile,
    Active,
    CreatedAt,
    UpdatedAt,
}
