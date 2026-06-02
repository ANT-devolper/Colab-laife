//! A structured feedback event about a collaborator. Lives in the tenant schema
//! and references its `collaborator`. Manager/sector are derived from the
//! collaborator at read time (not stored here). `active` is the soft-delete flag.

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "feedback")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub collaborator_id: Uuid,
    pub feedback_date: DateTimeWithTimeZone,
    pub next_feedback_date: Option<DateTimeWithTimeZone>,
    pub expectation_contract_observation: Option<String>,
    pub expectation_contract_observation_private: Option<String>,
    pub status: Option<String>,
    pub active: bool,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::collaborator::Entity",
        from = "Column::CollaboratorId",
        to = "super::collaborator::Column::Id"
    )]
    Collaborator,
}

impl Related<super::collaborator::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Collaborator.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
