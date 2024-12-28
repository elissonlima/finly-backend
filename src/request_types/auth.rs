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
