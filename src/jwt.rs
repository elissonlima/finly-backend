use chrono::{DateTime, Utc};
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct TokenClaims {
    exp: usize,
    sub: String,
    iat: usize,
}

pub fn generate_token(
    user_email: &str,
    private_key: &EncodingKey,
    expires_at: DateTime<Utc>,
) -> Result<String, jsonwebtoken::errors::Error> {
    let now = Utc::now();
    let iat = now.timestamp() as usize;
    let exp = expires_at.timestamp() as usize;
    let claims = TokenClaims {
        exp,
        sub: String::from(user_email),
        iat,
    };

    let mut header = Header::new(Algorithm::RS256);
    header.typ = Some(String::from("JWT"));

    let token = encode(&header, &claims, private_key)?;

    Ok(token)
}
