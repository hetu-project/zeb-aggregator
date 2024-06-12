use gateway::db::connection::setup_db;
#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    setup_db().await.expect("DB Setup error");
}
