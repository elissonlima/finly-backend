use actix_web::{error, http::StatusCode, HttpResponse};
use derive_more::{Display, Error};
use serde::{Deserialize, Serialize};

use crate::controller;

#[derive(Debug, Display, Error, Serialize, Deserialize)] // Derive Serialize and Deserialize
#[serde(rename_all = "snake_case")]
pub enum AppError {
    #[display("Unauthorized access")]
    Unauthorized,
    #[display("Internal server error")]
    InternalServerError,
    #[display("LLM Object Processing Error")]
    LLMObjectProcessingError,
    #[display("Invalid input for field: {}", field)]
    InvalidInput { field: String },
    #[display("Bad Request")]
    BadRequest,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ErrorResponse {
    pub message: String,
    pub status_code: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

impl error::ResponseError for AppError {
    fn status_code(&self) -> StatusCode {
        match *self {
            AppError::Unauthorized { .. } => StatusCode::UNAUTHORIZED,
            AppError::InternalServerError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::InvalidInput { .. } => StatusCode::BAD_REQUEST,
            AppError::LLMObjectProcessingError {..} => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::BadRequest => StatusCode::BAD_REQUEST
        }
    }

    fn error_response(&self) -> HttpResponse<actix_web::body::BoxBody> {
        let status_code = self.status_code();
        let message = self.to_string();
        let details = match self {
            AppError::InvalidInput { field } => {
                Some(format!("The field '{}' has an invalid value.", field))
            },
            AppError::LLMObjectProcessingError => {
                Some(format!("Could not convert the user input into a API action"))
            }
            // Add more specific details for other error types if needed
            _ => None,
        };

        let error_response = ErrorResponse {
            message,
            status_code: status_code.as_u16(),
            details,
        };

        HttpResponse::build(status_code)
            .insert_header(actix_web::http::header::ContentType::json())
            .json(error_response)
    }
}

impl From<controller::errors::GoogleControllerError> for AppError {
    fn from(_err: controller::errors::GoogleControllerError) -> Self {
        AppError::InternalServerError
    }
}