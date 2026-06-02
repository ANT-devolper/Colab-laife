//! A scored behaviour line of a feedback (the DISC-values assessment). Lives in
//! the tenant schema and references its `feedback`. `active` is the soft-delete
//! flag.

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "feedback_behavior")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub feedback_id: Uuid,
    pub value_description: String,
    pub behavior_description: String,
    pub behavior_obs: Option<String>,
    pub value_instruction: Option<String>,
    pub score: i32,
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
