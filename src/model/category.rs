use super::subcategory::Subcategory;
use serde::Serialize;
use uuid::Uuid;

#[derive(sqlx::FromRow, Clone, Serialize)]
pub struct Category {
    pub id: Uuid,
    pub user_id: i32,
    pub name: String,
    pub color: String,
    pub icon_name: String,
    pub subcategories: Vec<Subcategory>,
}
