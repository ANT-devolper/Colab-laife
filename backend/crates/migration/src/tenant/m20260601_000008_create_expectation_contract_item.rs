//! Tenant-schema `expectation_contract_item` table — a single item of a feedback's
//! expectation contract. The legacy model split this into two identical tables
//! (`expectation_contract_goals` and `expectation_contract_behavior`, same
//! columns); we unify them into one table with a `kind` discriminator
//! (`goal` | `behavior`). Each item is a checklist entry (`description` + `done`)
//! belonging to a feedback. `active` carries the soft-delete; audit columns are
//! deferred (only `created_at`/`updated_at`).

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
                    .table(ExpectationContractItem::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ExpectationContractItem::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(ExpectationContractItem::FeedbackId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ExpectationContractItem::Kind)
                            .text()
                            .not_null(),
                    )
                    .col(ColumnDef::new(ExpectationContractItem::Description).text())
                    .col(
                        ColumnDef::new(ExpectationContractItem::Done)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(ExpectationContractItem::Active)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(timestamp(ExpectationContractItem::CreatedAt))
                    .col(timestamp(ExpectationContractItem::UpdatedAt))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_expectation_contract_item_feedback")
                            .from(
                                ExpectationContractItem::Table,
                                ExpectationContractItem::FeedbackId,
                            )
                            .to(Feedback::Table, Feedback::Id),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .table(ExpectationContractItem::Table)
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
pub enum ExpectationContractItem {
    Table,
    Id,
    FeedbackId,
    Kind,
    Description,
    Done,
    Active,
    CreatedAt,
    UpdatedAt,
}
