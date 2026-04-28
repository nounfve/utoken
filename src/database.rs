use sqlx::{Pool, Postgres, postgres::PgPoolOptions};
use sutils::{Singleton, extension::str::env_or};

#[Singleton(zeroed)]
pub struct DataBase {
    pub conn: Pool<Postgres>,
}

impl DataBase {
    pub async fn init() -> anyhow::Result<()> {
        const DB_URL: &str = "postgres://utoken:some-secret@localhost/utoken";
        
        let conn = PgPoolOptions::new()
            .max_connections(2)
            .connect(env_or!(DB_URL))
            .await?;

        sqlx::migrate!("./migrations").run(&conn).await?;
        Self::Set(Self { conn });
        Ok(())
    }
}
