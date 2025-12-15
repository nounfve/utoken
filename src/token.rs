use std::{collections::HashSet, str::FromStr};

use axum::http::Uri;
use cell_reg::cell_reg_named::StaticRefSingleton as _;
use chrono::{DateTime, Duration, Utc};
use glob::Pattern;
use sqlx::{FromRow, Row, postgres::PgRow};
use tracing::error;
use uuid::Uuid;

use crate::database::DataBase;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AuthToken {
    pub claim: Claim,
    pub access: Token,
    pub refresh: Token,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Token {
    pub content: String,
    pub expire: DateTime<Utc>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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

    pub const ACCESS_EXPIRE: i64 = 4 * 60 * 60 * 1000; // 4hr
    pub const REFRESH_EXPIRE: i64 = Self::ACCESS_EXPIRE * 180; // 30 days
    pub const UTOKEN_ACCESS: &str = "uA";
    pub const UTOKEN_REFRESH: &str = "uR";
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

    pub fn reducted() -> Self {
        Self {
            inner: Uri::from_static("~"),
        }
    }
}

impl Claim {
    pub fn match_path(&self, path: &str) -> bool {
        let pattern = match Pattern::new(self.inner.path()) {
            Ok(p) => p,
            Err(err) => {
                error!("{err}");
                return false;
            }
        };

        if !pattern.matches(&path) {
            return false;
        };

        true
    }

    pub fn match_method(&self, method: &str) -> bool {
        let user_info = self
            .inner
            .authority()
            .map(|a| a.as_str().split("@").next().unwrap_or(""))
            .unwrap_or("");
        let username_as_allowed = user_info
            .split(":")
            .next()
            .unwrap_or("")
            .replace("+", ":+")
            .replace("-", ":-")
            .split(":")
            .map(|str| str.to_lowercase())
            .collect::<Vec<_>>();

        let allow = &mut HashSet::from(["get", "post"]);
        let deny = &mut HashSet::new();
        username_as_allowed.iter().for_each(|e| {
            if e.starts_with("+") {
                allow.insert(&e[1..]);
            } else if e.starts_with("-") {
                deny.insert(&e[1..]);
            }
        });

        let method = method.to_lowercase();
        if deny.contains(method.as_str()) {
            false
        } else if allow.contains(&method.as_str()) {
            true
        } else {
            false
        }
    }

    pub fn parse_scope_name(&self) -> Option<String> {
        self.inner
            .host()
            .map(|s| s.trim_end_matches(".").to_owned())
    }
}

impl AuthToken {
    pub async fn sql_insert_token(&self) -> anyhow::Result<()> {
        let db = DataBase::One();
        let _result = sqlx::query(
            "INSERT INTO utokens 
            (access,access_expire,refresh,refresh_expire,scope,claim) VALUES 
            ($1, $2, $3, $4, $5, $6)",
        )
        .bind(&self.access.content)
        .bind(&self.access.expire)
        .bind(&self.refresh.content)
        .bind(&self.refresh.expire)
        .bind(&self.claim.parse_scope_name())
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

    pub async fn sql_find_refresh_token(token: &str) -> anyhow::Result<Self> {
        let db = DataBase::One();
        let token = sqlx::query_as(
            "SELECT * FROM utokens 
            WHERE refresh = $1",
        )
        .bind(token)
        .fetch_one(&db.conn)
        .await?;
        Ok(token)
    }

    pub async fn sql_delete_token(&self) -> anyhow::Result<()> {
        let db = DataBase::One();
        let _result = sqlx::query(
            "DELETE FROM utokens 
            WHERE refresh = $1",
        )
        .bind(&self.refresh.content)
        .execute(&db.conn)
        .await?;
        Ok(())
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
