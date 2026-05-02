use std::{collections::HashSet, str::FromStr};

use anyhow::anyhow;
use axum::http::Uri;
use chrono::{DateTime, Duration, Utc};
use glob::Pattern;
use sqlx::{FromRow, Row, postgres::PgRow};
use sutils::{IntoOption, IntoResult, Singleton};
use tokio::time::sleep;
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
    pub content: Uuid,
    pub expire: DateTime<Utc>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct Claim {
    #[serde(with = "http_serde::uri")]
    pub inner: Uri,
}

impl AuthToken {
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }

    pub const ACCESS_EXPIRE: Duration = Duration::hours(4);
    pub const REFRESH_EXPIRE: Duration = Duration::days(30);
    pub const UTOKEN_ACCESS: &str = "uA";
    pub const UTOKEN_REFRESH: &str = "uR";
    pub const HEAD_X_SCOPE: &str = "X-Claim-Scope";
}

impl Token {
    pub fn new(unique: Uuid, alive: Duration) -> Self {
        let expire = Utc::now() + alive;
        Self { content: unique, expire }
    }

    pub fn as_bearer(&self) -> String {
        format!("Bearer {}", self.content)
    }
}

impl Claim {
    pub fn from_str(str: &str) -> anyhow::Result<Self> {
        let this = Self { inner: Uri::from_str(str)? };
        Ok(this)
    }

    pub fn user_name(&self) -> Option<&str> {
        self.inner
            .authority()
            .map(|a| a.as_str().split("@").next())
            .flatten()
            .unwrap_or("")
            .split(":")
            .next()
    }

    pub fn reducted() -> Self {
        Self { inner: Uri::from_static("~") }
    }

    pub fn scope_only(&self) -> Self {
        Self {
            inner: Uri::from_str(&self.parse_scope_name()).unwrap(),
        }
    }

    pub fn sub_claim(&self, url: &str) -> anyhow::Result<Self> {
        let url = Uri::from_str(url)?;
        if !self.match_path(url.path()) {
            return anyhow!("not valid child claim").Err();
        }
        let scheme = match self.inner.scheme_str() {
            Some(s) => format!("{s}://"),
            None => String::new(),
        };
        let auth = match self.inner.authority() {
            Some(auth) => auth.to_string(),
            None => String::new(),
        };
        let url = format!("{scheme}{auth}{}", url.path());
        Self::from_str(&url)?.Ok()
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

        pattern.matches(&path)
    }

    pub fn match_method(&self, method: &str) -> bool {
        let user_name = self
            .user_name()
            .unwrap_or("")
            .replace("+", ":+")
            .replace("-", ":-")
            .to_lowercase();

        let username_as_allowed = user_name // breakline
            .split(":")
            .skip_while(|str| str.is_empty());

        let allow = &mut HashSet::from(["get", "post"]);
        for val in username_as_allowed {
            match val.split_at(1) {
                ("+", val) => allow.insert(val),
                ("-", val) => allow.remove(val),
                _ => false,
            };
        }

        allow.contains(&*method.to_lowercase())
    }

    pub fn parse_scope_name(&self) -> &str {
        self.inner
            .host()
            .map(|s| s.trim_end_matches("."))
            .map(|s| if s.is_empty() { s.None() } else { s.Some() })
            .flatten()
            .unwrap_or(".")
    }
}

impl AuthToken {
    pub async fn sql_insert_token(claim: Claim, parent: Option<&Uuid>) -> anyhow::Result<Self> {
        let sql = r#"
            INSERT INTO utokens (refresh, scope, claim, child_of)
            VALUES (gen_random_uuid(), $1, $2, $3)
            RETURNING (refresh);
        "#;
        let row = sqlx::query(sql)
            .bind(claim.parse_scope_name())
            .bind(claim.inner.to_string())
            .bind(parent)
            .fetch_one(&DataBase::One().conn)
            .await?;

        let refresh = row.try_get::<Uuid, _>("refresh")?;

        Self::sql_refresh_token(&refresh).await
    }

    pub async fn sql_refresh_token(refresh: &Uuid) -> anyhow::Result<Self> {
        let sql = r#"
            UPDATE utokens
            SET access = gen_random_uuid(),
                access_expire = now() + ($2 * interval '1 second'),
                refresh = gen_random_uuid(),
                refresh_expire = now() + ($3 * interval '1 second')
            where refresh = $1
            RETURNING *;
        "#;
        let row = sqlx::query(sql)
            .bind(refresh)
            .bind(AuthToken::ACCESS_EXPIRE.num_seconds())
            .bind(AuthToken::REFRESH_EXPIRE.num_seconds())
            .fetch_one(&DataBase::One().conn)
            .await?;
        AuthToken::from_row(&row)?.Ok()
    }

    pub async fn sql_find_access_token(access: &Uuid) -> anyhow::Result<Self> {
        let sql = r#"
            SELECT *
            FROM utokens
            where access = $1
                AND refresh_expire > now();
        "#;
        let row = sqlx::query(sql)
            .bind(access)
            .fetch_one(&DataBase::One().conn)
            .await?;
        AuthToken::from_row(&row)?.Ok()
    }

    pub async fn sql_find_refresh_token(refresh: &Uuid) -> anyhow::Result<Self> {
        let sql = r#"
            SELECT *
            FROM utokens
            where refresh = $1
                AND refresh_expire > now();
        "#;
        let row = sqlx::query(sql)
            .bind(refresh)
            .fetch_one(&DataBase::One().conn)
            .await?;
        AuthToken::from_row(&row)?.Ok()
    }

    pub async fn sql_delete_token(access: &Uuid) -> anyhow::Result<Self> {
        let sql = r#"
            DELETE 
            FROM utokens
            where access = $1
            RETURNING *;
        "#;
        let row = sqlx::query(sql)
            .bind(access)
            .fetch_one(&DataBase::One().conn)
            .await?;
        AuthToken::from_row(&row)?.Ok()
    }

    pub async fn clean_outdated_token() {
        loop {
            if let Err(err) = sqlx::query(
                "DELETE FROM utokens
            WHERE refresh_expire < NOW()",
            )
            .execute(&DataBase::One().conn)
            .await
            {
                error!("{err}");
                sleep(Duration::seconds(60).to_std().expect("never")).await;
                continue;
            }
            sleep(Duration::hours(18).to_std().expect("never")).await
        }
    }
}

impl FromRow<'_, PgRow> for AuthToken {
    fn from_row(row: &'_ PgRow) -> Result<Self, sqlx::Error> {
        let claim = Claim::from_str(row.try_get("claim")?).unwrap();
        Self {
            claim,
            access: Token {
                content: row.try_get("access")?,
                expire: row.try_get("access_expire")?,
            },
            refresh: Token {
                content: row.try_get("refresh")?,
                expire: row.try_get("refresh_expire")?,
            },
        }
        .Ok()
    }
}
