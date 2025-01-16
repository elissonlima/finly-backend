use sqlx::migrate::MigrateDatabase;
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::{Acquire, Pool, Sqlite, SqlitePool};
use std::fs::File;
use std::io::Read;

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

    pub async fn build_in_memory(ddl_script_path: &str) -> Self {
        let con = SqliteConnectOptions::new().in_memory(true);

        log::info!("Created connection pool for Sqlite in-memory");
        let pool = match SqlitePool::connect_with(con).await {
            Ok(p) => p,
            Err(e) => {
                panic!("It wasn't possible to open a connection with Sqlite In-Memory database : {e:?}")
            }
        };

        let mut file = match File::open(ddl_script_path) {
            Ok(f) => f,
            Err(e) => {
                panic!("It wasn't possible to open ddl file: {e:?}");
            }
        };
        let mut sql = String::new();
        match file.read_to_string(&mut sql) {
            Ok(_) => (),
            Err(e) => {
                panic!("It wasn't possible to read ddl file: {e:?}");
            }
        };

        let mut db = match pool.acquire().await {
            Ok(d) => d,
            Err(e) => {
                panic!("Error while acquiring DB connection from pool: {e:?}");
            }
        };

        match sqlx::query(sql.as_str()).execute(&mut *db).await {
            Ok(_) => (),
            Err(e) => {
                panic!("Error while executing DDL script: {e:?}");
            }
        }
        log::info!("Successfully initialized in-memory database");
        Self { pool }
    }
}
