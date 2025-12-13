use std::time::Duration;

use cell_reg::cell_reg_named::StaticRefSingleton as _;
use tokio::time::sleep;
use tracing::error;

use crate::database::DataBase;

pub async fn clean_outdated_token() {
    let db = DataBase::One();
    loop {
        if let Err(err) = sqlx::query(
            "DELETE FROM utokens
            WHERE refresh_expire < NOW()",
        )
        .execute(&db.conn)
        .await
        {
            error!("{err}");
            sleep(Duration::from_secs(60)).await;
            continue;
        }
        sleep(Duration::from_secs(60 * 60 * 22)).await
    }
}
