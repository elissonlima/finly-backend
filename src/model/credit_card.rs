use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Serialize)]
pub struct CreditCard {
    pub id: i32,
    pub user_id: i32,
    pub name: String,
    pub color: String,
    pub closing_day: i16,
    pub is_default: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>
}
