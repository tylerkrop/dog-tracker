use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "feeding")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub amount_half_scoops: i32,
    pub fed_at: String,
    pub edited: bool,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
