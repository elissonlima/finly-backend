#[derive(sqlx::FromRow)]
pub struct ResetPassword {
    pub id: String,
    pub user_email: String,
    pub sent_at: String,
    pub expires_at: String,
    pub is_password_reset: i8,
}
