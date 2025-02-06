use std::{io::Read, path::PathBuf};

use actix_files::NamedFile;
use actix_web::{
    http::{header::ContentType, StatusCode},
    web::{self},
    HttpResponse,
};
use serde_json::json;

use crate::{
    controllers::{
        auth::{check_email_exists, update_password},
        reset_password::{
            self, check_reset_password_id, get_reset_password_email_valid_id,
            toggle_reset_password_flag,
        },
        ses::send_reset_password_email,
    },
    jwt::{generate_token_hs256, verify_token_hs256},
    request_types::reset_password::{
        CreateResetPasswordReq, DoResetPasswordReq, ResetPasswordFormReq,
    },
    state,
};

use super::{macros, util::build_conflict_response};

pub async fn create_reset_password_request(
    req: web::Json<CreateResetPasswordReq>,
    app_state: web::Data<state::AppState>,
) -> HttpResponse {
    let mut con = macros::get_database_connection!(app_state);
    let email_exists = macros::run_async_unwrap!(
        check_email_exists(req.email.as_str(), &mut *con),
        "an error occurred when tried to check if email exists on the db"
    );

    //Send CREATED to avoid requests to check whether the email
    //exists on DB or not.
    if !email_exists {
        return HttpResponse::build(StatusCode::CREATED)
            .insert_header(ContentType::json())
            .body(
                json!({
                "success": true,
                "message": "email sent"
                })
                .to_string(),
            );
    }

    //Check if the email does not have an reset record already created
    let reset_exists = macros::run_async_unwrap!(
    reset_password::get_reset_password_expiration_if_exists(req.email.as_str(), &mut *con),
    "an error occurred when tried to check if the user has already created an reset password record");
    if let Some(x) = reset_exists {
        return build_conflict_response(Some(x));
    }

    //Create reset password record on DB
    let rec = macros::run_async_unwrap!(
        reset_password::create_reset_password(req.email.as_str(), &mut *con),
        "an error occurred when tried to create a reset password record in db"
    );

    // Create token
    let token = macros::unwrap_res_or_error!(
        generate_token_hs256(rec.id.to_string().as_str(), rec.expires_at),
        "error when building token for reset password"
    );

    //TODO - send password reset link
    let reset_password_link = format!("https://192.168.1.19:3000/password/reset?t={}", token);
    macros::run_async_unwrap!(
        send_reset_password_email(req.email.as_str(), reset_password_link.as_str()),
        "an error occurred when tried to send the reset password link throught email"
    );

    HttpResponse::build(StatusCode::CREATED)
        .insert_header(ContentType::json())
        .body(
            json!({
            "success": true,
            "message": "email sent"
            })
            .to_string(),
        )
}

pub async fn reset_password_form(
    token: web::Query<ResetPasswordFormReq>,
    app_state: web::Data<state::AppState>,
) -> HttpResponse {
    let mut con = macros::get_database_connection!(app_state);

    let claims = match verify_token_hs256(token.t.as_str()) {
        Ok(c) => c,
        Err(e) => {
            log::warn!(
                "Error while trying to verify token claims for reset password: {}",
                e
            );
            let path: PathBuf = "/app/html/generic_message.html".parse().unwrap();
            let mut res_content = String::from("");
            let _ = NamedFile::open(path)
                .unwrap()
                .read_to_string(&mut res_content);

            let res_content = res_content.replace("{header}", "N&atilde;o autorizado");
            let res_content = res_content.replace(
                "{message}",
                "Token expirado. Solicite a mudan&ccedil;a de senha novamente.",
            );

            return HttpResponse::Unauthorized()
                .insert_header(ContentType::html())
                .body(res_content);
        }
    };

    let res_id = macros::uuid_from_str!(claims.sub.as_str());
    let check_token = macros::run_async_unwrap!(
        check_reset_password_id(&res_id, &mut *con),
        "error while trying to verify the reset password record in db"
    );

    if !check_token {
        let path: PathBuf = "/app/html/generic_message.html".parse().unwrap();
        let mut res_content = String::from("");
        let _ = NamedFile::open(path)
            .unwrap()
            .read_to_string(&mut res_content);
        let res_content = res_content.replace("{header}", "N&atilde;o autorizado");
        let res_content = res_content.replace(
            "{message}",
            "Token expirado. Solicite a mudan&ccedil;a de senha novamente.",
        );

        return HttpResponse::Unauthorized()
            .insert_header(ContentType::html())
            .body(res_content);
    }

    let path: PathBuf = "/app/html/reset_password.html".parse().unwrap();
    let mut res_content = String::from("");
    let _ = NamedFile::open(path)
        .unwrap()
        .read_to_string(&mut res_content);

    let res_content = res_content.replace("{resetToken}", token.t.as_str());

    HttpResponse::Ok()
        .insert_header(ContentType::html())
        .body(res_content)
}

pub async fn do_reset_password(
    req: web::Form<DoResetPasswordReq>,
    app_state: web::Data<state::AppState>,
) -> HttpResponse {
    // Verify if password and password confirmation match
    if req.password != req.confirm_password {
        return HttpResponse::build(StatusCode::SEE_OTHER)
            .insert_header((
                "Location",
                format!("/password/reset?e=PASSWORD_NOT_MATCH&t={}", req.t),
            ))
            .body("");
    }

    let mut con = match app_state.db.acquire().await {
        Ok(c) => c,
        Err(e) => {
            log::error!(
                "An error occurred when tried to acquire a db connection from pool: {}",
                e
            );
            let path: PathBuf = "/app/html/generic_message.html".parse().unwrap();
            let mut res_content = String::from("");
            let _ = NamedFile::open(path)
                .unwrap()
                .read_to_string(&mut res_content);

            let res_content = res_content.replace("{header}", "INTERNAL SERVER ERROR");
            let res_content =
                res_content.replace("{message}", "Um erro ocorreu. Tente novamente mais tarde.");
            return HttpResponse::Unauthorized()
                .insert_header(ContentType::html())
                .body(res_content);
        }
    };

    //Validate token
    let claims = match verify_token_hs256(req.t.as_str()) {
        Ok(c) => c,
        Err(e) => {
            log::warn!(
                "Error while trying to verify token claims for reset password: {}",
                e
            );
            let path: PathBuf = "/app/html/generic_message.html".parse().unwrap();
            let mut res_content = String::from("");
            let _ = NamedFile::open(path)
                .unwrap()
                .read_to_string(&mut res_content);

            let res_content = res_content.replace("{header}", "N&atilde;o autorizado");
            let res_content = res_content.replace(
                "{message}",
                "Token expirado. Solicite a mudan&ccedil;a de senha novamente.",
            );

            return HttpResponse::Unauthorized()
                .insert_header(ContentType::html())
                .body(res_content);
        }
    };

    let res_id = macros::uuid_from_str!(claims.sub.as_str());

    let user_email = match get_reset_password_email_valid_id(&res_id, &mut *con).await {
        Ok(c) => c,
        Err(e) => {
            log::warn!(
                "Error while trying to verify the reset password record in db: {}",
                e
            );
            None
        }
    };

    if user_email.is_none() {
        let path: PathBuf = "/app/html/generic_message.html".parse().unwrap();
        let mut res_content = String::from("");
        let _ = NamedFile::open(path)
            .unwrap()
            .read_to_string(&mut res_content);

        let res_content = res_content.replace("{header}", "N&atilde;o autorizado");
        let res_content = res_content.replace(
            "{message}",
            "Token expirado. Solicite a mudan&ccedil;a de senha novamente.",
        );
        return HttpResponse::Unauthorized()
            .insert_header(ContentType::html())
            .body(res_content);
    }

    //TODO - update password on user's table
    match update_password(
        user_email.unwrap().as_str(),
        req.confirm_password.as_str(),
        &mut *con,
    )
    .await
    {
        Ok(_) => {}
        Err(e) => {
            log::error!(
                "An error ocurred when tried to update user's password: {}",
                e
            );
            let path: PathBuf = "/app/html/generic_message.html".parse().unwrap();
            let mut res_content = String::from("");
            let _ = NamedFile::open(path)
                .unwrap()
                .read_to_string(&mut res_content);

            let res_content = res_content.replace("{header}", "INTERNAL SERVER ERROR");
            let res_content =
                res_content.replace("{message}", "Um erro ocorreu. Tente novamente mais tarde.");
            return HttpResponse::Unauthorized()
                .insert_header(ContentType::html())
                .body(res_content);
        }
    }

    //TODO - update reset password status: is_reset_password = 1
    let _ = match toggle_reset_password_flag(&res_id, &mut *con).await {
        Ok(_) => {}
        Err(e) => {
            log::error!(
                "An error occured when tried to update the reset password record: {}",
                e
            );
            let path: PathBuf = "/app/html/generic_message.html".parse().unwrap();
            let mut res_content = String::from("");
            let _ = NamedFile::open(path)
                .unwrap()
                .read_to_string(&mut res_content);

            let res_content = res_content.replace("{header}", "INTERNAL SERVER ERROR");
            let res_content =
                res_content.replace("{message}", "Um erro ocorreu. Tente novamente mais tarde.");
            return HttpResponse::Unauthorized()
                .insert_header(ContentType::html())
                .body(res_content);
        }
    };

    let path: PathBuf = "/app/html/generic_message.html".parse().unwrap();
    let mut res_content = String::from("");
    let _ = NamedFile::open(path)
        .unwrap()
        .read_to_string(&mut res_content);

    let res_content = res_content.replace("{header}", "Senha atualizada");
    let res_content = res_content.replace("{message}", "Senha atualizada com sucesso.");
    return HttpResponse::Unauthorized()
        .insert_header(ContentType::html())
        .body(res_content);
}
