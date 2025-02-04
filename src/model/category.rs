use super::subcategory::Subcategory;

#[derive(sqlx::FromRow, Clone)]
pub struct Category {
    pub id: String,
    pub user_id: i64,
    pub name: String,
    pub color: String,
    pub icon_name: String,
    pub subcategories: Vec<Subcategory>,
}
