use std::future::{ready, Ready};

use actix_web::{
    body::EitherBody,
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    http::{
        header::{ContentType, AUTHORIZATION},
        StatusCode,
    },
    web::Data,
    Error, HttpMessage, HttpResponse,
};

use futures_util::{future::LocalBoxFuture, FutureExt};
use serde_json::json;

use crate::{jwt::verify_token, state::AppState};

pub struct Authentication;

impl<S, B> Transform<S, ServiceRequest> for Authentication
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type InitError = ();
    type Transform = AuthenticationMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthenticationMiddleware { service }))
    }
}

pub struct AuthenticationMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for AuthenticationMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let auth = req.headers().get(AUTHORIZATION);
        if auth.is_none() {
            let http_res = HttpResponse::build(StatusCode::UNAUTHORIZED)
                .insert_header(ContentType::json())
                .body(json!({"success": false, "message": "unauthorized"}).to_string());
            let (http_req, _) = req.into_parts();
            let res = ServiceResponse::new(http_req, http_res);
            return (async move { Ok(res.map_into_right_body()) }).boxed_local();
        }

        let token_h = auth.and_then(|t| t.to_str().ok()).unwrap_or_default();
        let token_s: Vec<&str> = token_h.split_whitespace().collect();
        let mut token = "";

        if token_s.len() > 1 {
            token = token_s[1];
        }

        let app_data = match req.app_data::<Data<AppState>>() {
            Some(e) => e,
            None => {
                log::error!("Error trying to access app state from middleware.");
                let http_res = HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
                    .insert_header(ContentType::json())
                    .body(
                        json!({"success": false, "message": "internal server error"}).to_string(),
                    );
                let (http_req, _) = req.into_parts();
                let res = ServiceResponse::new(http_req, http_res);
                return (async move { Ok(res.map_into_right_body()) }).boxed_local();
            }
        };

        let claims = match verify_token(token, &app_data.jwt_decoding_key) {
            Ok(c) => c,
            Err(e) => {
                log::error!("Error trying to verify the token: {}", e);
                let http_res = HttpResponse::build(StatusCode::UNAUTHORIZED)
                    .insert_header(ContentType::json())
                    .body(json!({"success": false, "message": "unauthorized"}).to_string());
                let (http_req, _) = req.into_parts();
                let res = ServiceResponse::new(http_req, http_res);
                return (async move { Ok(res.map_into_right_body()) }).boxed_local();
            }
        };

        req.extensions_mut().insert(claims);
        let fut = self.service.call(req);

        Box::pin(async move {
            let res = fut.await?;
            Ok(res.map_into_left_body())
        })
    }
}
