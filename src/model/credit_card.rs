use serde::Serialize;
use uuid::Uuid;

#[derive(sqlx::FromRow, Serialize)]
pub struct CreditCard {
    pub id: Uuid,
    pub user_id: i32,
    pub name: String,
    pub icon_name: String,
    pub limit_value: i64,
    pub closing_day: i16,
}
