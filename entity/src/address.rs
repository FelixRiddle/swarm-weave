//! `SeaORM` Entity, @generated by sea-orm-codegen 1.0.0

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "address")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub street: String,
    pub city: String,
    pub state: String,
    pub country: String,
    #[sea_orm(column_type = "Double", nullable)]
    pub latitude: Option<f64>,
    #[sea_orm(column_type = "Double", nullable)]
    pub longitude: Option<f64>,
    pub postal: Option<String>,
    #[sea_orm(column_name = "createdAt")]
    pub created_at: Option<DateTime>,
    #[sea_orm(column_name = "updatedAt")]
    pub updated_at: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::company::Entity")]
    Company,
    #[sea_orm(has_many = "super::meeti::Entity")]
    Meeti,
    #[sea_orm(has_many = "super::personal_log::Entity")]
    PersonalLog,
}

impl Related<super::company::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Company.def()
    }
}

impl Related<super::meeti::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Meeti.def()
    }
}

impl Related<super::personal_log::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::PersonalLog.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
