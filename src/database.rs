use sqlx::{Pool, Postgres, postgres::PgPoolOptions};
use sutils::Singleton;

#[Singleton(zeroed)]
pub struct DataBase {
    pub conn: Pool<Postgres>,
}

impl DataBase {
    pub async fn init() -> anyhow::Result<()> {
        let conn = PgPoolOptions::new()
            .max_connections(2)
            .connect("postgres://utoken:some-secret@localhost/utoken")
            .await?;

        sqlx::migrate!("./migrations").run(&conn).await?;

        Self::Set(Self { conn });
        Ok(())
    }
}
