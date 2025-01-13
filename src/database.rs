use sqlx::migrate::MigrateDatabase;
use sqlx::{Pool, Sqlite, SqlitePool};

pub struct DbConnection {
    pub pool: Pool<Sqlite>,
}

impl DbConnection {
    pub async fn build(db_url: &str) -> Self {
        let database_exists = Sqlite::database_exists(db_url).await.unwrap_or(false);

        log::info!("Checking if database exists: {}", database_exists);

        if !database_exists {
            panic!("Database does not exist");
        }

        let pool = match SqlitePool::connect(db_url).await {
            Ok(c) => c,
            Err(e) => {
                panic!("It wasn't possible to open a connection with the Sqlite database: {e:?}")
            }
        };

        Self { pool }
    }
}
