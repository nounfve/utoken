use anyhow::{Ok, anyhow};
use axum::http::HeaderMap;
use axum_extra::extract::{
    CookieJar,
    cookie::{Cookie, Expiration},
};
use reqwest::header::SET_COOKIE;
use time::OffsetDateTime;

use crate::token::{AuthToken, Claim, Token};

impl Token {
    pub fn into_cookie<'c>(self, name: &'c str) -> Cookie<'c> {
        let expire = OffsetDateTime::from_unix_timestamp(self.expire.timestamp()).unwrap();
        Cookie::build((name, self.content))
            .expires(expire)
            .http_only(true)
            .path("/")
            .build()
    }
}

impl<'c> TryFrom<&Cookie<'c>> for Token {
    type Error = anyhow::Error;

    fn try_from(value: &Cookie<'c>) -> Result<Self, Self::Error> {
        let content = value.value().to_string();
        if content.is_empty() {
            Err(anyhow!("empty content"))?
        }
        let expire = if let Some(Expiration::DateTime(expire)) = value.expires() {
            chrono::DateTime::from_timestamp(expire.unix_timestamp(), 0).unwrap()
        } else {
            Err(anyhow!("empty expire"))?
        };
        Ok(Self { content, expire })
    }
}

impl Into<CookieJar> for &AuthToken {
    fn into(self) -> CookieJar {
        let access = self.access.clone().into_cookie(AuthToken::UTOKEN_ACCESS);
        let refresh = self.refresh.clone().into_cookie(AuthToken::UTOKEN_REFRESH);
        CookieJar::new().add(access).add(refresh)
    }
}

impl TryFrom<CookieJar> for AuthToken {
    type Error = anyhow::Error;

    fn try_from(value: CookieJar) -> Result<Self, Self::Error> {
        let empty: Cookie = Cookie::new("", "");
        let access = value.get(AuthToken::UTOKEN_ACCESS).unwrap_or(&empty);
        println!("{access:?}");
        let access = access.try_into()?;
        let refresh: Token = value
            .get(AuthToken::UTOKEN_REFRESH)
            .unwrap_or(&empty)
            .try_into()?;
        Ok(Self {
            claim: Claim::from_str(".").unwrap(),
            access,
            refresh,
        })
    }
}

pub fn headers_to_cookie_jar(headers: &HeaderMap) -> CookieJar {
    let mut jar = CookieJar::new();
    for cookie in cookies_from_request(headers) {
        jar = jar.add(cookie);
    }
    jar
}

fn cookies_from_request(headers: &HeaderMap) -> impl Iterator<Item = Cookie<'static>> + '_ {
    headers
        .get_all(SET_COOKIE)
        .into_iter()
        .filter_map(|value| value.to_str().ok())
        .map(|c| Cookie::parse_encoded(c.to_string()).unwrap())
}

