use std::str::FromStr;

use axum::http::Uri;
use uuid::Uuid;

#[derive(Debug)]
pub struct Authorization {
    pub claim: Claim,
    pub access: Token,
    pub refresh: Token,
}

#[derive(Debug)]
pub struct Token {
    pub content: String,
    pub expire: u64,
}

#[derive(Debug)]
pub struct Claim {
    pub inner: Uri,
}

impl Authorization {
    pub fn new(claim: Claim) -> Self {
        let access = Token::new(0);
        let refresh = Token::new(0);

        Self {
            claim,
            access,
            refresh,
        }
    }
}

impl Token {
    pub fn new(expire: u64) -> Self {
        Self {
            content: Uuid::new_v4().to_string(),
            expire,
        }
    }
}

impl Claim {
    pub fn from_str(str: &str) -> anyhow::Result<Self> {
        let this = Self {
            inner: Uri::from_str(str)?,
        };
        Ok(this)
    }
}
