pub fn token_route() -> Router {
    Router::new()
        .route("/create", put(token_create))
        .route("/refresh", put(token_refresh))
        .route("/info", get(token_info))
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
    info!("auth: {auth:?}");

    auth.claim = auth.claim.scope_only();
    RIP!(StatusCode::CREATED, auth.to_json())
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

async fn token_info(bearer: OptionBearer, refresh: Q_refresh) -> Response {
    let Some(bearer) = &*bearer else {
        RIP!(StatusCode::UNAUTHORIZED, "missing bearer header")
    };

    let Ok(access) = Uuid::from_str(bearer.token()) else {
        RIP!(StatusCode::UNAUTHORIZED, "invalid uuid token")
    };

    let mut auth = match AuthToken::sql_find_access_token(&access).await {
        Ok(auth) => auth,
        Err(err) => {
            error!("{err}");
            RIP!(StatusCode::UNAUTHORIZED, "invlid token");
        }
    };

    if let Some(refresh) = &*refresh
        && auth.access.expire - Utc::now() < AuthToken::ACCESS_EXPIRE / 1000 / 8
    {
        let resp = token_refresh(refresh.clone()).await;
        if !resp.status().is_success() {
            return resp;
        }
        let body = resp.into_body().collect().await.unwrap().to_bytes();
        auth = serde_json::from_slice(&body).unwrap();
    } else {
        auth.claim = auth.claim.scope_only()
    }

    if auth.access.expire < Utc::now() {
        RIP!(StatusCode::UNAUTHORIZED, "expired token")
    }

    RIP!(StatusCode::OK, auth.to_json())
}

pub async fn clean_outdated_token() {
    loop {
        if let Err(err) = sqlx::query(
            "DELETE FROM utokens
            WHERE refresh_expire < NOW()",
        )
        .execute(&DataBase::One().conn)
        .await
        {
            error!("{err}");
            sleep(Duration::from_secs(60)).await;
            continue;
        }
        sleep(Duration::from_secs(60 * 60 * 22)).await
    }
}

const LOCALHOST: Ipv4Addr = Ipv4Addr::new(127, 0, 0, 1);

use std::{
    net::{Ipv4Addr, SocketAddr},
    str::FromStr,
    time::Duration,
};

use axum::{
    Router,
    extract::ConnectInfo,
    response::Response,
    routing::{get, put},
};
use chrono::Utc;
use http_body_util::BodyExt;
use reqwest::StatusCode;
use sutils::{Singleton, boilerplates::RIP};
use tokio::time::sleep;
use tracing::{error, info};
use uuid::Uuid;

use crate::{
    axum_extract::{OptionBearer, Q_refresh},
    database::DataBase,
    token::{AuthToken, Claim},
};
