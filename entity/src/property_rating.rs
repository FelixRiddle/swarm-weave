//! `SeaORM` Entity, @generated by sea-orm-codegen 1.0.0

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "property-rating")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    #[sea_orm(column_type = "Float")]
    pub rating: f32,
    #[sea_orm(column_name = "createdAt")]
    pub created_at: Option<DateTime>,
    #[sea_orm(column_name = "updatedAt")]
    pub updated_at: Option<DateTime>,
    #[sea_orm(column_name = "propertyId")]
    pub property_id: Option<i64>,
    #[sea_orm(column_name = "userId")]
    pub user_id: Option<i64>,
    #[sea_orm(column_name = "generalPropertyInformationId")]
    pub general_property_information_id: Option<i64>,
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
    #[sea_orm(
        belongs_to = "super::general_property_information::Entity",
        from = "Column::GeneralPropertyInformationId",
        to = "super::general_property_information::Column::Id",
        on_update = "Cascade",
        on_delete = "SetNull"
    )]
    GeneralPropertyInformation,
    #[sea_orm(
        belongs_to = "super::property::Entity",
        from = "Column::PropertyId",
        to = "super::property::Column::Id",
        on_update = "Cascade",
        on_delete = "SetNull"
    )]
    Property,
}

impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Users.def()
    }
}

impl Related<super::general_property_information::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::GeneralPropertyInformation.def()
    }
}

impl Related<super::property::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Property.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
