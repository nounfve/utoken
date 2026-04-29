pub fn login_route() -> Router {
    Router::new()
        .nest("/steam", steam_route())
        .route("/.ui./", get(bridge_dist_slash))
        .route("/.ui./{*path}", get(bridge_dist))
        .fallback(async || Redirect::to("/login/.ui./"))
}

async fn bridge_dist_slash() -> Response {
    bridge_dist(Path(String::new())).await
}

async fn bridge_dist(Path(path): Path<String>) -> Response {
    let path = try_file_vite_build(&path);
    let Some(file) = UI_DIST.get_file(path.as_ref()) else {
        RIP!(StatusCode::NOT_FOUND)
    };
    let mime = mime_guess::from_path(&*path).first_or_octet_stream();
    RIP!([(header::CONTENT_TYPE, mime.as_ref())], file.contents())
}

fn try_file_vite_build(path: &str) -> Cow<'_, str> {
    let element = path
        .rsplit("/")
        .skip_while(|str| str.is_empty())
        .take(2)
        .collect::<Vec<_>>();

    let index = "index.html";
    let file = match element.len() {
        0 => index.INTO::<Cow<'_, _>>(),
        1 => path.INTO::<Cow<'_, _>>(),
        _ => format!("{}/{}", element[1], element[0]).INTO::<Cow<'_, _>>(),
    };
    if UI_DIST.contains(&*file) {
        return file;
    }
    index.into()
}

static UI_DIST: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/src/.ui./");

use std::borrow::Cow;

use axum::{
    Router,
    extract::Path,
    response::{Redirect, Response},
    routing::get,
};
use include_dir::{Dir, include_dir};
use reqwest::{StatusCode, header};
use sutils::{TypedInto, boilerplates::RIP};

use crate::oauth_steam::steam_route;
