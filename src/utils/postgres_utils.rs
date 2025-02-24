use std::sync::OnceLock;
use tokio_postgres::{Client, NoTls};

pub static POSTGRES_CLIENT: OnceLock<Client> = OnceLock::new();
pub async fn connect(
    db_host: String,
    db_user: String,
    db_pass: String,
    db_name: String,
    db_port: String,
) {
    let (client, connection) = tokio_postgres::connect(
        format!(
            "host={} user={} password={} dbname={} port={}",
            db_host, db_user, db_pass, db_name, db_port
        )
        .as_str(),
        NoTls,
    )
    .await
    .unwrap();
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            tracing::info!("POSTGRES connection error: {}", e.to_string());
        }
    });
    //it's ok for it to crash since it is still on initialization phase and is a requirement
    POSTGRES_CLIENT.get_or_init(|| client);
}
