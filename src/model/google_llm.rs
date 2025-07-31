use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// --- Data Structures for Service Account Key ---
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct ServiceAccountKey {
    #[serde(rename = "type")]
    pub key_type: String,
    pub project_id: String,
    pub private_key_id: String,
    pub private_key: String,
    pub client_email: String,
    pub client_id: String,
    pub auth_uri: String,
    pub token_uri: String,
    pub auth_provider_x509_cert_url: String,
    pub client_x509_cert_url: String,
    pub universe_domain: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct GoogleServiceToken {
    pub access_token: String,
    pub token_type: String,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct GoogleServiceTokenApiResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u32,
}


#[derive(Debug, Serialize)]
pub struct Claims {
    pub iss: String,   // Issuer (client_email)
    pub scope: String, // Scopes requested
    pub aud: String,   // Audience (token_uri)
    pub exp: i64,      // Expiration time
    pub iat: i64,      // Issued at time
}

#[derive(Debug, Deserialize)]
pub struct LLMResponseCandidatePart {
    pub text: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct LLMResponseCandidateContent {
    pub role: String,
    pub parts: Vec<LLMResponseCandidatePart>,
    #[serde(rename = "finishReason")]
    pub finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LLMResponseCandidate {
    pub content: LLMResponseCandidateContent,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct LLMResponseUsageMetadataTokensDetails {
    pub modality: String,
    #[serde(rename = "tokenCount")]
    pub token_count: i32,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct LLMResponseUsageMetadata {
    #[serde(rename = "promptTokenCount")]
    pub prompt_token_count: Option<i32>,
    #[serde(rename = "candidatesTokenCount")]
    pub candidates_token_count: Option<i32>,
    #[serde(rename = "totalTokenCount")]
    pub total_token_count: Option<i32>,
    #[serde(rename = "trafficType")]
    pub traffic_type: String,
    #[serde(rename = "promptTokenDetails")]
    pub prompt_token_details: Option<Vec<LLMResponseUsageMetadataTokensDetails>>,
    #[serde(rename = "candidatesTokensDetails")]
    pub candidates_tokens_details: Option<Vec<LLMResponseUsageMetadataTokensDetails>>,
    #[serde(rename = "thoughtsTokenCount")]
    pub thoughts_token_count: Option<i32>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct LLMResponse {
    pub candidates: Vec<LLMResponseCandidate>,
    #[serde(rename = "usageMetadata")]
    pub usage_metadata: LLMResponseUsageMetadata,
    #[serde(rename = "modelVersion")]
    pub model_version: String,
    #[serde(rename = "createTime")]
    pub create_time: String,
    #[serde(rename = "responseId")]
    pub response_id: String,
}