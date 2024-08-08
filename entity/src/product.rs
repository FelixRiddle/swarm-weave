//! `SeaORM` Entity, @generated by sea-orm-codegen 1.0.0

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "product")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
    pub price: Option<i32>,
    pub image: Option<String>,
    pub stock: Option<i32>,
    #[sea_orm(column_name = "createdAt")]
    pub created_at: Option<DateTime>,
    #[sea_orm(column_name = "updatedAt")]
    pub updated_at: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::invoice_product_junction::Entity")]
    InvoiceProductJunction,
}

impl Related<super::invoice_product_junction::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::InvoiceProductJunction.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
