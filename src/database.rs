use sqlx::migrate::MigrateDatabase;
use sqlx::{Pool, Sqlite, SqlitePool};

pub struct DbConnection {
    pub pool: Pool<Sqlite>,
}

impl DbConnection {
    pub async fn build(db_url: &str) -> Self {
        let database_exists = Sqlite::database_exists(db_url).await.unwrap_or(false);

        if !database_exists {
            panic!("Database does not exist");
        }

        let pool = match SqlitePool::connect(db_url).await {
            Ok(c) => c,
            Err(e) => {
                panic!("It wasn't possible to open a connection with the Sqlite database: {e:?}")
            }
        };

        log::info!("Created connection pool for Sqlite {db_url}");
        Self { pool }
    }
}
