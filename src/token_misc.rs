use std::{
    net::{Ipv4Addr, SocketAddr, SocketAddrV4},
    time::Duration,
};

use axum::{Router, extract::ConnectInfo, routing::put};
use cell_reg::cell_reg_named::StaticRefSingleton as _;
use reqwest::StatusCode;
use tokio::time::sleep;
use tracing::{error, info};

use crate::{
    database::DataBase,
    token::{AuthToken, Claim},
};

const LOCALHOST: Ipv4Addr = Ipv4Addr::new(127, 0, 0, 1);
const LOCALHOST_SOCKET: SocketAddr = SocketAddr::V4(SocketAddrV4::new(LOCALHOST, 0));

pub fn token_route() -> Router {
    Router::new()
        .route("/create", put(token_create))
        .route("/refresh", put(token_refresh))
}

pub async fn token_create(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    claim: String,
) -> (StatusCode, String) {
    if addr.ip() != LOCALHOST {
        return (StatusCode::UNAUTHORIZED, format!("not allowed"));
    }

    info!("token_claim: {claim}");
    let claim = match Claim::from_str(&claim) {
        Ok(claim) => claim,
        Err(err) => {
            error!("{err}");
            return (StatusCode::BAD_REQUEST, format!("payload not a valid uri"));
        }
    };

    let auth = AuthToken::new(claim);
    info!("auth: {auth:?}");

    match auth.sql_insert_token().await {
        Ok(_) => (),
        Err(err) => {
            error!("{err}");
            return (StatusCode::BAD_REQUEST, format!("database raise error"));
        }
    }

    let auth = match serde_json::to_string(&auth) {
        Ok(s) => s,
        Err(err) => {
            error!("{err}");
            return (StatusCode::BAD_REQUEST, format!("serde json failed"));
        }
    };

    (StatusCode::CREATED, auth)
}

pub async fn token_refresh(token: String) -> (StatusCode, String) {
    info!("refresh token: {token}");
    let auth = match AuthToken::sql_find_refresh_token(&token).await {
        Ok(auth) => auth,
        Err(err) => {
            error!("{err}");
            return (StatusCode::BAD_REQUEST, format!("token not exists"));
        }
    };

    match auth.sql_delete_token().await {
        Ok(()) => (),
        Err(err) => {
            error!("{err}");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("token deletion failed"),
            );
        }
    }

    token_create(ConnectInfo(LOCALHOST_SOCKET), auth.claim.inner.to_string()).await
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
