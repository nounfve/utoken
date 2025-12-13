use std::str::FromStr;

use axum::http::HeaderValue;
use reqwest::{Request, Url, header::AUTHORIZATION};
use tracing::info;

use crate::token::AuthToken;

pub struct Client {
    inner: reqwest::Client,
    pub endpoint: String,
}

impl Default for Client {
    fn default() -> Self {
        Self {
            inner: reqwest::Client::new(),
            endpoint: format!("http://localhost:6201"),
        }
    }
}

impl Client {
    pub async fn create_token(&self, claim: String) -> anyhow::Result<AuthToken> {
        let url = format!("{}/token/create", &self.endpoint);
        let url = Url::from_str(&url)?;
        let resp = self
            .inner
            .put(url)
            .body(claim)
            .send()
            .await?
            .error_for_status()?;
        println!("{resp:?}");
        let json = resp.json::<AuthToken>().await?;
        Ok(json)
    }

    pub async fn auth_request(&self, mut req: Request, token: &AuthToken) -> anyhow::Result<()> {
        let token = HeaderValue::from_str(&format!("Bearer {}", &token.access.content))?;
        req.headers_mut().insert(AUTHORIZATION, token);
        let resp = self.inner.execute(req).await?.error_for_status()?;
        info!("{}", resp.text().await?);
        Ok(())
    }
}
