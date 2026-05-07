pub struct LinkBind {
    pub path: String,
    pub query: Option<String>,
    pub tokens: Option<(Uuid, Uuid)>,
}

impl LinkBind {
    pub async fn sql_insert_link(&self, bind: &Uuid) -> anyhow::Result<Uuid> {
        let sql = r#"
            INSERT INTO link_bind (link, path, query, bind_to)
            VALUES (gen_random_uuid(), $1, $2, $3)
            RETURNING link;
        "#;
        let row = sqlx::query(sql)
            .bind(&self.path)
            .bind(&self.query)
            .bind(&bind)
            .fetch_one(&DataBase::One().conn)
            .await?;
        let link = row.get::<Uuid, _>("link");
        link.Ok()
    }

    pub async fn sql_retrive_link(link: &Uuid) -> anyhow::Result<LinkBind> {
        let sql = r#"
            SELECT L.path,
                L.query,
                T.access,
                T.refresh
            FROM link_bind L
                JOIN utokens T ON T.refresh = L.bind_to
            where L.link = $1;
        "#;
        let row = sqlx::query(sql)
            .bind(link)
            .fetch_one(&DataBase::One().conn)
            .await?;
        Self::from_row(&row)?.Ok()
    }

    pub async fn sql_list_links(refresh: &Uuid) -> anyhow::Result<LinkPathVec> {
        let sql = r#"
            SELECT L.path,
                L.link
            FROM utokens T
                JOIN link_bind L ON L.bind_to = T.refresh
            WHERE T.child_of = $1
            LIMIT 200;
        "#;
        let rows = sqlx::query(sql) // force break
            .bind(&refresh)
            .fetch_all(&DataBase::One().conn)
            .await?;
        let links = rows
            .iter()
            .map(|row| (row.get("link"), row.get("path")))
            .collect::<LinkPathVec>();
        links.Ok()
    }
}

pub type LinkPathVec = Vec<(Uuid, String)>;

impl LinkBind {
    pub fn set_headers(&self, mut headers: HeaderMap) -> HeaderMap {
        headers.insert(
            Self::HEAD_X_PATH,
            HeaderValue::from_str(&self.path).unwrap(),
        );
        if let Some(query) = &self.query {
            headers.insert(Self::HEAD_X_QUERY, HeaderValue::from_str(query).unwrap());
        }
        headers
    }

    pub fn from_headers(headers: &HeaderMap) -> Self {
        let path = headers
            .get(Self::HEAD_X_PATH)
            .map(|h| h.to_str().unwrap().to_string())
            .unwrap_or_default();
        let query = headers
            .get(Self::HEAD_X_QUERY)
            .map(|h| h.to_str().unwrap().to_string());
        Self { path, query, tokens: None }
    }

    pub fn to_path_query(&self) -> String {
        let Self { path, query, .. } = self;
        let query = query.as_ref().map_or("", |q| q.as_str());
        format!("{path}{query}")
    }

    pub const HEAD_X_PATH: &str = "X-Link-Path";
    pub const HEAD_X_QUERY: &str = "X-Link-Query";
}

impl<'r> FromRow<'r, PgRow> for LinkBind {
    fn from_row(row: &'r PgRow) -> Result<Self, sqlx::Error> {
        Self {
            path: row.try_get("path")?,
            query: row.try_get("query")?,
            tokens: (row.try_get("access")?, row.try_get("refresh")?).Some(),
        }
        .Ok()
    }
}

use axum::http::{HeaderMap, HeaderValue};
use sqlx::{FromRow, Row, postgres::PgRow};
use sutils::{IntoOption, IntoResult, Singleton};
use uuid::Uuid;

use crate::database::DataBase;
