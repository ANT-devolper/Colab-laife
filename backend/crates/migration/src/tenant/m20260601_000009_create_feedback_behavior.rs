//! Tenant-schema `feedback_behavior` table — a scored behaviour line of a
//! feedback (the DISC-values assessment: a value + the observed behaviour and its
//! score). Belongs to a feedback. `active` carries the soft-delete; audit columns
//! are deferred (only `created_at`/`updated_at`).

use sea_orm_migration::prelude::*;

use super::m20260601_000007_create_feedback::Feedback;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(FeedbackBehavior::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(FeedbackBehavior::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(FeedbackBehavior::FeedbackId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(FeedbackBehavior::ValueDescription)
                            .text()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(FeedbackBehavior::BehaviorDescription)
                            .text()
                            .not_null(),
                    )
                    .col(ColumnDef::new(FeedbackBehavior::BehaviorObs).text())
                    .col(ColumnDef::new(FeedbackBehavior::ValueInstruction).text())
                    .col(ColumnDef::new(FeedbackBehavior::Score).integer().not_null())
                    .col(
                        ColumnDef::new(FeedbackBehavior::Active)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(timestamp(FeedbackBehavior::CreatedAt))
                    .col(timestamp(FeedbackBehavior::UpdatedAt))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_feedback_behavior_feedback")
                            .from(FeedbackBehavior::Table, FeedbackBehavior::FeedbackId)
                            .to(Feedback::Table, Feedback::Id),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(FeedbackBehavior::Table).to_owned())
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
pub enum FeedbackBehavior {
    Table,
    Id,
    FeedbackId,
    ValueDescription,
    BehaviorDescription,
    BehaviorObs,
    ValueInstruction,
    Score,
    Active,
    CreatedAt,
    UpdatedAt,
}
