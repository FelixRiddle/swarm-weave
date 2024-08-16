//! `SeaORM` Entity, @generated by sea-orm-codegen 1.0.0

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "user")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub name: String,
    pub surname: Option<String>,
    #[sea_orm(unique)]
    pub email: String,
    pub password: String,
    #[sea_orm(column_name = "confirmedEmail")]
    pub confirmed_email: Option<i8>,
    pub token: Option<String>,
    pub expires: Option<DateTime>,
    pub pfp: Option<String>,
    #[sea_orm(column_name = "createdAt")]
    pub created_at: Option<DateTime>,
    #[sea_orm(column_name = "updatedAt")]
    pub updated_at: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::comment::Entity")]
    Comment,
    #[sea_orm(has_many = "super::company_staff::Entity")]
    CompanyStaff,
    #[sea_orm(has_many = "super::groups::Entity")]
    Groups,
    #[sea_orm(has_many = "super::invoice::Entity")]
    Invoice,
    #[sea_orm(has_many = "super::meeti::Entity")]
    Meeti,
    #[sea_orm(has_many = "super::meeti_participants::Entity")]
    MeetiParticipants,
    #[sea_orm(has_many = "super::property::Entity")]
    Property,
    #[sea_orm(has_many = "super::property_comment::Entity")]
    PropertyComment,
    #[sea_orm(has_many = "super::property_rating::Entity")]
    PropertyRating,
    #[sea_orm(has_many = "super::property_seller_message::Entity")]
    PropertySellerMessage,
    #[sea_orm(has_many = "super::user_contact_methods::Entity")]
    UserContactMethods,
    #[sea_orm(has_many = "super::user_favorite_property::Entity")]
    UserFavoriteProperty,
    #[sea_orm(has_many = "super::user_messages::Entity")]
    UserMessages,
}

impl Related<super::comment::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Comment.def()
    }
}

impl Related<super::company_staff::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::CompanyStaff.def()
    }
}

impl Related<super::groups::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Groups.def()
    }
}

impl Related<super::invoice::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Invoice.def()
    }
}

impl Related<super::meeti::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Meeti.def()
    }
}

impl Related<super::meeti_participants::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::MeetiParticipants.def()
    }
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

impl Related<super::user_contact_methods::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::UserContactMethods.def()
    }
}

impl Related<super::user_favorite_property::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::UserFavoriteProperty.def()
    }
}

impl Related<super::user_messages::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::UserMessages.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
