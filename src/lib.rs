pub mod database;
pub mod token;

use axum::{
    Router,
    extract::Path,
    http::StatusCode,
    routing::{get, put},
};
use chrono::Utc;
use sutils::tracing_setup;
use tracing::{error, info};

use crate::{
    database::DataBase,
    token::{Authorization, Claim},
};

pub async fn _main() {
    tracing_setup();

    DataBase::init().await.expect("database conn failed");

    let app = Router::new()
        .route("/token/create", put(token_create))
        .route("/auth/{*path}", get(get_all))
        .fallback(health);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:6201").await.unwrap();
    info!("listen on: {listener:?}");
    axum::serve(listener, app).await.unwrap();
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

    let auth = Authorization::new(claim);
    info!("auth: {auth:?}");

    match auth.sql_insert_token().await {
        Ok(_) => (),
        Err(err) => {
            error!("{err}");
            return (StatusCode::BAD_REQUEST, format!("database raise error"));
        }
    }

    (
        StatusCode::CREATED, //
        format!("ok"),
    )
}

async fn get_all(Path(path): Path<String>) -> String {
    info!("{path:?}");
    format!("hello, {path}")
}

async fn health() -> String {
    let time = Utc::now().to_string();
    format!("Ok @[{time}]")
}
