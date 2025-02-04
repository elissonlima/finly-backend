#[derive(sqlx::FromRow, Clone)]
pub struct Subcategory {
    pub id: String,
    pub category_id: String,
    pub name: String,
    pub color: String,
    pub icon_name: String,
}
