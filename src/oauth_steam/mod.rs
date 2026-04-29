mod steal;
mod verify;

use axum::{Router, routing::get};

use crate::oauth_steam::verify::verify_steam;

pub fn steam_route() -> Router {
    Router::new().route("/verify", get(verify_steam))
}
