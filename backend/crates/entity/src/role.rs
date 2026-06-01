//! A job title inside a tenant, with the legacy set of descriptive fields. Lives
//! in the tenant schema; a collaborator references its `role`. All description
//! fields are optional; `active` is the soft-delete flag.

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "role")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub name: String,
    pub profile_suggestion: Option<String>,
    pub objective: Option<String>,
    pub requirement_education: Option<String>,
    pub requirement_experience: Option<String>,
    pub requirement_attention: Option<String>,
    pub requirement_knowledge: Option<String>,
    pub requirement_skill: Option<String>,
    pub requirement_attitude: Option<String>,
    pub requirement_delivery: Option<String>,
    pub observation: Option<String>,
    pub active: bool,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
