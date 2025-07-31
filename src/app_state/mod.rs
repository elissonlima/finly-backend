use std::sync::Mutex;

use jsonwebtoken::{DecodingKey, EncodingKey};
use sqlx::{Pool, Postgres};

use crate::model::{GoogleServiceToken, ServiceAccountKey};

pub struct AppState {
    pub pool: Pool<Postgres>,
    pub jwt_encoding_key: EncodingKey,
    pub jwt_decoding_key: DecodingKey,
    pub google_service_token: Mutex<GoogleServiceToken>,
    pub google_service_key: ServiceAccountKey
}

// impl AppState {
//     pub fn update_google_service_token(&mut self, n_google_service_token: GoogleServiceToken) {
//         self.google_service_token = n_google_service_token;
//     }
// }