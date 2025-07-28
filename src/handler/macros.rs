macro_rules! get_database_connection {
    ($e:expr) => {
        match $e.pool.acquire().await {
            Ok(c) => c,
            Err(e) => {
                log::error!(
                    "An error occurred when tried to acquire a connection to session db from pool: {}",
                    e
                );
                return HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
                .insert_header(ContentType::json())
                .body(json!({"error": "internal server error"}).to_string());
            }
        }
    };
}

pub(crate) use get_database_connection;

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

macro_rules! unwrap_res_and_error {
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

pub(crate) use unwrap_res_and_error;

macro_rules! unwrap_res_and_warn {
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

pub(crate) use unwrap_res_and_warn;
