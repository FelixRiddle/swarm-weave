//! `SeaORM` Entity, @generated by sea-orm-codegen 1.0.0

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "general-property-information")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    #[sea_orm(column_name = "createdAt")]
    pub created_at: Option<DateTime>,
    #[sea_orm(column_name = "updatedAt")]
    pub updated_at: Option<DateTime>,
    #[sea_orm(column_name = "propertyId")]
    pub property_id: Option<i64>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::property::Entity",
        from = "Column::PropertyId",
        to = "super::property::Column::Id",
        on_update = "Cascade",
        on_delete = "SetNull"
    )]
    Property,
    #[sea_orm(has_many = "super::property_comment::Entity")]
    PropertyComment,
    #[sea_orm(has_many = "super::property_rating::Entity")]
    PropertyRating,
    #[sea_orm(has_many = "super::property_seller_message::Entity")]
    PropertySellerMessage,
}

impl Related<super::property::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Property.def()
    }
}

impl Related<super::property_comment::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::PropertyComment.def()
    }
}

impl Related<super::property_rating::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::PropertyRating.def()
    }
}

impl Related<super::property_seller_message::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::PropertySellerMessage.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
