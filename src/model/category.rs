use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Serialize)]
pub struct Category {
    pub id: i32,
    pub user_id: i32,
    pub icon_id: i32,
    pub name: String,
    pub color: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>
}


#[derive(Serialize)]
pub struct CategoryIcon {
    pub id: i32,
    pub user_id: i32,
    pub xml_icon: String,
    pub name: String,
    pub color: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>
}