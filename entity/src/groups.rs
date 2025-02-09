//! `SeaORM` Entity, @generated by sea-orm-codegen 1.0.0

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "groups")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub name: String,
    #[sea_orm(column_type = "Text", nullable)]
    pub description: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub url: Option<String>,
    pub image: Option<String>,
    #[sea_orm(column_name = "createdAt")]
    pub created_at: Option<DateTime>,
    #[sea_orm(column_name = "updatedAt")]
    pub updated_at: Option<DateTime>,
    #[sea_orm(column_name = "socialCategoryId")]
    pub social_category_id: Option<i32>,
    #[sea_orm(column_name = "userId")]
    pub user_id: Option<i64>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::users::Entity",
        from = "Column::UserId",
        to = "super::users::Column::Id",
        on_update = "Cascade",
        on_delete = "SetNull"
    )]
    Users,
    #[sea_orm(has_many = "super::meeti::Entity")]
    Meeti,
    #[sea_orm(
        belongs_to = "super::social_category::Entity",
        from = "Column::SocialCategoryId",
        to = "super::social_category::Column::Id",
        on_update = "Cascade",
        on_delete = "SetNull"
    )]
    SocialCategory,
}

impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Users.def()
    }
}

impl Related<super::meeti::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Meeti.def()
    }
}

impl Related<super::social_category::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::SocialCategory.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
