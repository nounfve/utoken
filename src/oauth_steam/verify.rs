use anyhow::anyhow;
use axum::{
    extract::Query,
    response::{Redirect, Response},
};
use percent_encoding::{NON_ALPHANUMERIC, utf8_percent_encode};
use reqwest::StatusCode;
use sutils::{Singleton, boilerplates::RIP};
use tracing::{error, info, warn};

use crate::{client::Client, oauth_steam::steal::VerifyForm, token::AuthToken};

pub async fn verify_steam(
    Query(mut verify): Query<VerifyForm>,
    Query(OnSuccessQuery {
        on_success: return_to,
    }): Query<OnSuccessQuery>,
) -> Response {
    info!("{verify:?}");
    verify.mode = format!("check_authentication");
    let resp = match reqwest::Client::new()
        .get("https://steamcommunity.com/openid/login")
        .query(&verify)
        .send()
        .await
    {
        Ok(resp) => resp,
        Err(err) => {
            error!("{err}");
            RIP!(StatusCode::NOT_FOUND, format!("steam connection error"));
        }
    };

    let resp = match resp.error_for_status() {
        Ok(resp) => resp,
        Err(err) => {
            error!("{err}");
            RIP!(StatusCode::UNAUTHORIZED, format!("invliad auth request"));
        }
    };

    let resp = match resp.text().await {
        Ok(resp) => resp,
        Err(err) => {
            error!("{err}");
            RIP!(
                StatusCode::NOT_FOUND,
                format!("steam connection interupted")
            );
        }
    };

    let is_valid = resp.split('\n').any(|line| line == "is_valid:true");
    if !is_valid {
        warn!("[rejected] {resp:?}");
        RIP!(StatusCode::UNAUTHORIZED, format!("rejected by steam"));
    }

    let auth_json = match get_utoken_from_claim(verify.claimed_id).await {
        Ok(token) => serde_json::to_string(&token).unwrap(),
        Err(err) => {
            error!("{err}");
            RIP!(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("failed to issue token")
            );
        }
    };

    let url = return_to.unwrap_or(format!("/"));
    let auth_token = utf8_percent_encode(&auth_json, NON_ALPHANUMERIC).collect::<String>();
    let url = if url.find("?").is_some() {
        format!("{url}&token={auth_token}")
    } else {
        format!("{url}?token={auth_token}")
    };

    RIP!(Redirect::to(&url.to_string()))
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct OnSuccessQuery {
    on_success: Option<String>,
}

async fn get_utoken_from_claim(claim: String) -> anyhow::Result<AuthToken> {
    let steam_id = match claim.strip_prefix("https://steamcommunity.com/openid/id/") {
        Some(id) => id,
        None => Err(anyhow!("unrecognized steam id"))?,
    };

    let steam_id = steam_id.parse::<i64>()?;

    let utoken_claim = format!("u://+delete+put+patch@{steam_id}.steam_token/**");
    let token = Client::One().create_token(utoken_claim).await?;
    Ok(token)
}
