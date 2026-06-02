//! A quick note about a collaborator, with up to two scores and a free main note.
//! Lives in the tenant schema and references its `collaborator`. Manager is
//! derived from the collaborator at read time (not stored). `active` is the
//! soft-delete flag.

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "annotation")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub collaborator_id: Uuid,
    pub note_date: DateTimeWithTimeZone,
    pub score1_number: i32,
    pub score1_description: Option<String>,
    pub score1_type: String,
    pub ask_amount_days: bool,
    pub score2_number: Option<i32>,
    pub score2_description: Option<String>,
    pub score2_type: Option<String>,
    pub amount_days: Option<i32>,
    pub main_note: Option<String>,
    pub period_start_date: Option<Date>,
    pub observation: Option<String>,
    pub recorded_on_mobile: bool,
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
