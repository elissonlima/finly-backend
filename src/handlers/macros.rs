macro_rules! get_session {
    ($e:expr) => {
        match $e.get::<crate::model::session::Session>() {
            Some(c) => c,
            None => {
                log::error!("Could not retrieve session from request object");
                return crate::handlers::util::build_error_response();
            }
        }
    };
}

pub(crate) use get_session;

macro_rules! get_database_connection {
    ($e:expr) => {
        match $e.db.acquire().await {
            Ok(c) => c,
            Err(e) => {
                log::error!(
                    "An error occurred when tried to acquire a connection to session db from pool: {}",
                    e
                );
                return crate::handlers::util::build_error_response();
            }
        }
    };
}

pub(crate) use get_database_connection;

macro_rules! run_async_unwrap {
    ($e:expr, $log_msg: literal) => {
        match $e.await {
            Ok(d) => d,
            Err(e) => {
                log::error!("{}: {}", $log_msg, e);
                return crate::handlers::util::build_error_response();
            }
        }
    };
}

pub(crate) use run_async_unwrap;

macro_rules! run_async_or {
    ($e: expr, $and_e: expr) => {
        match $e.await {
            Ok(d) => d,
            Err(e) => {
                log::warn!("An error was captured: {}", e);
                $and_e
            }
        }
    };
}

pub(crate) use run_async_or;

macro_rules! unwrap_opt_or_unauthorize {
    ($e:expr) => {
        match $e {
            Some(e) => e,
            None => {
                log::warn!("{} returned None", stringify!($e));
                return crate::handlers::util::build_unauthorized_response(None);
            }
        }
    };
    ($e:expr, $log_msg:literal) => {
        match $e {
            Some(e) => e,
            None => {
                log::warn!("{}", $log_msg);
                return crate::handlers::util::build_unauthorized_response(None);
            }
        }
    };
    ($e:expr, $log_msg:literal, $err_msg:literal) => {
        match $e {
            Some(e) => e,
            None => {
                log::warn!("{}", $log_msg);
                return crate::handlers::util::build_unauthorized_response(Some($err_msg.to_string()));
            }
        }
    };
}

pub(crate) use unwrap_opt_or_unauthorize;

macro_rules! unwrap_opt_or_error {
    ($e:expr) => {
        match $e {
            Some(e) => e,
            None => {
                log::warn!("{} returned None", stringify!($e));
                return crate::handlers::util::build_error_response();
            }
        }
    };

    ($e:expr, $log_err_msg:literal) => {
        match $e {
            Some(e) => e,
            None => {
                log::error!($log_err_msg);
                return crate::handlers::util::build_error_response();
            }
        }
    };
}

pub(crate) use unwrap_opt_or_error;

macro_rules! unwrap_res_or_error {
    ($e:expr, $log_err_msg:literal) => {
        match $e {
            Ok(s) => s,
            Err(e) => {
                log::error!("{}: {}", $log_err_msg, e);
                return crate::handlers::util::build_error_response();
            }
        }
    };
}

pub(crate) use unwrap_res_or_error;

macro_rules! uuid_from_str {
    ($e: expr) => {
        match uuid::Uuid::parse_str($e) {
            Ok(u) => u,
            Err(e) => {
                log::error!("can't parse uuid from string: {}", e);
                return crate::handlers::util::build_error_response();
            }
        }
    };
}

pub(crate) use uuid_from_str;

macro_rules! begin_transaction {
    ($e: expr) => {
        match $e.begin().await {
            Ok(tx) => tx,
            Err(e) => {
                log::error!("can't begin transaction: {}", e);
                return crate::handlers::util::build_error_response();
            }
        }
    };
}

pub(crate) use begin_transaction;

macro_rules! commit_transaction {
    ($e: expr) => {
        match $e.commit().await {
            Ok(_) => (),
            Err(e) => {
                log::error!("can't commit transaction: {}", e);
                return crate::handlers::util::build_error_response();
            }
        }
    };
}

pub(crate) use commit_transaction;
