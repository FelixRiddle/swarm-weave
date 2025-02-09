//! `SeaORM` Entity, @generated by sea-orm-codegen 1.0.0

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "job")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub title: String,
    pub location: String,
    pub salary: i32,
    pub contract: String,
    #[sea_orm(column_type = "Text")]
    pub description: String,
    pub url: String,
    #[sea_orm(column_name = "createdAt")]
    pub created_at: Option<DateTime>,
    #[sea_orm(column_name = "updatedAt")]
    pub updated_at: Option<DateTime>,
    #[sea_orm(column_name = "companyId")]
    pub company_id: Option<i64>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::company::Entity",
        from = "Column::CompanyId",
        to = "super::company::Column::Id",
        on_update = "Cascade",
        on_delete = "SetNull"
    )]
    Company,
    #[sea_orm(has_many = "super::job_skill_junction::Entity")]
    JobSkillJunction,
}

impl Related<super::company::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Company.def()
    }
}

impl Related<super::job_skill_junction::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::JobSkillJunction.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
