use std::{collections::BTreeMap, fs};

use chrono::{Duration, Utc};
use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};
use serde::{Deserialize, Serialize};
use serde_json::json;

const PROJECT_ID: &str = "finlydigital-dev";
const LOCATION_ID: &str = "global";
const API_ENDPOINT: &str = "aiplatform.googleapis.com";
const MODEL_ID: &str = "gemini-2.5-flash";
const GENERATE_CONTENT_API: &str = "streamGenerateContent";
const SERVICE_ACCOUNT_KEY: &str = "/home/elisson/finlydigital-dev-661f325a3dc1.json";

// --- Data Structures for Service Account Key ---
#[derive(Debug, Deserialize)]
struct ServiceAccountKey {
    #[serde(rename = "type")]
    key_type: String,
    project_id: String,
    private_key_id: String,
    private_key: String,
    client_email: String,
    client_id: String,
    auth_uri: String,
    token_uri: String,
    auth_provider_x509_cert_url: String,
    client_x509_cert_url: String,
    universe_domain: Option<String>,
}

#[derive(Debug, Serialize)]
struct Claims {
    iss: String,   // Issuer (client_email)
    scope: String, // Scopes requested
    aud: String,   // Audience (token_uri)
    exp: i64,      // Expiration time
    iat: i64,      // Issued at time
}

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    token_type: String,
    expires_in: u32,
}

#[derive(Debug, Deserialize)]
pub struct LLMResponseCandidatePart {
    pub text: String,
}

#[derive(Debug, Deserialize)]
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
pub struct LLMResponseUsageMetadataTokensDetails {
    pub modality: String,
    #[serde(rename = "tokenCount")]
    pub token_count: i32,
}

#[derive(Debug, Deserialize)]
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

async fn get_access_token() -> Result<String, Box<dyn std::error::Error>> {
    let key_json = fs::read_to_string(SERVICE_ACCOUNT_KEY)?;
    let key: ServiceAccountKey = serde_json::from_str(&key_json)?;

    let now = Utc::now();
    let claims = Claims {
        iss: key.client_email.clone(),
        scope: "https://www.googleapis.com/auth/cloud-platform".to_string(),
        aud: key.token_uri.clone(),
        exp: (now + Duration::minutes(60)).timestamp(),
        iat: now.timestamp(),
    };

    let header = Header::new(Algorithm::RS256);
    let encoding_key = EncodingKey::from_rsa_pem(key.private_key.as_bytes())?;
    let jwt = encode(&header, &claims, &encoding_key)?;

    let client = reqwest::Client::new();
    let mut params = BTreeMap::new();
    params.insert("grant_type", "urn:ietf:params:oauth:grant-type:jwt-bearer");
    params.insert("assertion", &jwt);

    let token_res: TokenResponse = client
        .post(&key.token_uri)
        .form(&params)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    Ok(token_res.access_token)
}

pub async fn exec_prompt(user_text: &str) -> Result<Option<Vec<LLMResponse>>, reqwest::Error> {
    let prompt_text = build_prompt(user_text);
    let system_instructions = get_system_instructions();

    let access_token = match get_access_token().await {
        Ok(a) => a,
        Err(e) => {
            log::error!(
                "An error occurred while trying to get access for google api request: {}",
                e
            );
            return Ok(None);
        }
    };

    let client = reqwest::Client::new();
    let url = format!(
        "https://{api_endpoint}/v1/projects/{project_id}/locations/{location_id}/publishers/google/models/{model_id}:{generate_content_api}",
        api_endpoint = API_ENDPOINT,
        project_id = PROJECT_ID,
        location_id = LOCATION_ID,
        model_id = MODEL_ID,
        generate_content_api = GENERATE_CONTENT_API
    );
    let response = client
        .post(url)
        .header("Authorization", format!("Bearer {}", access_token))
        .header(
            reqwest::header::CONTENT_TYPE,
            "application/json; charset=utf-8",
        )
        .body(
            json!({
                "contents": [
                    {
                        "role": "user",
                        "parts": [{"text":prompt_text}]
                    }
                ],
                "systemInstruction": {
                    "parts": [{"text":system_instructions}]
                },
                "generationConfig": {
                    "candidateCount": 1,
                    "temperature": 1,
                    "maxOutputTokens": 10_000,
                    "topP": 0.95,
                    "thinkingConfig": {
                        "thinkingBudget": -1
                    },
                    "responseMimeType": "application/json",
                    "responseSchema": {"type":"OBJECT","properties":{"command":{"type":"STRING"},"object":{"type":"STRING"},"description":{"type":"STRING"},"value":{"type":"INTEGER"},"category":{"type":"STRING"},"credit_card":{"type":"STRING"},"period":{"type":"STRING"},"frequency":{"type":"STRING"}},"required":["command","object","description"]}
                },
                "safetySettings": [
                    {
                        "category": "HARM_CATEGORY_HATE_SPEECH",
                        "threshold": "OFF"
                    },
                    {
                        "category":"HARM_CATEGORY_DANGEROUS_CONTENT",
                        "threshold": "OFF"
                    },
                    {
                        "category":"HARM_CATEGORY_SEXUALLY_EXPLICIT",
                        "threshold": "OFF"
                    },
                    {
                        "category":"HARM_CATEGORY_HARASSMENT",
                        "threshold": "OFF"
                    },
                ]
            })
            .to_string(),
        )
        .send()
        .await?;

    if !response.status().is_success() {
        println!("OH NO");
        println!("Response status: {}", response.status().as_u16());
        let res_text = response.text().await?;
        println!("Body: {}", res_text);
        return Ok(None);
    }

    let response_text: Vec<LLMResponse> = response.json().await?;
    Ok(Some(response_text))
}

fn get_system_instructions() -> String {
    String::from(
        "You are an AI assistant designed to translate Portuguese sentences into JSON objects for a personal expense management application. Your primary task is to accurately interpret the user's input and format it into a structured JSON object that can be easily processed by the application.",
    )
}

fn build_prompt(user_text: &str) -> String {
    let prompt_text = format!(
        "You will be provided with a Portuguese sentence:\n{portuguese_sentence}\n\nYour task is to translate the given Portuguese sentence into a JSON object. The JSON object should have the following structure:\n\n*   **command:** This field indicates the action to be performed. It can be either \"CREATE\" or \"LIST\".\n*   **object:** This field specifies the type of object being referred to in the sentence. It can be \"EXPENSE\", \"CREDIT CARD\", \"INCOME\", or \"CATEGORY\".\n*   **Other fields:** Depending on the \"Object\", include the necessary fields to describe the object.\n\nHere are the guidelines for determining the values of each field:\n\n*   **command:**\n    *   If the sentence indicates the creation of a new expense, income, or category, set the value to \"CREATE\".\n    *   If the sentence requests a list of existing expenses, incomes, or categories, set the value to \"LIST\".\n*   **object:**\n    *   If the sentence refers to an expense, set the value to \"EXPENSE\".\n    *   If the sentence refers to an income, set the value to \"INCOME\".\n    *   If the sentence refers to a category, set the value to \"CATEGORY\".\n    *   If the sentence refers to a credit card, set the value to \"CREDIT_CARD\".\n*   **other fields:**\n    *   For \"EXPENSE\", include fields such as \"description\", \"value\", \"category\", \"credit_card\", \"period\", and \"frequency\". \n    *   For \"INCOME\", include fields such as \"description\", \"value\", and \"date\".\n    *   For \"CATEGORY\" and \"CREDIT_CARD\", include fields such as \"name\" and \"description\".\n*    **constraints:**\n    *   If no clear description is provided, the \"description\" value should be the Portuguese word for the object\n    *   For date ranges, set the period as English labels (TODAY, YESTERDAY, THIS_WEEK, THIS_MONTH, PAST_3_MONTHS, LAST_YEAR, PAST_11_DAYS) rather than actual date intervals.\n    *   The \"frequency\" on the EXPENSE object refers to how often this expense repeats. Translate this to a label in plain English like (EVERY_MONTH,  EVERY_YEAR, EVERY_QUARTER, EVERY_SEMESTER, etc). If no clear frequency is provided it must be assign as ONCE.\n    *   If no clear category name is found on the EXPENSE input sentence, the category field must be set as NO_CATEGORY.\n    *   If the sentence refers to a EXPENSE using a credit card, but no clear credit card name is defined, set the credit_card field to DEFAULT.\n\nExample:\n\nInput: \"Crie uma despesa de R$50 com alimentação para amanhã.\"\n\nOutput:\n\n```json\n{{\n  \"command\": \"CREATE\",\n  \"object\": \"EXPENSE\",\n  \"description\": \"alimentação\",\n  \"value\": 50.00,\n  \"category\": \"alimentação\",\n  \"period\": \"TOMORROW\"\n}}\n```\n\nIf the input sentence is unclear or does not fit the expected format, return an error message in JSON format:\n\n```json\n{{\n  \"error\": \"Unable to parse the sentence. Please provide a clearer instruction.\"\n}}\n```",
        portuguese_sentence = user_text
    );
    return prompt_text;
}
