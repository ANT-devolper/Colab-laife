//! A person managed inside a tenant (the corporate record). Lives in the tenant
//! schema. Distinct from `user` (the login identity in `public.users`): the
//! optional `user_id` links the two by value, with no cross-schema FK. References
//! its `sector` and `role`, and an optional `manager` (self-reference). `active`
//! is the soft-delete flag.

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "collaborator")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub name: String,
    pub sector_id: Option<Uuid>,
    pub role_id: Option<Uuid>,
    pub manager_id: Option<Uuid>,
    pub whatsapp: Option<String>,
    pub email: Option<String>,
    pub is_manager: bool,
    pub user_id: Option<Uuid>,
    pub date_of_hire: Option<Date>,
    pub active: bool,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::sector::Entity",
        from = "Column::SectorId",
        to = "super::sector::Column::Id"
    )]
    Sector,
    #[sea_orm(
        belongs_to = "super::role::Entity",
        from = "Column::RoleId",
        to = "super::role::Column::Id"
    )]
    Role,
    #[sea_orm(belongs_to = "Entity", from = "Column::ManagerId", to = "Column::Id")]
    Manager,
}

impl Related<super::sector::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Sector.def()
    }
}

impl Related<super::role::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Role.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
