mod database;

pub mod client;
pub mod token;
pub mod token_misc;
mod conversion;


use std::net::SocketAddr;

use axum::{
    Router, extract::Path,
    http::{self, StatusCode},
    routing::{any, get},
};
use axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Bearer},
};
use chrono::Utc;
use sutils::tracing_setup;
use tracing::{error, info, warn};

use crate::{
    database::DataBase,
    token::AuthToken,
    token_misc::{clean_outdated_token, token_route},
};

pub async fn _main() {
    tracing_setup();

    DataBase::init().await.expect("database conn failed");
    spwan_periodic_tasks().await;

    let app = Router::new()
        .nest("/token", token_route())
        .route("/auth/{*path}", any(handle_auth_path))
        .route("/health", get(health))
        .fallback(health);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:6201").await.unwrap();
    info!("listen on: {listener:?}");
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}

async fn spwan_periodic_tasks() {
    tokio::spawn(clean_outdated_token());
}

async fn handle_auth_path(
    method: http::Method,
    Path(path): Path<String>,
    TypedHeader(Authorization(bearer)): TypedHeader<Authorization<Bearer>>,
) -> (StatusCode, String) {
    let path = format!("/{path}");
    info!("{method},{path},{bearer:?}");
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

    if !auth.claim.match_path(&path) || !auth.claim.match_method(method.as_str()) {
        warn!("claim not matched {}", auth.claim.inner.to_string());
        return (
            StatusCode::UNAUTHORIZED,
            format!("token not valid for this use"),
        );
    }

    (StatusCode::OK, String::new())
}

pub async fn health() -> String {
    let time = Utc::now().to_string();
    format!("Ok @[{time}]")
}
