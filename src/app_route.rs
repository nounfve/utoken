pub fn app_route() -> Router {
    Router::new()
        .route("/auth/{*path}", any(handle_auth_path))
        .route("/bind/{*path}", any(bind_link))
        .route("/link/.self/list", get(list_links))
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
) -> Result<impl IntoResponse, ErrorResponse> {
    info!("{method},{path},{bearer:?}");
    let bearer = must_bearer(bearer).await?;

    let auth = handle_auth_path_reuse(method, path, bearer).await?;

    (
        StatusCode::OK,
        [(AuthToken::HEAD_X_SCOPE, auth.claim.parse_scope_name())],
    )
        .into_response()
        .Ok()
}

async fn bind_link(
    method: http::Method,
    Path(path): Path<String>,
    RawQuery(query): RawQuery,
    bearer: OptionBearer,
) -> Result<impl IntoResponse, ErrorResponse> {
    info!("{path},{query:?},{bearer:?}");
    let bearer = must_bearer(bearer).await?;

    let auth = handle_auth_path_reuse(method, path.clone(), bearer).await?;

    let path = format!("/{path}");
    let query = query.map(|q| format!("?{q}"));

    let sub = create_sub_token(&auth, &path).await?;

    let link = LinkBind { path, query, tokens: None };
    let link = match link.sql_insert_link(&sub.refresh.content).await {
        Ok(val) => val,
        Err(err) => {
            error! {"{err}"};
            return (StatusCode::INTERNAL_SERVER_ERROR, "").Err();
        }
    };

    (link.simple().to_string()).Ok()
}

async fn resolve_link(
    Path(link): Path<String>,
    RawQuery(rq): RawQuery,
) -> Result<impl IntoResponse, ErrorResponse> {
    let (link, rest) = match link.split_once("/") {
        Some(split) => split,
        _ => (link.as_str(), ""),
    };
    let Ok(link) = Uuid::from_str(&link) else {
        return (StatusCode::BAD_REQUEST, "not valid link value").Err();
    };

    let Ok(LinkBind {
        mut path,
        mut query,
        tokens: Some((access, refresh)),
    }) = LinkBind::sql_retrive_link(&link).await
    else {
        return (StatusCode::UNAUTHORIZED, "non exists link").Err();
    };

    let info_resp = token_info(
        OptionBearer(access.to_string().Some()),
        Q_refresh(refresh.to_string().Some()),
    )
    .await?
    .into_response();

    let info_header = match info_resp.status().is_success() {
        false => return (StatusCode::UNAUTHORIZED, "token expired").Err(),
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
    (StatusCode::NO_CONTENT, link.set_headers(info_header)).Ok()
}

async fn list_links(bearer: OptionBearer) -> Result<impl IntoResponse, ErrorResponse> {
    let bearer = must_bearer(bearer).await?;
    let auth = verify_bearer_as_access(bearer).await?;
    let links = match LinkBind::sql_list_links(&auth.refresh.content).await {
        Ok(val) => val,
        Err(err) => {
            error!("{err}");
            return (StatusCode::INTERNAL_SERVER_ERROR, "").Err();
        }
    };
    (axum::Json(links)).Ok()
}

use std::str::FromStr;

use axum::{
    Router,
    extract::{Path, RawQuery},
    http::{self},
    response::IntoResponse,
    routing::{any, get},
};
use reqwest::StatusCode;
use sutils::{
    IntoOption, IntoResult,
    boilerplates::{health, not_found},
};
use tracing::{error, info};
use uuid::Uuid;

use crate::{
    account::account_route,
    axum_extract::{OptionBearer, Q_refresh},
    link_bind::LinkBind,
    reuse_handler::{
        ErrorResponse, create_sub_token, handle_auth_path_reuse, must_bearer,
        verify_bearer_as_access,
    },
    token::AuthToken,
    token_route::{token_info, token_route},
};
