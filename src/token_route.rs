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

async fn token_delete(bearer: OptionBearer) -> Response {
    check_bearer_is_some!(bearer);
    match AuthToken::sql_delete_token(&bearer).await {
        Ok(_) => (),
        Err(err) => {
            warn!("[maybe error delete failed] {err}");
        }
    };
    RIP!(StatusCode::NO_CONTENT)
}

pub async fn token_info(bearer: OptionBearer, refresh: Q_refresh) -> Response {
    check_bearer_is_some!(bearer);

    let auth = match AuthToken::sql_find_access_token(&bearer).await {
        Ok(auth) => auth,
        Err(err) => {
            error!("{err}");
            RIP!(StatusCode::UNAUTHORIZED, "invlid token");
        }
    };

    if let Some(refresh) = &*refresh
        && auth.access.expire - Utc::now() < AuthToken::AUTO_REFRESH
    {
        return token_refresh(refresh.clone()).await;
    }

    if auth.access.expire < Utc::now() {
        RIP!(StatusCode::UNAUTHORIZED, "expired token")
    }
    let scope_name = auth.claim.parse_scope_name();
    RIP!(
        StatusCode::OK,
        [(AuthToken::HEAD_X_SCOPE, scope_name)],
        json!({"claim":scope_name}).to_string()
    )
}

async fn sub_token_create(bearer: OptionBearer, claim: String) -> Response {
    check_bearer_is_some!(bearer);

    let auth = match AuthToken::sql_find_access_token(&bearer).await {
        Ok(auth) => auth,
        Err(err) => {
            error!("{err}");
            RIP!(StatusCode::UNAUTHORIZED, "invlid token");
        }
    };

    if auth.access.expire < Utc::now() {
        RIP!(StatusCode::UNAUTHORIZED, "expired token")
    }

    let mut sub = match create_sub_token(&auth, &claim).await {
        Ok(val) => val,
        Err(err) => RIP!(err),
    };

    sub.claim = sub.claim.scope_only();
    RIP!(StatusCode::CREATED, sub.to_json())
}

#[PutInMacro(inline_macro)]
macro_rules! check_bearer_is_some {
    ($B:ident) => {
        let Some($B) = &*$B else {
            RIP!(StatusCode::UNAUTHORIZED, "missing bearer header")
        };

        let Ok($B) = <uuid::Uuid as std::str::FromStr>::from_str($B) else {
            RIP!(StatusCode::UNAUTHORIZED, "invalid uuid token")
        };
    };
}

const LOCALHOST: Ipv4Addr = Ipv4Addr::new(127, 0, 0, 1);

use std::{
    net::{Ipv4Addr, SocketAddr},
    str::FromStr,
};

use axum::{
    Router,
    extract::ConnectInfo,
    response::Response,
    routing::{delete, get, put},
};
use chrono::Utc;
use reqwest::StatusCode;
use serde_json::json;
use sutils::{PutInMacro, boilerplates::RIP, inline_macro};
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::{
    axum_extract::{OptionBearer, Q_refresh},
    reuse_handler::create_sub_token,
    token::{AuthToken, Claim},
};
