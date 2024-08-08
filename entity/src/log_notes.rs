//! `SeaORM` Entity, @generated by sea-orm-codegen 1.0.0

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "log-notes")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    #[sea_orm(column_name = "createdAt")]
    pub created_at: DateTime,
    #[sea_orm(column_name = "updatedAt")]
    pub updated_at: DateTime,
    #[sea_orm(column_name = "personalLogId")]
    pub personal_log_id: Option<i64>,
    #[sea_orm(column_name = "noteId")]
    pub note_id: Option<i64>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::note::Entity",
        from = "Column::NoteId",
        to = "super::note::Column::Id",
        on_update = "Cascade",
        on_delete = "SetNull"
    )]
    Note,
    #[sea_orm(
        belongs_to = "super::personal_log::Entity",
        from = "Column::PersonalLogId",
        to = "super::personal_log::Column::Id",
        on_update = "Cascade",
        on_delete = "SetNull"
    )]
    PersonalLog,
}

impl Related<super::note::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Note.def()
    }
}

impl Related<super::personal_log::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::PersonalLog.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
