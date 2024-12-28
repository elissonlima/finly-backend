use sqlx::migrate::MigrateDatabase;
use sqlx::{Executor, Pool, Sqlite, SqlitePool};
use std::fs::File;
use std::io::{self, Read};

pub struct DbConnection {
    pub pool: Pool<Sqlite>,
}

impl DbConnection {
    pub async fn build(db_url: &str, ddl_script_path: &str) -> Self {
        let database_exists = Sqlite::database_exists(db_url).await.unwrap_or(false);

        log::info!("Checking if database exists: {}", database_exists);

        if !database_exists {
            match Sqlite::create_database(db_url).await {
                Ok(_) => log::info!("Successfully created database at {}", db_url),
                Err(error) => panic!("It wasn't possible to create database {}", error),
            };
        }

        let pool = match SqlitePool::connect(db_url).await {
            Ok(c) => c,
            Err(e) => {
                panic!("It wasn't possible to open a connection with the Sqlite database: {e:?}")
            }
        };

        if !database_exists {
            let mut con = match pool.acquire().await {
                Ok(c) => c,
                Err(e) => panic!("Error getting connection to database: {e:?}"),
            };

            match Self::run_ddl_script(ddl_script_path, &mut *con).await {
                Ok(_) => log::info!("Successfuly initialized database"),
                Err(e) => panic!("Error while executing ddl script: {}", e),
            };
        }

        Self { pool }
    }

    fn sql_script_from_file(script_path: &str) -> Result<String, io::Error> {
        let mut file = File::open(script_path)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        Ok(content)
    }

    async fn run_ddl_script<'e, E>(
        ddl_script_path: &str,
        con: E,
    ) -> Result<(), Box<dyn std::error::Error>>
    where
        E: Executor<'e, Database = Sqlite>,
    {
        let sql = Self::sql_script_from_file(ddl_script_path)?;

        sqlx::query(sql.as_str()).execute(con).await?;
        Ok(())
    }
}
