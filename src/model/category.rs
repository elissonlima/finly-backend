use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Serialize)]
pub struct Category {
    pub id: i32,
    pub user_id: i32,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>
}
