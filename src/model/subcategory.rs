use serde::Serialize;
use uuid::Uuid;

#[derive(sqlx::FromRow, Clone, Serialize)]
pub struct Subcategory {
    pub id: Uuid,
    pub category_id: Uuid,
    pub name: String,
    pub color: String,
    pub icon_name: String,
}
