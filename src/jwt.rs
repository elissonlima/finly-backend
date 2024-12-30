use chrono::{DateTime, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenClaims {
    pub exp: usize,
    pub sub: String,
    pub iat: usize,
}

pub fn generate_token(
    subject: &str,
    private_key: &EncodingKey,
    expires_at: DateTime<Utc>,
) -> Result<String, jsonwebtoken::errors::Error> {
    let now = Utc::now();
    let iat = now.timestamp() as usize;
    let exp = expires_at.timestamp() as usize;
    let claims = TokenClaims {
        exp,
        sub: String::from(subject),
        iat,
    };

    let mut header = Header::new(Algorithm::RS256);
    header.typ = Some(String::from("JWT"));

    let token = encode(&header, &claims, private_key)?;

    Ok(token)
}

pub fn verify_token(
    token: &str,
    decoding_key: &DecodingKey,
) -> Result<TokenClaims, jsonwebtoken::errors::Error> {
    let token_message =
        decode::<TokenClaims>(token, decoding_key, &Validation::new(Algorithm::RS256))?;

    Ok(token_message.claims)
}
