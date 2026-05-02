#![cfg(test)]

use std::time::Duration;

use reqwest::Method;

use crate::_main;

use super::Client;

#[tokio::test]
async fn test_create_token() {
    tokio::spawn(_main());
    tokio::time::sleep(Duration::from_secs(1)).await;

    let client = Client::default();

    let mut token = client
        .create_token(format!("u://+delete-post@./*"))
        .await
        .unwrap();

    let sub_token = client
        .create_sub_token(&token, format!("u://no-effect@no-effect./some-claim"))
        .await
        .unwrap();

    let mut req = reqwest::Request::new(
        reqwest::Method::GET,
        "https://some-damain/some-claim?some=query"
            .try_into()
            .unwrap(),
    );

    let link = client.link_bind(&req, &token).await.unwrap();

    {
        // test default allow
        let resp = client.auth_request(&req, &token).await;
        assert!(&resp.unwrap() == ".");

        *req.method_mut() = reqwest::Method::POST;
        let resp = client.auth_request(&req, &token).await;
        assert!(resp.is_err());

        *req.method_mut() = reqwest::Method::DELETE;
        let resp = client.auth_request(&req, &token).await;
        assert!(&resp.unwrap() == ".");
    }

    {
        // test info noop when access not near expire
        let info_refresh = client.info_token(&token).await;
        assert!(info_refresh.unwrap().is_none());

        let resp = client.auth_request(&req, &token).await;
        assert!(&resp.unwrap() == ".");
    }

    {
        // test force refresh
        #[allow(unused)]
        let token2 = client.refresh_token(&token).await.unwrap();

        // old token invalid
        let resp = client.auth_request(&req, &token).await;
        assert!(resp.is_err());

        let resp = client.auth_request(&req, &token2).await;
        assert!(&resp.unwrap() == ".");

        let _ = std::mem::replace(&mut token, token2);
    }

    {
        let resp = client.auth_request(&req, &sub_token).await;
        assert!(&resp.unwrap() == ".");

        let resolved = client.resolve_link(Method::GET, &link).await;
        assert!(resolved.unwrap() == "/some-claim?some=query")
    }

    {
        let resp = client.delete_token(&token).await;
        assert!(resp.is_ok());

        let resp = client.auth_request(&req, &token).await;
        assert!(resp.is_err());

        let resp = client.auth_request(&req, &sub_token).await;
        assert!(resp.is_err());

        let resolved = client.resolve_link(Method::GET, &link).await;
        assert!(resolved.is_err())

    }
}
