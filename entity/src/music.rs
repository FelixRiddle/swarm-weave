//! `SeaORM` Entity, @generated by sea-orm-codegen 1.0.0

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "music")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub composer: Option<String>,
    pub featuring: Option<Json>,
    pub name: Option<String>,
    pub url: Option<String>,
    #[sea_orm(column_name = "createdAt")]
    pub created_at: DateTime,
    #[sea_orm(column_name = "updatedAt")]
    pub updated_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::listening_to::Entity")]
    ListeningTo,
}

impl Related<super::listening_to::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ListeningTo.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
