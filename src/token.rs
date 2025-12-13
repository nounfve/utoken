use std::str::FromStr;

use axum::http::Uri;
use cell_reg::cell_reg_named::StaticRefSingleton as _;
use chrono::{DateTime, Duration, Utc};
use sqlx::{FromRow, Row, postgres::PgRow};
use uuid::Uuid;

use crate::database::DataBase;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct AuthToken {
    pub claim: Claim,
    pub access: Token,
    pub refresh: Token,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Token {
    pub content: String,
    pub expire: DateTime<Utc>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct Claim {
    #[serde(with = "http_serde::uri")]
    pub inner: Uri,
}

impl AuthToken {
    pub fn new(claim: Claim) -> Self {
        let access = Token::new(Self::ACCESS_EXPIRE);
        let refresh = Token::new(Self::REFRESH_EXPIRE);

        Self {
            claim,
            access,
            refresh,
        }
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

impl AuthToken {
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

    pub async fn sql_find_access_token(token: &str) -> anyhow::Result<Self> {
        let db = DataBase::One();
        let token = sqlx::query_as(
            "SELECT * FROM utokens 
            WHERE access = $1",
        )
        .bind(token)
        .fetch_one(&db.conn)
        .await?;
        Ok(token)
    }
}

impl FromRow<'_, PgRow> for AuthToken {
    fn from_row(row: &'_ PgRow) -> Result<Self, sqlx::Error> {
        let claim = Claim::from_str(row.try_get("claim")?).unwrap();
        Ok(Self {
            claim,
            access: Token {
                content: row.try_get("access")?,
                expire: row.try_get("access_expire")?,
            },
            refresh: Token {
                content: row.try_get("refresh")?,
                expire: row.try_get("refresh_expire")?,
            },
        })
    }
}
