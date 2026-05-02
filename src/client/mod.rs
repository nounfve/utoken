mod test;

use reqwest::{Request, StatusCode, header::AUTHORIZATION};
use sutils::{ContextFunction, IntoOption, IntoResult, Singleton};
use tracing::info;

use crate::token::AuthToken;

#[Singleton]
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
        let resp = self
            .inner
            .put(&url)
            .body(claim)
            .send()
            .await?
            .error_for_status()?;
        let auth_json = resp.json().await?;
        Ok(auth_json)
    }

    pub async fn refresh_token(&self, token: &AuthToken) -> anyhow::Result<AuthToken> {
        let url = format!("{}/token/refresh", &self.endpoint);
        let resp = self
            .inner
            .put(&url)
            .body(token.refresh.content.to_string())
            .send()
            .await?
            .error_for_status()?;
        let auth_json = resp.json().await?;
        Ok(auth_json)
    }

    pub async fn info_token(&self, token: &AuthToken) -> anyhow::Result<Option<AuthToken>> {
        let url = format!("{}/token/info", &self.endpoint);
        let resp = self
            .inner
            .get(&url)
            .header(AUTHORIZATION, token.access.as_bearer())
            .query(&[("refresh", token.refresh.content)])
            .send()
            .await?
            .error_for_status()?;
        match resp.json::<AuthToken>().await {
            Ok(token) => token.Some(),
            Err(_) => None,
        }
        .Ok()
    }

    pub async fn delete_token(&self, token: &AuthToken) -> anyhow::Result<()> {
        let url = format!("{}/token/delete", &self.endpoint);
        let _resp = self
            .inner
            .delete(&url)
            .header(AUTHORIZATION, token.access.as_bearer())
            .send()
            .await?
            .error_for_status()?;
        ().Ok()
    }

    pub async fn auth_request(
        &self,
        req: &Request,
        token: &AuthToken,
    ) -> anyhow::Result<StatusCode> {
        let method = req.method().clone();
        let url = format!("{}/auth{}", self.endpoint, req.url().path());
        let access = token.access.as_bearer().try_into()?;

        let auth_req = reqwest::Request::new(method, url.as_str().try_into()?)
            .apply(|r| r.headers_mut().insert(AUTHORIZATION, access));
        let resp = self.inner.execute(auth_req).await?.error_for_status()?;
        let status = resp.status();
        let body = resp.text().await?;
        info!("{url} [{status}] {body}");
        Ok(status)
    }
}
