pub use utoken::client::*;

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use reqwest::Url;

    use crate::Client;

    #[tokio::test]
    async fn test_create_token() {
        let client = Client::default();
        let token = client
            .create_token(format!("u://+delete-post@./some-claim"))
            .await
            .unwrap();

        let url = format!("{}/auth/some-claim", client.endpoint);
        let url = Url::from_str(&url).unwrap();
        let req = reqwest::Request::new(reqwest::Method::GET, url);
        client.auth_request(req, &token).await.unwrap();

        #[allow(unused)]
        let token = client.refresh_token(token).await.unwrap();
    }
}
