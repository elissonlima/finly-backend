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

macro_rules! unwrap_res_or_error {
    ($e:expr, $log_err_msg:literal) => {
        match $e {
            Ok(s) => s,
            Err(e) => {
                log::error!("{}: {}", $log_err_msg, e);
                return HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
                .insert_header(ContentType::json())
                .body(json!({"error": "internal server error"}).to_string());
            }
        }
    };
}

pub(crate) use unwrap_res_or_error;

macro_rules! unwrap_res_or_bad_request {
    ($e:expr, $log_err_msg:literal) => {
        match $e {
            Ok(s) => s,
            Err(e) => {
                log::warn!("{}: {}", $log_err_msg, e);
                return HttpResponse::build(StatusCode::BAD_REQUEST)
                .insert_header(ContentType::json())
                .body(json!({"error": "bad request"}).to_string());
            }
        }
    };
}

pub(crate) use unwrap_res_or_bad_request;
