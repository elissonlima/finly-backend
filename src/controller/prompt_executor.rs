use std::{collections::BTreeMap};

use chrono::{Duration, Utc};
use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};
use serde_json::json;

use crate::{controller::errors::GoogleControllerError, model::{Claims, GoogleServiceToken, GoogleServiceTokenApiResponse, LLMResponse, ServiceAccountKey}};

const PROJECT_ID: &str = "finlydigital-dev";
const LOCATION_ID: &str = "global";
const API_ENDPOINT: &str = "aiplatform.googleapis.com";
const MODEL_ID: &str = "gemini-2.5-flash";
const GENERATE_CONTENT_API: &str = "streamGenerateContent";

pub fn is_google_access_token_expired(token: &GoogleServiceToken) -> bool {
    let now = Utc::now();
    return now >= token.expires_at;
}

pub async fn get_access_token(
    key: &ServiceAccountKey
) -> Result<GoogleServiceToken, GoogleControllerError> {

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

    let token_res: GoogleServiceTokenApiResponse = client
        .post(&key.token_uri)
        .form(&params)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    let now = Utc::now();
    let expires_at = now + Duration::seconds(token_res.expires_in as i64);


    Ok(GoogleServiceToken { 
        access_token: token_res.access_token,
        token_type: token_res.token_type,
        expires_at
    })
}

pub async fn exec_prompt(
    user_text: &str,
    service_token: &GoogleServiceToken,
    prompt_case: PromptCase
) -> Result<Option<Vec<LLMResponse>>, GoogleControllerError> {
    let prompt_text = build_prompt(user_text, &prompt_case);
    let system_instructions = get_system_instructions(&prompt_case);

    let client = reqwest::Client::new();
    let url = format!(
        "https://{api_endpoint}/v1/projects/{project_id}/locations/{location_id}/publishers/google/models/{model_id}:{generate_content_api}",
        api_endpoint = API_ENDPOINT,
        project_id = PROJECT_ID,
        location_id = LOCATION_ID,
        model_id = MODEL_ID,
        generate_content_api = GENERATE_CONTENT_API
    );
    let response_schema = get_response_schema(&prompt_case);
    let response = client
        .post(url)
        .header("Authorization", format!("Bearer {}", service_token.access_token))
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
                    "responseSchema": response_schema
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
                        "category":"HARM_ATEGORY_HARASSMENT",
                        "threshold": "OFF"
                    },
                ]
            })
            .to_string(),
        )
        .send()
        .await?;

    if !response.status().is_success() {
        let status_code = response.status().as_u16();
        let res_body = response.text().await.unwrap_or_default();
        return Err(GoogleControllerError::HttpRequestError(API_ENDPOINT.to_string(), status_code, res_body));
    }

    let response_text: Vec<LLMResponse> = response.json().await?;
    Ok(Some(response_text))
}

pub enum PromptCase {
    UserInputToJson,
    CategoryIconExtract(String)
}

fn get_response_schema(prompt_case: &PromptCase) -> serde_json::Value {
    match prompt_case {
        PromptCase::UserInputToJson => json!({"type":"OBJECT","properties":{"command":{"type":"STRING"},"object":{"type":"STRING"},"description":{"type":"STRING"},"value":{"type":"INTEGER"},"category":{"type":"STRING"},"credit_card":{"type":"STRING"},"period":{"type":"STRING"},"frequency":{"type":"STRING"}},"required":["command","object","description"]}),
        PromptCase::CategoryIconExtract(_) => json!({"type":"OBJECT","properties":{"icon_name":{"type":"STRING"}},"required":["icon_name"]})
    }
}

fn get_system_instructions(prompt_case: &PromptCase) -> String {

    match prompt_case {
        PromptCase::UserInputToJson => String::from(
            "You are an AI assistant designed to translate Portuguese sentences into JSON objects for a personal expense management application. Your primary task is to accurately interpret the user's input and format it into a structured JSON object that can be easily processed by the application.",
        ),
        PromptCase::CategoryIconExtract(_) => String::from(
            "You are an expert in understanding expense categories and matching them to appropriate icons. Your task is to analyze a user-provided expense category description in Portuguese and select the most appropriate icon from a given list."
        )
    }
}

fn build_prompt(user_text: &str, prompt_case: &PromptCase) -> String {
    let prompt_text = match prompt_case {
        PromptCase::CategoryIconExtract(i_list) => format!("You will be provided with an expense category description in Portuguese and a list of available icons.\n\nExpense category description (Portuguese): {expense_category_description}\nAvailable icons: {icon_list}\n\nFollow these steps:\n1. Analyze the expense category description to understand its meaning and purpose.\n2. Compare the expense category to each icon in the list to determine the best match.\n3. Output the name of the selected icon in JSON format.\n\nExample output:\n{{\"icon_name\": \"home\"}}",
            expense_category_description = user_text,
            icon_list = i_list
        ),
        PromptCase::UserInputToJson => format!(
        "You will be provided with a Portuguese sentence:\n{portuguese_sentence}\n\nYour task is to translate the given Portuguese sentence into a JSON object. The JSON object should have the following structure:\n\n*   **command:** This field indicates the action to be performed. It can be either \"CREATE\" or \"LIST\".\n*   **object:** This field specifies the type of object being referred to in the sentence. It can be \"EXPENSE\", \"CREDIT CARD\", \"INCOME\", or \"CATEGORY\".\n*   **Other fields:** Depending on the \"Object\", include the necessary fields to describe the object.\n\nHere are the guidelines for determining the values of each field:\n\n*   **command:**\n    *   If the sentence indicates the creation of a new expense, income, or category, set the value to \"CREATE\".\n    *   If the sentence requests a list of existing expenses, incomes, or categories, set the value to \"LIST\".\n*   **object:**\n    *   If the sentence refers to an expense, set the value to \"EXPENSE\".\n    *   If the sentence refers to an income, set the value to \"INCOME\".\n    *   If the sentence refers to a category, set the value to \"CATEGORY\".\n    *   If the sentence refers to a credit card, set the value to \"CREDIT_CARD\".\n*   **other fields:**\n    *   For \"EXPENSE\", include fields such as \"description\", \"value\", \"category\", \"credit_card\", \"period\", and \"frequency\". \n    *   For \"INCOME\", include fields such as \"description\", \"value\", and \"date\".\n    *   For \"CATEGORY\" and \"CREDIT_CARD\", include fields such as \"name\" and \"description\".\n*    **constraints:**\n    *   If no clear description is provided, the \"description\" value should be the Portuguese word for the object\n    *   For date ranges, set the period as English labels (TODAY, YESTERDAY, THIS_WEEK, THIS_MONTH, PAST_3_MONTHS, LAST_YEAR, PAST_11_DAYS) rather than actual date intervals.\n    *   The \"frequency\" on the EXPENSE object refers to how often this expense repeats. Translate this to a label in plain English like (EVERY_MONTH,  EVERY_YEAR, EVERY_QUARTER, EVERY_SEMESTER, etc). If no clear frequency is provided it must be assign as ONCE.\n    *   If no clear category name is found on the EXPENSE input sentence, the category field must be set as NO_CATEGORY.\n    *   If the sentence refers to a EXPENSE using a credit card, but no clear credit card name is defined, set the credit_card field to DEFAULT.\n\nExample:\n\nInput: \"Crie uma despesa de R$50 com alimentação para amanhã.\"\n\nOutput:\n\n```json\n{{\n  \"command\": \"CREATE\",\n  \"object\": \"EXPENSE\",\n  \"description\": \"alimentação\",\n  \"value\": 50.00,\n  \"category\": \"alimentação\",\n  \"period\": \"TOMORROW\"\n}}\n```\n\nIf the input sentence is unclear or does not fit the expected format, return an error message in JSON format:\n\n```json\n{{\n  \"error\": \"Unable to parse the sentence. Please provide a clearer instruction.\"\n}}\n```",
        portuguese_sentence = user_text
     )
    };
    return prompt_text;
}
