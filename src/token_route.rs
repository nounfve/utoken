pub fn token_route() -> Router {
    Router::new()
        .route("/create", put(token_create))
        .route("/refresh", put(token_refresh))
        .route("/info", get(token_info))
        .route("/delete", delete(token_delete))
        .route("/subtoken", put(sub_token_create))
}

async fn token_create(ConnectInfo(addr): ConnectInfo<SocketAddr>, claim: String) -> Response {
    if addr.ip() != LOCALHOST {
        RIP!(StatusCode::UNAUTHORIZED, "not allowed");
    }

    info!("token_claim: {claim}");
    let claim = match Claim::from_str(&claim) {
        Ok(claim) => claim,
        Err(err) => {
            error!("{err}");
            RIP!(StatusCode::BAD_REQUEST, "payload not a valid uri");
        }
    };

    let mut auth = match AuthToken::sql_insert_token(claim, None).await {
        Ok(val) => val,
        Err(err) => {
            error!("{err}");
            RIP!(StatusCode::INTERNAL_SERVER_ERROR, "database raise error");
        }
    };

    auth.claim = auth.claim.scope_only();
    RIP!(
        StatusCode::CREATED,
        [(AuthToken::HEAD_X_SCOPE, auth.claim.parse_scope_name())],
        auth.to_json()
    )
}

async fn token_refresh(token: String) -> Response {
    info!("refresh token: {token}");
    let Ok(refresh) = Uuid::from_str(&token) else {
        RIP!(StatusCode::BAD_REQUEST, "invalid refresh token");
    };

    let mut auth = match AuthToken::sql_refresh_token(&refresh).await {
        Ok(auth) => auth,
        Err(err) => {
            error!("{err}");
            RIP!(StatusCode::BAD_REQUEST, "token not exists");
        }
    };

    auth.claim = auth.claim.scope_only();
    RIP!(StatusCode::OK, auth.to_json())
}

async fn token_delete(bearer: OptionBearer) -> Result<Response, ErrorResponse> {
    let bearer = must_bearer(bearer).await?;
    match AuthToken::sql_delete_token(&bearer).await {
        Ok(_) => (),
        Err(err) => {
            warn!("[maybe error delete failed] {err}");
        }
    };
    (StatusCode::NO_CONTENT).into_response().Ok()
}

pub async fn token_info(
    bearer: OptionBearer,
    refresh: Q_refresh,
) -> Result<Response, ErrorResponse> {
    let bearer = must_bearer(bearer).await?;

    let auth = match AuthToken::sql_find_access_token(&bearer).await {
        Ok(auth) => auth,
        Err(err) => {
            error!("{err}");
            return (StatusCode::UNAUTHORIZED, "invlid token").Err();
        }
    };

    if let Some(refresh) = &*refresh
        && auth.access.expire - Utc::now() < AuthToken::AUTO_REFRESH
    {
        return token_refresh(refresh.clone()).await.Ok();
    }

    if auth.access.expire < Utc::now() {
        return (StatusCode::UNAUTHORIZED, "expired token").Err();
    }
    let scope_name = auth.claim.parse_scope_name();

    (
        StatusCode::OK,
        [(AuthToken::HEAD_X_SCOPE, scope_name)],
        json!({"claim":scope_name}).to_string(),
    )
        .into_response()
        .Ok()
}

async fn sub_token_create(bearer: OptionBearer, claim: String) -> Result<Response, ErrorResponse> {
    let bearer = must_bearer(bearer).await?;

    let auth = match AuthToken::sql_find_access_token(&bearer).await {
        Ok(auth) => auth,
        Err(err) => {
            error!("{err}");
            return (StatusCode::UNAUTHORIZED, "invlid token").Err();
        }
    };

    if auth.access.expire < Utc::now() {
        return (StatusCode::UNAUTHORIZED, "expired token").Err();
    }

    let mut sub = create_sub_token(&auth, &claim).await?;

    sub.claim = sub.claim.scope_only();
    (StatusCode::CREATED, sub.to_json()).into_response().Ok()
}

const LOCALHOST: Ipv4Addr = Ipv4Addr::new(127, 0, 0, 1);

use std::{
    net::{Ipv4Addr, SocketAddr},
    str::FromStr,
};

use axum::{
    Router,
    extract::ConnectInfo,
    response::{IntoResponse, Response},
    routing::{delete, get, put},
};
use chrono::Utc;
use reqwest::StatusCode;
use serde_json::json;
use sutils::{IntoResult, boilerplates::RIP};
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::{
    axum_extract::{OptionBearer, Q_refresh},
    reuse_handler::{ErrorResponse, create_sub_token, must_bearer},
    token::{AuthToken, Claim},
};
