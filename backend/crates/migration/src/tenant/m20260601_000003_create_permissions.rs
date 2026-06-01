//! Tenant-schema RBAC tables. A user is granted one or more profiles; each
//! profile groups tasks; each task groups resources (the protected actions).
//! Permission is the union of the resources reachable from a user's profiles.
//! `permission_profile_users.user_id` references `public.users` by value only
//! (no cross-schema foreign key, mirroring the legacy decoupling).

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Resources::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Resources::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Resources::Name)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(Resources::Label).string().not_null())
                    .col(timestamp(Resources::CreatedAt))
                    .col(timestamp(Resources::UpdatedAt))
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Tasks::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Tasks::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Tasks::Name).string().not_null().unique_key())
                    .col(ColumnDef::new(Tasks::Label).string().not_null())
                    .col(timestamp(Tasks::CreatedAt))
                    .col(timestamp(Tasks::UpdatedAt))
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(TaskResources::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(TaskResources::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(TaskResources::TaskId).uuid().not_null())
                    .col(ColumnDef::new(TaskResources::ResourceId).uuid().not_null())
                    .col(timestamp(TaskResources::CreatedAt))
                    .col(timestamp(TaskResources::UpdatedAt))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_task_resources_task")
                            .from(TaskResources::Table, TaskResources::TaskId)
                            .to(Tasks::Table, Tasks::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_task_resources_resource")
                            .from(TaskResources::Table, TaskResources::ResourceId)
                            .to(Resources::Table, Resources::Id),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Profiles::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Profiles::Id).uuid().not_null().primary_key())
                    .col(
                        ColumnDef::new(Profiles::Name)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(Profiles::Label).string().not_null())
                    .col(timestamp(Profiles::CreatedAt))
                    .col(timestamp(Profiles::UpdatedAt))
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(ProfileTasks::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ProfileTasks::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(ProfileTasks::ProfileId).uuid().not_null())
                    .col(ColumnDef::new(ProfileTasks::TaskId).uuid().not_null())
                    .col(timestamp(ProfileTasks::CreatedAt))
                    .col(timestamp(ProfileTasks::UpdatedAt))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_profile_tasks_profile")
                            .from(ProfileTasks::Table, ProfileTasks::ProfileId)
                            .to(Profiles::Table, Profiles::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_profile_tasks_task")
                            .from(ProfileTasks::Table, ProfileTasks::TaskId)
                            .to(Tasks::Table, Tasks::Id),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(ProfileUsers::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ProfileUsers::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    // References public.users by value; no cross-schema FK.
                    .col(ColumnDef::new(ProfileUsers::UserId).uuid().not_null())
                    .col(ColumnDef::new(ProfileUsers::ProfileId).uuid().not_null())
                    .col(timestamp(ProfileUsers::CreatedAt))
                    .col(timestamp(ProfileUsers::UpdatedAt))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_profile_users_profile")
                            .from(ProfileUsers::Table, ProfileUsers::ProfileId)
                            .to(Profiles::Table, Profiles::Id),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        for table in [
            ProfileUsers::Table.into_table_ref(),
            ProfileTasks::Table.into_table_ref(),
            Profiles::Table.into_table_ref(),
            TaskResources::Table.into_table_ref(),
            Tasks::Table.into_table_ref(),
            Resources::Table.into_table_ref(),
        ] {
            manager
                .drop_table(Table::drop().table(table).to_owned())
                .await?;
        }
        Ok(())
    }
}

/// A non-null timestamp column defaulting to now — the shape every RBAC table
/// shares for its audit columns.
fn timestamp<T: IntoIden>(name: T) -> ColumnDef {
    ColumnDef::new(name)
        .timestamp_with_time_zone()
        .not_null()
        .default(Expr::current_timestamp())
        .to_owned()
}

#[derive(DeriveIden)]
enum Resources {
    #[sea_orm(iden = "permission_resources")]
    Table,
    Id,
    Name,
    Label,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Tasks {
    #[sea_orm(iden = "permission_tasks")]
    Table,
    Id,
    Name,
    Label,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum TaskResources {
    #[sea_orm(iden = "permission_task_resources")]
    Table,
    Id,
    TaskId,
    ResourceId,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Profiles {
    #[sea_orm(iden = "permission_profiles")]
    Table,
    Id,
    Name,
    Label,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum ProfileTasks {
    #[sea_orm(iden = "permission_profile_tasks")]
    Table,
    Id,
    ProfileId,
    TaskId,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum ProfileUsers {
    #[sea_orm(iden = "permission_profile_users")]
    Table,
    Id,
    UserId,
    ProfileId,
    CreatedAt,
    UpdatedAt,
}
