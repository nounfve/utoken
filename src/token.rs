use std::str::FromStr;

use axum::http::Uri;
use cell_reg::cell_reg_named::StaticRefSingleton as _;
use chrono::{DateTime, Duration, Utc};
use uuid::Uuid;

use crate::database::DataBase;

#[derive(Debug)]
pub struct Authorization {
    pub claim: Claim,
    pub access: Token,
    pub refresh: Token,
}

#[derive(Debug)]
pub struct Token {
    pub content: String,
    pub expire: DateTime<Utc>,
}

#[derive(Debug)]
pub struct Claim {
    pub inner: Uri,
}

impl Authorization {
    pub fn new(claim: Claim) -> Self {
        let access = Token::new(Self::ACCESS_EXPIRE);
        let refresh = Token::new(Self::REFRESH_EXPIRE);

        Self {
            claim,
            access,
            refresh,
        }
    }

    pub async fn sql_insert_token(&self) -> anyhow::Result<()> {
        let db = DataBase::One();
        let _result = sqlx::query(
            "INSERT INTO utokens 
            (access,access_expire,refresh,refresh_expire,claim) VALUES 
            ($1, $2, $3, $4, $5)",
        )
        .bind(&self.access.content)
        .bind(&self.access.expire)
        .bind(&self.refresh.content)
        .bind(&self.refresh.expire)
        .bind(&self.claim.inner.to_string())
        .execute(&db.conn)
        .await?;
        Ok(())
    }

    const ACCESS_EXPIRE: i64 = 4 * 60 * 60 * 1000; // 4hr
    const REFRESH_EXPIRE: i64 = Self::ACCESS_EXPIRE * 180; // 30 days
}

impl Token {
    pub fn new(expire: i64) -> Self {
        let expire = Utc::now() + Duration::milliseconds(expire);
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
