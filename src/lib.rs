mod database;
mod test;

pub mod account;
pub mod app_route;
pub mod axum_extract;
pub mod client;
pub mod conversion;
pub mod link_bind;
pub mod oauth_steam;
pub mod reuse_handler;
pub mod token;
pub mod token_route;

pub async fn _main() {
    tracing_env_or_info!();

    DataBase::init().await.expect("database conn failed");
    spwan_periodic_tasks().await;

    let listener = tokio::net::TcpListener::bind("0.0.0.0:6201").await.unwrap();
    info!("listen on: {listener:?}");

    axum::serve(
        listener,
        app_route().into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}

async fn spwan_periodic_tasks() {
    tokio::spawn(AuthToken::clean_outdated_token());
}

use std::net::SocketAddr;

use sutils::boilerplates::tracing_env_or_info;
use tracing::info;

use crate::{app_route::app_route, database::DataBase, token::AuthToken};
