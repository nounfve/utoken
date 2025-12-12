use std::str::FromStr;

use reqwest::Url;

pub struct Client {
    inner: reqwest::Client,
    endpoint: String,
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
    pub async fn create_token(&self, claim: String) -> anyhow::Result<()> {
        let url = format!("{}/token/create", &self.endpoint);
        let url = Url::from_str(&url)?;
        let resp = self.inner.put(url).body(claim).send().await?;
        println!("{resp:?}");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::Client;

    #[tokio::test]
    async fn test_create_token() {
        Client::default()
            .create_token(format!("u://+get-put@./some-claim"))
            .await
            .unwrap();
    }
}
