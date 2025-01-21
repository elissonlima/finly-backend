use actix_web::web;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct CreateResetPasswordReq {
    pub email: String,
}

impl From<web::Json<CreateResetPasswordReq>> for CreateResetPasswordReq {
    fn from(res: web::Json<CreateResetPasswordReq>) -> Self {
        CreateResetPasswordReq {
            email: res.email.clone(),
        }
    }
}

#[derive(Deserialize)]
pub struct ResetPasswordFormReq {
    pub t: String,
}

#[derive(Deserialize)]
pub struct DoResetPasswordReq {
    pub password: String,
    #[serde(alias = "confirmPassword")]
    pub confirm_password: String,
    pub t: String,
}
