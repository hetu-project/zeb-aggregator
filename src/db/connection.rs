use sea_orm::entity::prelude::*;
use sea_orm::{Database, DbBackend, Statement};
use tokio::sync::OnceCell;
use sea_orm_migration::prelude::*;
use super::migration::Migrator;
// use super::entities::{prelude::*, *};

async fn init_conn() -> DatabaseConnection {
    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL not set") + "/" + &*std::env::var("DB_NAME").expect("DATABASE_URL not set");
    Database::connect(db_url)
        .await
        .expect("failed to connect to database")
}

static CONN: OnceCell<DatabaseConnection> = OnceCell::const_new();

pub async fn get_conn() -> &'static DatabaseConnection {
    CONN.get_or_init(init_conn).await
}


pub async fn setup_db() -> Result<DatabaseConnection, DbErr> {
    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL not set");
    let db_name = std::env::var("DB_NAME").expect("DB_NAME not set");
    let db = Database::connect(db_url.clone()).await?;
    let db = match db.get_database_backend() {
        DbBackend::MySql => {
            db.execute(Statement::from_string(
                db.get_database_backend(),
                format!("CREATE DATABASE IF NOT EXISTS `{}`;", db_name),
            ))
                .await?;
            let url = format!("{}/{}", db_url.clone(), db_name);
            Database::connect(&url).await?
        }
        DbBackend::Postgres => {
            db.execute(Statement::from_string(
                db.get_database_backend(),
                format!("DROP DATABASE IF EXISTS \"{}\";", db_name),
            ))
                .await?;
            db.execute(Statement::from_string(
                db.get_database_backend(),
                format!("CREATE DATABASE \"{}\";", db_name),
            ))
                .await?;
            let url = format!("{}/{}", db_url.clone(), db_name);
            Database::connect(&url).await?
        }
        DbBackend::Sqlite => db,
    };

    let schema_manager = SchemaManager::new(&db); // To investigate the schema

    Migrator::up(&db, None).await.expect("fail to migrate");
    assert!(schema_manager.has_table("clock_infos").await.expect("can not find table"));
    assert!(schema_manager.has_table("merge_logs").await.expect("can not find table"));
    assert!(schema_manager.has_table("z_messages").await.expect("can not find table"));
    assert!(schema_manager.has_table("node_info").await.expect("can not find table"));
    Ok(db)
}
