use actix_web::web;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct CreateUserReq {
    pub email: String,
    pub name: String,
    pub password: String,
}

impl From<web::Json<CreateUserReq>> for CreateUserReq {
    fn from(user: web::Json<CreateUserReq>) -> Self {
        CreateUserReq {
            email: user.email.clone(),
            name: user.name.clone(),
            password: user.password.clone(),
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct LoginUserReq {
    pub email: String,
    pub password: String,
}

impl From<web::Json<LoginUserReq>> for LoginUserReq {
    fn from(u: web::Json<LoginUserReq>) -> Self {
        LoginUserReq {
            email: u.email.clone(),
            password: u.password.clone(),
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct GoogleSignInReq {
    pub token: String,
}

impl From<web::Json<GoogleSignInReq>> for GoogleSignInReq {
    fn from(payload: web::Json<GoogleSignInReq>) -> Self {
        GoogleSignInReq {
            token: payload.token.clone(),
        }
    }
}
