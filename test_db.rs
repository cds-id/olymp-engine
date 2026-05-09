use sqlx::postgres::PgPoolOptions;

#[tokio::main]
async fn main() {
    let url = "postgres://blurp:blurp@127.0.0.1:5433/blurp";
    println!("Connecting to: {}", url);
    match PgPoolOptions::new()
        .max_connections(1)
        .connect_timeout(std::time::Duration::from_secs(5))
        .connect(url)
        .await
    {
        Ok(pool) => println!("✓ Connected"),
        Err(e) => println!("✗ Error: {}", e),
    }
}
