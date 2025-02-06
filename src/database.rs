use sqlx::{PgPool, Pool, Postgres};

pub struct DbConnection {
    pub pool: Pool<Postgres>,
}

impl DbConnection {
    pub async fn build(db_url: &str) -> Self {
        let pool = match PgPool::connect(db_url).await {
            Ok(c) => c,
            Err(e) => {
                panic!("It wasn't possible to open a connection with the Sqlite database: {e:?}")
            }
        };

        log::info!("Created connection pool for Sqlite {db_url}");
        Self { pool }
    }
}
