use actix_web::http::header::ContentType;
use actix_web::http::StatusCode;
use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use chrono::{DateTime, Duration, Utc};
use serde_json::json;

use crate::controllers::auth::*;
use crate::jwt::{generate_token, TokenClaims};
use crate::model::session::Session;
use crate::model::user::AuthType;
use crate::request_types::auth::{CreateUserReq, LoginUserReq};
use crate::state;

use super::util::{
    build_conflict_response, build_error_response, build_method_not_allowed,
    build_unauthorized_response,
};

pub async fn refresh_token(
    req: HttpRequest,
    app_state: web::Data<state::AppState>,
) -> HttpResponse {
    let ext = req.extensions();
    let claims = match ext.get::<TokenClaims>() {
        Some(c) => c,
        None => {
            log::error!("Could not retrieve claims from request object.");
            return HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
                .insert_header(ContentType::json())
                .body(json!({"success": false, "message": "internal server error"}).to_string());
        }
    };
    let user_email = claims.sub.clone();

    let mut session_db_con = match app_state.session_db.acquire().await {
        Ok(c) => c,
        Err(e) => {
            log::error!(
                "An error occurred when tried to acquire a connection to session db from pool: {}",
                e
            );
            return build_error_response();
        }
    };

    let session_req =
        match get_session_by_user_email(&mut *session_db_con, user_email.as_str()).await {
            Ok(s) => s,
            Err(e) => {
                log::error!(
                    "An error ocurred when tried to get session by user email: {}",
                    e
                );
                return build_error_response();
            }
        };
    let mut session = match session_req {
        Some(s) => s,
        None => {
            return build_unauthorized_response(None);
        }
    };

    if !session.is_refresh_token_valid() || user_email.ne(&session.user_email) {
        return build_unauthorized_response(None);
    }

    if !session.is_current_access_token_valid() {
        let now: DateTime<Utc> = Utc::now().into();
        let access_token_exp = now + Duration::minutes(15);

        let access_token = match generate_token(
            session.id.as_str(),
            &app_state.jwt_encoding_key,
            access_token_exp,
        ) {
            Ok(a) => a,
            Err(e) => {
                log::error!("An error occurred while generating access_token: {}", e);
                return build_error_response();
            }
        };

        session.current_access_token = access_token;
        session.current_access_token_expires_at = access_token_exp.to_rfc3339();

        match update_session(&mut *session_db_con, &session).await {
            Ok(_) => (),
            Err(err) => {
                log::error!("Error while trying update session: {}", err);
                return build_error_response();
            }
        };
    }

    HttpResponse::build(StatusCode::OK)
        .insert_header(ContentType::json())
        .body(
            json!({
            "success": true,
            "access_token": session.current_access_token,
            "access_token_exp": session.current_access_token_expires_at })
            .to_string(),
        )
}

pub async fn login_user(
    req: web::Json<LoginUserReq>,
    app_state: web::Data<state::AppState>,
) -> HttpResponse {
    //Get DB Connection for Transational Database
    let mut con = match app_state.db.acquire().await {
        Ok(c) => c,
        Err(e) => {
            log::error!(
                "An error occurred when tried to acquire a db connection from pool: {}",
                e
            );
            return build_error_response();
        }
    };

    let db_user_req = match get_user(req.email.as_str(), &mut *con).await {
        Ok(u) => u,
        Err(e) => {
            log::error!(
                "An error occurred when tried to retrieve user from database: {}",
                e
            );
            return build_error_response();
        }
    };

    let db_user = match db_user_req {
        Some(u) => {
            if u.auth_type != AuthType::UsernamePassword {
                return build_method_not_allowed(Some(String::from(
                    "auth method is not username and password",
                )));
            }
            u
        }
        None => {
            return build_unauthorized_response(Some(String::from("incorrect email or password")));
        }
    };

    let pwd = db_user.password.unwrap_or_else(|| String::from(""));
    let is_pwd_valid = match bcrypt::verify(req.password.as_str(), pwd.as_str()) {
        Ok(i) => i,
        Err(e) => {
            log::error!("An error occurred while verifying user's password: {}", e);
            return build_error_response();
        }
    };

    if !is_pwd_valid {
        return build_unauthorized_response(Some(String::from("incorrect email or password")));
    }

    let mut session_db_con = match app_state.session_db.acquire().await {
        Ok(c) => c,
        Err(e) => {
            log::error!(
                "An error occurred when tried to acquire a connection to session db from pool: {}",
                e
            );
            return build_error_response();
        }
    };

    let session_req =
        match get_session_by_user_email(&mut *session_db_con, db_user.email.as_str()).await {
            Ok(s) => s,
            Err(e) => {
                log::error!(
                    "An error ocurred when tried to get session by user email: {}",
                    e
                );
                return build_error_response();
            }
        };

    let mut new_session = false;
    let mut refresh_session = false;
    let mut session = session_req.unwrap_or_else(|| {
        new_session = true;
        Session::build(db_user.email.as_str())
    });

    let now: DateTime<Utc> = Utc::now().into();
    if !session.is_refresh_token_valid() {
        let refresh_token_exp = now + Duration::days(1);
        let refresh_token = match generate_token(
            db_user.email.as_str(),
            &app_state.jwt_encoding_key,
            refresh_token_exp.clone(),
        ) {
            Ok(a) => a,
            Err(e) => {
                log::error!("An error occurred while generating refresh_token: {}", e);
                return build_error_response();
            }
        };

        session.refresh_token = refresh_token;
        session.refresh_token_expires_at = refresh_token_exp.to_rfc3339();
        refresh_session = true;
    }

    if !session.is_current_access_token_valid() {
        let access_token_exp = now + Duration::minutes(15);
        let access_token = match generate_token(
            session.id.as_str(),
            &app_state.jwt_encoding_key,
            access_token_exp,
        ) {
            Ok(a) => a,
            Err(e) => {
                log::error!("An error occurred while generating access_token: {}", e);
                return build_error_response();
            }
        };

        session.current_access_token = access_token;
        session.current_access_token_expires_at = access_token_exp.to_rfc3339();
        refresh_session = true;
    }

    if new_session {
        log::warn!("DEBUG!");
        match create_session(&mut *session_db_con, &session).await {
            Ok(_) => (),
            Err(err) => {
                log::error!("Error attempting create session on database: {}", err);
                return build_error_response();
            }
        };
    } else if refresh_session {
        match update_session(&mut *session_db_con, &session).await {
            Ok(_) => (),
            Err(err) => {
                log::error!("Error attempting update session on database: {}", err);
                return build_error_response();
            }
        };
    }

    return HttpResponse::build(StatusCode::OK)
        .insert_header(ContentType::json())
        .body(
            json!({
                "user": {
                    "id": db_user.id,
                    "name": db_user.name,
                    "email": db_user.email,
                    "created_at": db_user.created_at,
                    "auth_type": db_user.auth_type,
                    "is_email_verified": db_user.is_email_verified,
                    "is_premium": db_user.is_premium
                },
                "access_token": session.current_access_token,
                "access_token_exp": session.current_access_token_expires_at,
                "refresh_token": session.refresh_token,
                "refresh_token_exp": session.refresh_token_expires_at,
                "success": true})
            .to_string(),
        );
}

pub async fn post_new_user(
    new_user: web::Json<CreateUserReq>,
    app_state: web::Data<state::AppState>,
) -> HttpResponse {
    let mut con = match app_state.db.acquire().await {
        Ok(c) => c,
        Err(e) => {
            log::error!(
                "An error occurred when tried to acquire a db connection from pool: {}",
                e
            );
            return build_error_response();
        }
    };

    let email_exists = match check_email_exists(new_user.email.as_str(), &mut *con).await {
        Ok(c) => c,
        Err(e) => {
            log::error!(
                "An error occurred when tried to check if email exists on the db: {}",
                e
            );
            return build_error_response();
        }
    };

    if email_exists {
        return build_conflict_response(Some(String::from("email already exists")));
    }

    let _ = match create_user(&mut *con, new_user.into()).await {
        Ok(u) => u,
        Err(e) => {
            log::error!(
                "An error occurred when tried to insert user on the database: {}",
                e
            );
            return build_error_response();
        }
    };

    return HttpResponse::build(StatusCode::CREATED)
        .insert_header(ContentType::json())
        .body(
            json!({
                "success": true,
                "message": "user created"
            })
            .to_string(),
        );
}
