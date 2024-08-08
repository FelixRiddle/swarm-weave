//! `SeaORM` Entity, @generated by sea-orm-codegen 1.0.0

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "invoice-product-junction")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    #[sea_orm(column_name = "createdAt")]
    pub created_at: Option<DateTime>,
    #[sea_orm(column_name = "updatedAt")]
    pub updated_at: Option<DateTime>,
    #[sea_orm(column_name = "invoiceId")]
    pub invoice_id: Option<i64>,
    #[sea_orm(column_name = "productId")]
    pub product_id: Option<i32>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::invoice::Entity",
        from = "Column::InvoiceId",
        to = "super::invoice::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    Invoice,
    #[sea_orm(
        belongs_to = "super::product::Entity",
        from = "Column::ProductId",
        to = "super::product::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    Product,
}

impl Related<super::invoice::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Invoice.def()
    }
}

impl Related<super::product::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Product.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
