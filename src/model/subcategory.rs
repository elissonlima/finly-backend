use serde::Serialize;

#[derive(sqlx::FromRow, Clone, Serialize)]
pub struct Subcategory {
    pub id: String,
    pub category_id: String,
    pub name: String,
    pub color: String,
    pub icon_name: String,
}
