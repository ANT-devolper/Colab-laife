//! A collaborator's DISC assessment result. Lives in the tenant schema and
//! references its `collaborator`. Stores the four dimension scores (executor = D,
//! communicator = I, planner = S, analyst = C); the primary/secondary profile is
//! derived at read time (see `service::disc`). Results are kept as history (no
//! soft-delete flag).

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "collaborator_disc_result")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub collaborator_id: Uuid,
    pub executor: i32,
    pub communicator: i32,
    pub planner: i32,
    pub analyst: i32,
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
