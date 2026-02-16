// tests for database operations
// run with: cargo test --features test-db
// requires DATABASE_URL env var

#![cfg(feature = "test-db")]

use nlql::Db;

fn get_db_url() -> String {
    std::env::var("DATABASE_URL").expect("DATABASE_URL must be set for db tests")
}

#[tokio::test]
async fn test_connect() {
    let db = Db::connect(&get_db_url()).await;
    assert!(db.is_ok());
}

#[tokio::test]
async fn test_schema() {
    let db = Db::connect(&get_db_url()).await.unwrap();
    let schema = db.schema().await.unwrap();

    // should contain our test tables
    assert!(schema.contains("users"));
    assert!(schema.contains("orders"));
}

#[tokio::test]
async fn test_execute_select() {
    let db = Db::connect(&get_db_url()).await.unwrap();
    let result = db.execute("SELECT id, name FROM users").await.unwrap();

    assert_eq!(result.columns.len(), 2);
    assert!(result.row_count > 0);
}

#[tokio::test]
async fn test_execute_count() {
    let db = Db::connect(&get_db_url()).await.unwrap();
    let result = db
        .execute("SELECT COUNT(*) as count FROM users")
        .await
        .unwrap();

    assert_eq!(result.columns[0], "count");
    assert_eq!(result.row_count, 1);
}

#[tokio::test]
async fn test_execute_with_where() {
    let db = Db::connect(&get_db_url()).await.unwrap();
    let result = db
        .execute("SELECT name FROM users WHERE role = 'admin'")
        .await
        .unwrap();

    assert!(result.row_count >= 1);
}

#[tokio::test]
async fn test_execute_join() {
    let db = Db::connect(&get_db_url()).await.unwrap();
    let result = db
        .execute(
            "SELECT u.name, o.amount
             FROM users u
             JOIN orders o ON u.id = o.user_id",
        )
        .await
        .unwrap();

    assert_eq!(result.columns.len(), 2);
    assert!(result.row_count > 0);
}

#[tokio::test]
async fn test_empty_result() {
    let db = Db::connect(&get_db_url()).await.unwrap();
    let result = db
        .execute("SELECT * FROM users WHERE id = -999")
        .await
        .unwrap();

    assert_eq!(result.row_count, 0);
    assert!(result.rows.is_empty());
}
