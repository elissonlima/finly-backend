use std::process::{Command, Stdio};

use actix_web::http::header::ContentType;
use actix_web::http::StatusCode;
use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use chrono::{DateTime, Duration, Utc};
use serde_json::json;
use uuid::Uuid;

use crate::controllers::auth::*;
use crate::controllers::session_mgm::{
    create_session, get_session_by_session_id, get_session_by_user_email, reset_session,
    update_session,
};
use crate::jwt::generate_token;
use crate::model::session::Session;
use crate::model::user::{AuthType, User};
use crate::request_types::auth::{CreateUserReq, GoogleSignInReq, LoginUserReq};
use crate::state;

use super::macros;
use super::util::{
    build_conflict_response, build_error_response, build_method_not_allowed,
    build_unauthorized_response,
};

pub async fn refresh_token(
    req: HttpRequest,
    app_state: web::Data<state::AppState>,
) -> HttpResponse {
    let ext = req.extensions();
    let claims = macros::unwrap_opt_or_error!(
        ext.get::<crate::jwt::TokenClaims>(),
        "it wasn't possible to get TokenClaims from Request Object"
    );

    let session_id = macros::uuid_from_str!(claims.sub.as_str());
    let mut db = macros::get_database_connection!(app_state);
    let mut session = macros::unwrap_opt_or_unauthorize!(macros::run_async_unwrap!(
        get_session_by_session_id(&mut *db, &session_id),
        "an error ocurred when tried to get session by id"
    ));

    if !session.is_refresh_token_valid() {
        return build_unauthorized_response(None);
    }

    if !session.is_current_access_token_valid() {
        let now: DateTime<Utc> = Utc::now().into();
        let access_token_exp = now + Duration::minutes(15);

        let access_token = macros::unwrap_res_or_error!(
            generate_token(
                session.id.to_string().as_str(),
                &app_state.jwt_encoding_key,
                access_token_exp,
            ),
            "an error occurred while generating access_token"
        );

        session.current_access_token = access_token;
        session.current_access_token_expires_at = access_token_exp;

        macros::run_async_unwrap!(
            update_session(&mut *db, &session),
            "error while trying update session"
        );
    }

    HttpResponse::build(StatusCode::OK)
        .insert_header(ContentType::json())
        .body(
            json!({
            "success": true,
            "access_token": session.current_access_token,
            "access_token_exp": session.current_access_token_expires_at.to_rfc3339()})
            .to_string(),
        )
}

pub async fn login_user(
    req: web::Json<LoginUserReq>,
    app_state: web::Data<state::AppState>,
) -> HttpResponse {
    //Get DB Connection for Transational Database
    let mut con = macros::get_database_connection!(app_state);

    let db_user = macros::unwrap_opt_or_unauthorize!(
        macros::run_async_unwrap!(
            get_user(req.email.as_str(), &mut *con),
            "an error occurred when tried to retrieve user from database"
        ),
        "can't find user on DB: Invalid email or password",
        "incorrect email or password"
    );

    if db_user.auth_type != AuthType::UsernamePassword {
        return build_method_not_allowed(Some(String::from(
            "auth method is not username and password",
        )));
    }

    let pwd = db_user.password.unwrap_or_else(|| String::from(""));
    let is_pwd_valid = macros::unwrap_res_or_error!(
        bcrypt::verify(req.password.as_str(), pwd.as_str()),
        "an error occurred while verifying user's password"
    );

    if !is_pwd_valid {
        return build_unauthorized_response(Some(String::from("incorrect email or password")));
    }

    let session_req = macros::run_async_unwrap!(
        get_session_by_user_email(&mut *con, db_user.email.as_str()),
        "an error ocurred when tried to get session by user email"
    );

    let mut new_session = false;
    let mut refresh_session = false;
    let mut session = session_req.unwrap_or_else(|| {
        new_session = true;
        Session::build(db_user.email.as_str())
    });

    if !new_session {
        session.id = Uuid::new_v4();
        refresh_session = true;
    }

    let now: DateTime<Utc> = Utc::now().into();
    let refresh_token_exp = now + Duration::days(1);
    let refresh_token = macros::unwrap_res_or_error!(
        generate_token(
            session.id.to_string().as_str(),
            &app_state.jwt_encoding_key,
            refresh_token_exp.clone(),
        ),
        "an error occurred while generating refresh_token"
    );

    session.refresh_token = refresh_token;
    session.refresh_token_expires_at = refresh_token_exp;

    let access_token_exp = now + Duration::minutes(15);
    let access_token = macros::unwrap_res_or_error!(
        generate_token(
            session.id.to_string().as_str(),
            &app_state.jwt_encoding_key,
            access_token_exp,
        ),
        "an error occurred while generating access_token"
    );

    session.current_access_token = access_token;
    session.current_access_token_expires_at = access_token_exp;

    if new_session {
        macros::run_async_unwrap!(
            create_session(&mut *con, &session),
            "error attempting create session on database"
        );
    } else if refresh_session {
        macros::run_async_unwrap!(
            reset_session(&mut *con, &session),
            "error attempting update session on database"
        );
    }

    return HttpResponse::build(StatusCode::OK)
        .insert_header(ContentType::json())
        .body(
            json!({
                "user": {
                    "id": db_user.id,
                    "name": db_user.name,
                    "email": db_user.email,
                    "created_at": db_user.created_at.to_rfc3339(),
                    "auth_type": db_user.auth_type,
                    "is_email_verified": db_user.is_email_verified,
                    "is_premium": db_user.is_premium
                },
                "access_token": session.current_access_token,
                "access_token_exp": session.current_access_token_expires_at.to_rfc3339(),
                "refresh_token": session.refresh_token,
                "refresh_token_exp": session.refresh_token_expires_at.to_rfc3339(),
                "success": true})
            .to_string(),
        );
}

pub async fn post_new_user(
    new_user: web::Json<CreateUserReq>,
    app_state: web::Data<state::AppState>,
) -> HttpResponse {
    let mut con = macros::get_database_connection!(app_state);

    let email_exists = macros::run_async_unwrap!(
        check_email_exists(new_user.email.as_str(), &mut *con),
        "an error occurred when tried to check if email exists on the db"
    );

    if email_exists {
        return build_conflict_response(Some("email already exists".to_string()));
    }

    let usr_obj = macros::unwrap_res_or_error!(
        User::from_signup_request(new_user.into()),
        "an error ocurred when tried to create user object"
    );

    let _ = macros::run_async_unwrap!(
        create_user(&mut *con, &usr_obj),
        "an error occurred when tried to insert user on the database"
    );

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

pub async fn google_signin(
    req: web::Json<GoogleSignInReq>,
    app_state: web::Data<state::AppState>,
) -> HttpResponse {
    let oauth_res = match Command::new("sh")
        .arg("-c")
        .arg("google_oauth_api_client")
        .env("GOOGLE_OAUTH_EXEC_INPUT_TOKEN", req.token.as_str())
        .env(
            "GOOGLE_OAUTH_EXEC_CLIENT_ID",
            app_state.google_oauth_client_id.as_str(),
        )
        .stdout(Stdio::piped())
        .output()
    {
        Ok(c) => {
            let o = std::str::from_utf8(&c.stdout).unwrap_or_else(|_| "");
            let e = std::str::from_utf8(&c.stderr).unwrap_or_else(|_| "");
            let status_code = c.status.code().unwrap_or_else(|| -1);
            if status_code == 0 {
                Some(String::from(o))
            } else {
                log::warn!("Could not get google_oauth user information: {}", e);
                None
            }
        }
        Err(e) => {
            log::error!(
                "An error occurred when tried to run google_oauth_api_client {}",
                e
            );
            return build_error_response();
        }
    };

    if oauth_res.is_none() {
        return build_unauthorized_response(Some("invalid token".to_string()));
    }

    let oauth_user_data: GoogleOauthUserInformation = match serde_json::from_str(
        oauth_res.unwrap().as_str(),
    ) {
        Ok(o) => o,
        Err(e) => {
            log::error!("An error occurred when tried to convert the google oauth information to object: {}", e);
            return build_error_response();
        }
    };

    let mut con = macros::get_database_connection!(app_state);

    //CHECK IF THERE'S A USER WITH SAME EMAIL
    let mut new_user = false;
    let mut usr = macros::run_async_unwrap!(
        get_user(oauth_user_data.email.as_str(), &mut *con),
        "error trying to get user from database"
    )
    .unwrap_or_else(|| {
        new_user = true;
        User::from_google(oauth_user_data)
    });

    if usr.auth_type != AuthType::Google {
        return build_method_not_allowed(Some("invalid auth method".to_string()));
    }

    if new_user {
        let usr_database = macros::run_async_unwrap!(
            create_user(&mut *con, &usr),
            "an error ocurred when tried to insert a new user on the database"
        );
        usr = usr_database.clone();
    }

    //GENERATE REFRESH_TOKEN AND ACCESS_TOKEN
    let session_req = macros::run_async_unwrap!(
        get_session_by_user_email(&mut *con, usr.email.as_str()),
        "an error ocurred when tried to get session by user email"
    );

    let mut new_session = false;
    let mut refresh_session = false;
    let mut session = session_req.unwrap_or_else(|| {
        new_session = true;
        Session::build(usr.email.as_str())
    });

    if !new_session {
        session.id = Uuid::new_v4();
        refresh_session = true;
    }

    let now: DateTime<Utc> = Utc::now().into();
    let refresh_token_exp = now + Duration::days(1);
    let refresh_token = macros::unwrap_res_or_error!(
        generate_token(
            session.id.to_string().as_str(),
            &app_state.jwt_encoding_key,
            refresh_token_exp.clone(),
        ),
        "an error occurred while generating refresh_token"
    );

    session.refresh_token = refresh_token;
    session.refresh_token_expires_at = refresh_token_exp;

    let access_token_exp = now + Duration::minutes(15);
    let access_token = macros::unwrap_res_or_error!(
        generate_token(
            session.id.to_string().as_str(),
            &app_state.jwt_encoding_key,
            access_token_exp,
        ),
        "an error occurred while generating access_token"
    );

    session.current_access_token = access_token;
    session.current_access_token_expires_at = access_token_exp;

    if new_session {
        macros::run_async_unwrap!(
            create_session(&mut *con, &session),
            "error attempting create session on database"
        );
    } else if refresh_session {
        macros::run_async_unwrap!(
            reset_session(&mut *con, &session),
            "error attempting update session on database"
        );
    }

    let mut sts = StatusCode::OK;
    if new_user {
        sts = StatusCode::CREATED
    }
    HttpResponse::build(sts)
        .insert_header(ContentType::json())
        .body(
            json!({
                "user": {
                    "id": usr.id,
                    "name": usr.name,
                    "email": usr.email,
                    "created_at": usr.created_at.to_rfc3339(),
                    "auth_type": usr.auth_type,
                    "is_email_verified": usr.is_email_verified,
                    "is_premium": usr.is_premium
                },
                "access_token": session.current_access_token,
                "access_token_exp": session.current_access_token_expires_at.to_rfc3339(),
                "refresh_token": session.refresh_token,
                "refresh_token_exp": session.refresh_token_expires_at.to_rfc3339(),
                "success": true})
            .to_string(),
        )
}
