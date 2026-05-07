pub type ErrorResponse = (StatusCode, &'static str);

pub async fn must_bearer(bearer: OptionBearer) -> Result<Uuid, ErrorResponse> {
    let Some(bearer) = &*bearer else {
        return (StatusCode::UNAUTHORIZED, "missing bearer header").Err();
    };
    let Ok(bearer) = <uuid::Uuid as std::str::FromStr>::from_str(bearer) else {
        return (StatusCode::UNAUTHORIZED, "invalid uuid token").Err();
    };
    bearer.Ok()
}

pub async fn verify_bearer_as_access(bearer: Uuid) -> Result<AuthToken, ErrorResponse> {
    let auth = match AuthToken::sql_find_access_token(&bearer).await {
        Ok(auth) => auth,
        Err(err) => {
            warn!("{err}");
            return (StatusCode::UNAUTHORIZED, "token not exists").Err();
        }
    };

    if auth.access.expire < Utc::now() {
        warn!("token expired");
        return (StatusCode::UNAUTHORIZED, "token expired").Err();
    };
    auth.Ok()
}

pub async fn handle_auth_path_reuse(
    method: http::Method,
    path: String,
    bearer: Uuid,
) -> Result<AuthToken, ErrorResponse> {
    info!("{method},{path},{bearer:?}");
    let auth = verify_bearer_as_access(bearer).await?;

    if !auth.claim.match_path(&format!("/{path}")) || !auth.claim.match_method(method.as_str()) {
        warn!("claim not matched {}", auth.claim.inner.to_string());
        return (StatusCode::UNAUTHORIZED, "token not valid for this use").Err();
    };
    auth.Ok()
}

pub async fn create_sub_token(auth: &AuthToken, claim: &str) -> Result<AuthToken, ErrorResponse> {
    info!("token_claim: {claim}");
    let claim = match auth.claim.sub_claim(&claim) {
        Ok(claim) => claim,
        Err(err) => {
            error!("{err}");
            return (StatusCode::BAD_REQUEST, "payload not a valid child claim").Err();
        }
    };

    let sub = match AuthToken::sql_insert_token(claim, (&auth.refresh.content).Some()).await {
        Ok(val) => val,
        Err(err) => {
            error!("{err}");
            return (StatusCode::INTERNAL_SERVER_ERROR, "database raise error").Err();
        }
    };

    sub.Ok()
}

use axum::http;
use chrono::Utc;
use reqwest::StatusCode;
use sutils::{IntoOption, IntoResult};
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::{axum_extract::OptionBearer, token::AuthToken};
