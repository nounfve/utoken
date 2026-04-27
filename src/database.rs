use cell_reg::cell_reg::StaticRef;
use sqlx::{Pool, Postgres, postgres::PgPoolOptions};

pub struct DataBase {
    pub conn: Pool<Postgres>,
}

impl Default for DataBase {
    fn default() -> Self {
        panic!("shold call init instead default");
    }
}

impl DataBase {
    pub async fn init() -> anyhow::Result<()> {
        let conn = PgPoolOptions::new()
            .max_connections(2)
            .connect("postgres://utoken:some-secret@localhost/utoken")
            .await?;

        sqlx::migrate!("./migrations").run(&conn).await?;

        StaticRef::new(Self { conn }).setSingleton();
        Ok(())
    }
}
