pub fn app_route() -> Router {
    Router::new()
        .route("/auth/{*path}", any(handle_auth_path))
        .route("/bind/{*path}", any(bind_link))
        .route("/link/{*link}", any(resolve_link))
        .nest("/token", token_route())
        .nest("/@me", account_route())
        .route("/health", get(health!(^async)))
        .fallback(not_found!(^async))
}

async fn handle_auth_path(
    method: http::Method,
    Path(path): Path<String>,
    bearer: OptionBearer,
) -> Response {
    info!("{method},{path},{bearer:?}");
    check_bearer_is_some!(bearer);

    let auth = match handle_auth_path_reuse(method, path, bearer).await {
        Ok(val) => val,
        Err(err) => RIP!(err),
    };

    RIP!(
        StatusCode::OK,
        [(AuthToken::HEAD_X_SCOPE, auth.claim.parse_scope_name())]
    )
}

async fn bind_link(
    method: http::Method,
    Path(path): Path<String>,
    RawQuery(query): RawQuery,
    bearer: OptionBearer,
) -> Response {
    info!("{path},{query:?},{bearer:?}");
    check_bearer_is_some!(bearer);

    let auth = match handle_auth_path_reuse(method, path.clone(), bearer).await {
        Ok(val) => val,
        Err(err) => RIP!(err),
    };

    let path = format!("/{path}");
    let query = query.map(|q| format!("?{q}"));

    let sub = match create_sub_token(&auth, &path).await {
        Ok(val) => val,
        Err(err) => RIP!(err),
    };

    let link = LinkBind { path, query, tokens: None };
    let link = match link.sql_insert_link(&sub.refresh.content).await {
        Ok(val) => val,
        Err(err) => {
            error! {"{err}"};
            RIP!(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    RIP!(link.simple().to_string())
}

async fn resolve_link(Path(link): Path<String>, RawQuery(rq): RawQuery) -> Response {
    let (link, rest) = match link.split_once("/") {
        Some(split) => split,
        _ => (link.as_str(), ""),
    };
    let Ok(link) = Uuid::from_str(&link) else {
        RIP!(StatusCode::BAD_REQUEST, "not valid link value")
    };

    let Ok(LinkBind {
        mut path,
        mut query,
        tokens: Some((access, refresh)),
    }) = LinkBind::sql_retrive_link(&link).await
    else {
        RIP!(StatusCode::UNAUTHORIZED, "non exists link")
    };

    let info_resp = token_info(
        OptionBearer(access.to_string().Some()),
        Q_refresh(refresh.to_string().Some()),
    )
    .await;
    let info_header = match info_resp.status().is_success() {
        false => RIP!(StatusCode::UNAUTHORIZED, "token expired"),
        true => info_resp.headers().clone(),
    };

    if !rest.is_empty() {
        path = format!("{path}/{rest}")
    }
    if let Some(rq) = rq {
        query = match query {
            Some(val) => format!("{val}&{rq}"),
            None => format!("?{rq}"),
        }
        .Some()
    }

    let link = LinkBind { path, query, tokens: None };
    RIP!(StatusCode::NO_CONTENT, link.set_headers(info_header))
}

use std::str::FromStr;

use axum::{
    Router,
    extract::{Path, RawQuery},
    http::{self},
    response::Response,
    routing::{any, get},
};
use reqwest::StatusCode;
use sutils::{
    IntoOption,
    boilerplates::{RIP, health, not_found},
};
use tracing::{error, info};
use uuid::Uuid;

use crate::{
    account::account_route,
    axum_extract::{OptionBearer, Q_refresh},
    link_bind::LinkBind,
    reuse_handler::{create_sub_token, handle_auth_path_reuse},
    token::AuthToken,
    token_route::{check_bearer_is_some, token_info, token_route},
};
