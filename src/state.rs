use jsonwebtoken::{DecodingKey, EncodingKey};
use sqlx::{Pool, Sqlite};

pub struct AppState {
    pub db: Pool<Sqlite>,
    pub session_db: Pool<Sqlite>,
    pub jwt_encoding_key: EncodingKey,
    pub jwt_decoding_key: DecodingKey,
}
