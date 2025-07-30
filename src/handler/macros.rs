macro_rules! get_user_id {
    ($e:expr) => {
        match $e.get::<i32>() {
            Some(c) => c,
            None => {
                log::error!("Could not retrieve User ID from request object");
                return HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
                .insert_header(ContentType::json())
                .body(json!({"error": "internal server error"}).to_string());
            }
        }
    };
}

pub(crate) use get_user_id;

macro_rules! unwrap_res_or_app_err_log_err {
    ($e:expr, $app_err:expr, $log_err_msg:literal) => {
        match $e {
            Ok(s) => s,
            Err(e) => {
                log::error!("{}: {}", $log_err_msg, e);
                return Err($app_err);
            }
        }
    };
}

pub(crate) use unwrap_res_or_app_err_log_err;

macro_rules! unwrap_res_or_app_err_log_warn {
    ($e:expr, $app_err:expr, $log_err_msg:literal) => {
        match $e {
            Ok(s) => s,
            Err(e) => {
                log::warn!("{}: {}", $log_err_msg, e);
                return Err($app_err);
            }
        }
    };
}

pub(crate) use unwrap_res_or_app_err_log_warn;

macro_rules! unwrap_opt_or_app_err_log_err {
    ($e:expr, $app_err:expr, $log_err_msg:literal) => {
        match $e {
            Some(s) => s,
            None => {
                log::error!("{}", $log_err_msg);
                return Err($app_err);
            }
        }
    };
}

pub(crate) use unwrap_opt_or_app_err_log_err;
