pub mod client;
pub mod database;
pub mod token;
pub mod token_misc;

use axum::{
    Router,
    extract::Path,
    http::StatusCode,
    routing::{get, put},
};
use axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Bearer},
};
use chrono::Utc;
use glob::Pattern;
use sutils::tracing_setup;
use tracing::{error, info, warn};

use crate::{
    database::DataBase,
    token::{AuthToken, Claim},
    token_misc::clean_outdated_token,
};

pub async fn _main() {
    tracing_setup();

    DataBase::init().await.expect("database conn failed");
    spwan_periodic_tasks().await;

    let app = Router::new()
        .route("/token/create", put(token_create))
        .route("/auth/{*path}", get(get_all))
        .fallback(health);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:6201").await.unwrap();
    info!("listen on: {listener:?}");
    axum::serve(listener, app).await.unwrap();
}

async fn spwan_periodic_tasks() {
    tokio::spawn(clean_outdated_token());
}

async fn token_create(body: String) -> (StatusCode, String) {
    info!("token_claim: {body}");
    let claim = match Claim::from_str(&body) {
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

async fn get_all(
    Path(path): Path<String>,
    TypedHeader(Authorization(bearer)): TypedHeader<Authorization<Bearer>>,
) -> (StatusCode, String) {
    let path = format!("/{path}");
    info!("{path:?},{bearer:?}");
    let auth = match AuthToken::sql_find_access_token(bearer.token()).await {
        Ok(auth) => auth,
        Err(err) => {
            error!("{err}");
            return (StatusCode::UNAUTHORIZED, format!("token not exists"));
        }
    };
    
    if auth.access.expire < Utc::now() {
        warn!("token expired");
        return (StatusCode::UNAUTHORIZED, format!("token expired"));
    };

    let pattern = Pattern::new(auth.claim.inner.path()).unwrap();
    if !pattern.matches(&path) {
        warn!("claim not matched {pattern}");
        return (
            StatusCode::UNAUTHORIZED,
            format!("token not valid for this use"),
        );
    }

    (StatusCode::OK, format!("hello, {path}"))
}

async fn health() -> String {
    let time = Utc::now().to_string();
    format!("Ok @[{time}]")
}
