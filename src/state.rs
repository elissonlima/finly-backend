use jsonwebtoken::{DecodingKey, EncodingKey};
use sqlx::{Pool, Postgres};

pub struct AppState {
    pub db: Pool<Postgres>,
    pub jwt_encoding_key: EncodingKey,
    pub jwt_decoding_key: DecodingKey,
    pub google_oauth_client_id: String,
}
