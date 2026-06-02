//! A single item of a feedback's expectation contract. Lives in the tenant schema
//! and references its `feedback`. `kind` discriminates a `goal` from a `behavior`
//! (the legacy model used two identical tables). Each item is a checklist entry
//! (`description` + `done`). `active` is the soft-delete flag.

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "expectation_contract_item")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub feedback_id: Uuid,
    pub kind: String,
    pub description: Option<String>,
    pub done: bool,
    pub active: bool,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::feedback::Entity",
        from = "Column::FeedbackId",
        to = "super::feedback::Column::Id"
    )]
    Feedback,
}

impl Related<super::feedback::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Feedback.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
