use std::{
    net::{Ipv4Addr, SocketAddr, SocketAddrV4},
    time::Duration,
};

use axum::{
    Router,
    extract::{ConnectInfo, Query},
    response::Response,
    routing::{get, put},
};
use axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Bearer},
};
use cell_reg::cell_reg_named::StaticRefSingleton as _;
use chrono::Utc;
use http_body_util::BodyExt;
use reqwest::StatusCode;
use tokio::time::sleep;
use tracing::{error, info};

use crate::{
    RIP,
    database::DataBase,
    token::{AuthToken, Claim},
};

const LOCALHOST: Ipv4Addr = Ipv4Addr::new(127, 0, 0, 1);
const LOCALHOST_SOCKET: SocketAddr = SocketAddr::V4(SocketAddrV4::new(LOCALHOST, 0));

pub fn token_route() -> Router {
    Router::new()
        .route("/create", put(token_create))
        .route("/refresh", put(token_refresh))
        .route("/info", get(token_info))
}

pub async fn token_create(ConnectInfo(addr): ConnectInfo<SocketAddr>, claim: String) -> Response {
    if addr.ip() != LOCALHOST {
        RIP!(StatusCode::UNAUTHORIZED, format!("not allowed"));
    }

    info!("token_claim: {claim}");
    let claim = match Claim::from_str(&claim) {
        Ok(claim) => claim,
        Err(err) => {
            error!("{err}");
            RIP!(StatusCode::BAD_REQUEST, format!("payload not a valid uri"));
        }
    };

    let mut auth = AuthToken::new(claim);
    info!("auth: {auth:?}");

    match auth.sql_insert_token().await {
        Ok(_) => (),
        Err(err) => {
            error!("{err}");
            RIP!(StatusCode::BAD_REQUEST, format!("database raise error"));
        }
    }

    auth.claim = auth.claim.scope_only();
    let auth_json = serde_json::to_string(&auth).unwrap();

    RIP!(StatusCode::CREATED, auth_json)
}

pub async fn token_refresh(token: String) -> Response {
    info!("refresh token: {token}");
    let auth = match AuthToken::sql_find_refresh_token(&token).await {
        Ok(auth) => auth,
        Err(err) => {
            error!("{err}");
            RIP!(StatusCode::BAD_REQUEST, format!("token not exists"));
        }
    };

    match auth.sql_delete_token().await {
        Ok(()) => (),
        Err(err) => {
            error!("{err}");
            RIP!(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("token deletion failed")
            );
        }
    }

    token_create(ConnectInfo(LOCALHOST_SOCKET), auth.claim.inner.to_string()).await
}

#[derive(serde::Deserialize, Debug)]
pub struct RefreshQuery {
    pub refresh: Option<String>,
}

pub async fn token_info(
    TypedHeader(Authorization(bearer)): TypedHeader<Authorization<Bearer>>,
    Query(RefreshQuery { refresh }): Query<RefreshQuery>,
) -> Response {
    let mut auth = match AuthToken::sql_find_access_token(bearer.token()).await {
        Ok(auth) => auth,
        Err(err) => {
            error!("{err}");
            RIP!(StatusCode::UNAUTHORIZED, format!("invlid token"));
        }
    };

    let access_remain_sec = (auth.access.expire - Utc::now()).as_seconds_f64() as i64;
    if let Some(refresh) = refresh {
        if access_remain_sec < AuthToken::ACCESS_EXPIRE / 1000 / 8 {
            let resp = token_refresh(refresh).await;
            if !resp.status().is_success() {
                return resp;
            }
            let body = resp.into_body().collect().await.unwrap().to_bytes();
            auth = serde_json::from_slice(&body).unwrap();
        }
    }

    let access_remain_sec = (auth.access.expire - Utc::now()).as_seconds_f64() as i64;
    if access_remain_sec <= 0 {
        RIP!(StatusCode::UNAUTHORIZED, format!("expired token"))
    }

    let auth_json = serde_json::to_string(&auth).unwrap();

    RIP!(StatusCode::OK, auth_json)
}

pub async fn clean_outdated_token() {
    let db = DataBase::One();
    loop {
        if let Err(err) = sqlx::query(
            "DELETE FROM utokens
            WHERE refresh_expire < NOW()",
        )
        .execute(&db.conn)
        .await
        {
            error!("{err}");
            sleep(Duration::from_secs(60)).await;
            continue;
        }
        sleep(Duration::from_secs(60 * 60 * 22)).await
    }
}
