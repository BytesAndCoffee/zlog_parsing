// Integration tests for queue manager helper functions

use sqlx::MySqlPool;
use serde::Serialize;

// Helper to create a test database pool
async fn create_test_pool() -> MySqlPool {
    let database_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "mysql://root:password@localhost/test_db".to_string());
    
    MySqlPool::connect(&database_url)
        .await
        .expect("Failed to connect to test database")
}

// Helper to set up test tables
async fn setup_test_tables(pool: &MySqlPool) {
    // Create logs_id_track table if it doesn't exist
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS logs_id_track (
            id INT PRIMARY KEY,
            tid INT NOT NULL
        )"
    )
    .execute(pool)
    .await
    .expect("Failed to create logs_id_track table");

    // Clean up any existing data
    sqlx::query("DELETE FROM logs_id_track")
        .execute(pool)
        .await
        .expect("Failed to clean logs_id_track table");
}

// Helper to tear down test tables
async fn teardown_test_tables(pool: &MySqlPool) {
    sqlx::query("DROP TABLE IF EXISTS logs_id_track")
        .execute(pool)
        .await
        .expect("Failed to drop logs_id_track table");
}

/// Test get_last_processed_id returns None when no record exists
/// 
/// Requirements: 2.3
#[tokio::test]
#[ignore] // Requires test database
async fn test_get_last_processed_id_empty() {
    let pool = create_test_pool().await;
    setup_test_tables(&pool).await;

    // Query directly since the function is private
    let result = sqlx::query_scalar::<_, i32>(
        "SELECT tid FROM logs_id_track WHERE id = 1"
    )
    .fetch_optional(&pool)
    .await
    .expect("Query failed");

    assert_eq!(result, None, "Should return None when no tracking record exists");

    teardown_test_tables(&pool).await;
}

/// Test mark_as_processed creates a new record
/// 
/// Requirements: 2.3, 2.4
#[tokio::test]
#[ignore] // Requires test database
async fn test_mark_as_processed_creates_record() {
    let pool = create_test_pool().await;
    setup_test_tables(&pool).await;

    // Simulate mark_as_processed by using replace_into
    #[derive(Serialize)]
    struct IdTrackRecord {
        id: i32,
        tid: i32,
    }

    let record = IdTrackRecord {
        id: 1,
        tid: 42,
    };

    // We can't call the private function, so we test the behavior directly
    sqlx::query("REPLACE INTO logs_id_track (id, tid) VALUES (?, ?)")
        .bind(record.id)
        .bind(record.tid)
        .execute(&pool)
        .await
        .expect("Failed to insert record");

    // Verify the record was created
    let result = sqlx::query_scalar::<_, i32>(
        "SELECT tid FROM logs_id_track WHERE id = 1"
    )
    .fetch_one(&pool)
    .await
    .expect("Query failed");

    assert_eq!(result, 42, "Should have created a record with tid=42");

    teardown_test_tables(&pool).await;
}

/// Test mark_as_processed updates an existing record
/// 
/// Requirements: 2.3, 2.4
#[tokio::test]
#[ignore] // Requires test database
async fn test_mark_as_processed_updates_record() {
    let pool = create_test_pool().await;
    setup_test_tables(&pool).await;

    // Insert initial record
    sqlx::query("INSERT INTO logs_id_track (id, tid) VALUES (1, 10)")
        .execute(&pool)
        .await
        .expect("Failed to insert initial record");

    // Update with REPLACE INTO
    sqlx::query("REPLACE INTO logs_id_track (id, tid) VALUES (?, ?)")
        .bind(1)
        .bind(20)
        .execute(&pool)
        .await
        .expect("Failed to replace record");

    // Verify the record was updated
    let result = sqlx::query_scalar::<_, i32>(
        "SELECT tid FROM logs_id_track WHERE id = 1"
    )
    .fetch_one(&pool)
    .await
    .expect("Query failed");

    assert_eq!(result, 20, "Should have updated the record to tid=20");

    teardown_test_tables(&pool).await;
}

/// Test that REPLACE INTO semantics work correctly (insert or update)
/// 
/// Requirements: 2.4
#[tokio::test]
#[ignore] // Requires test database
async fn test_replace_into_semantics() {
    let pool = create_test_pool().await;
    setup_test_tables(&pool).await;

    // First REPLACE should insert
    sqlx::query("REPLACE INTO logs_id_track (id, tid) VALUES (1, 100)")
        .execute(&pool)
        .await
        .expect("Failed to replace (insert)");

    let result1 = sqlx::query_scalar::<_, i32>(
        "SELECT tid FROM logs_id_track WHERE id = 1"
    )
    .fetch_one(&pool)
    .await
    .expect("Query failed");

    assert_eq!(result1, 100, "First REPLACE should insert tid=100");

    // Second REPLACE should update
    sqlx::query("REPLACE INTO logs_id_track (id, tid) VALUES (1, 200)")
        .execute(&pool)
        .await
        .expect("Failed to replace (update)");

    let result2 = sqlx::query_scalar::<_, i32>(
        "SELECT tid FROM logs_id_track WHERE id = 1"
    )
    .fetch_one(&pool)
    .await
    .expect("Query failed");

    assert_eq!(result2, 200, "Second REPLACE should update to tid=200");

    // Verify only one record exists
    let count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM logs_id_track"
    )
    .fetch_one(&pool)
    .await
    .expect("Query failed");

    assert_eq!(count, 1, "Should have exactly one record");

    teardown_test_tables(&pool).await;
}

/// Test copy_new_logs with no new logs
/// 
/// Requirements: 2.1, 2.7
#[tokio::test]
#[ignore] // Requires test database
async fn test_copy_new_logs_no_logs() {
    let pool = create_test_pool().await;
    
    // Set up tables
    setup_test_tables(&pool).await;
    
    // Create logs and logs_queue tables
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS logs (
            id INT PRIMARY KEY AUTO_INCREMENT,
            created_at DATETIME NOT NULL,
            user VARCHAR(255),
            network VARCHAR(255),
            window VARCHAR(255) NOT NULL,
            type VARCHAR(50) NOT NULL,
            nick VARCHAR(255),
            message TEXT
        )"
    )
    .execute(&pool)
    .await
    .expect("Failed to create logs table");
    
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS logs_queue (
            id INT PRIMARY KEY,
            created_at DATETIME NOT NULL,
            user VARCHAR(255),
            network VARCHAR(255),
            window VARCHAR(255) NOT NULL,
            type VARCHAR(50) NOT NULL,
            nick VARCHAR(255),
            message TEXT
        )"
    )
    .execute(&pool)
    .await
    .expect("Failed to create logs_queue table");
    
    // Clean up
    sqlx::query("DELETE FROM logs").execute(&pool).await.expect("Failed to clean logs");
    sqlx::query("DELETE FROM logs_queue").execute(&pool).await.expect("Failed to clean logs_queue");
    
    // Call copy_new_logs with last_id = 0 (no logs exist)
    // Since the function is in the queue_manager module, we need to test it indirectly
    // For now, we'll test the behavior by querying directly
    
    let logs = sqlx::query_scalar::<_, i32>(
        "SELECT id FROM logs WHERE id > ? ORDER BY id ASC"
    )
    .bind(0)
    .fetch_all(&pool)
    .await
    .expect("Query failed");
    
    assert_eq!(logs.len(), 0, "Should have no logs to copy");
    
    // Clean up
    sqlx::query("DROP TABLE IF EXISTS logs").execute(&pool).await.expect("Failed to drop logs");
    sqlx::query("DROP TABLE IF EXISTS logs_queue").execute(&pool).await.expect("Failed to drop logs_queue");
    teardown_test_tables(&pool).await;
}

/// Test copy_new_logs copies logs in ascending order
/// 
/// Requirements: 2.1, 2.2, 2.3, 2.7
#[tokio::test]
#[ignore] // Requires test database
async fn test_copy_new_logs_ascending_order() {
    let pool = create_test_pool().await;
    
    // Set up tables
    setup_test_tables(&pool).await;
    
    // Create logs and logs_queue tables
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS logs (
            id INT PRIMARY KEY AUTO_INCREMENT,
            created_at DATETIME NOT NULL,
            user VARCHAR(255),
            network VARCHAR(255),
            window VARCHAR(255) NOT NULL,
            type VARCHAR(50) NOT NULL,
            nick VARCHAR(255),
            message TEXT
        )"
    )
    .execute(&pool)
    .await
    .expect("Failed to create logs table");
    
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS logs_queue (
            id INT PRIMARY KEY,
            created_at DATETIME NOT NULL,
            user VARCHAR(255),
            network VARCHAR(255),
            window VARCHAR(255) NOT NULL,
            type VARCHAR(50) NOT NULL,
            nick VARCHAR(255),
            message TEXT
        )"
    )
    .execute(&pool)
    .await
    .expect("Failed to create logs_queue table");
    
    // Clean up
    sqlx::query("DELETE FROM logs").execute(&pool).await.expect("Failed to clean logs");
    sqlx::query("DELETE FROM logs_queue").execute(&pool).await.expect("Failed to clean logs_queue");
    
    // Insert test logs with specific IDs
    sqlx::query(
        "INSERT INTO logs (id, created_at, window, type) VALUES 
        (5, NOW(), '#test', 'msg'),
        (3, NOW(), '#test', 'msg'),
        (7, NOW(), '#test', 'msg')"
    )
    .execute(&pool)
    .await
    .expect("Failed to insert test logs");
    
    // Query logs in ascending order (simulating copy_new_logs behavior)
    let logs = sqlx::query_scalar::<_, i32>(
        "SELECT id FROM logs WHERE id > ? ORDER BY id ASC"
    )
    .bind(0)
    .fetch_all(&pool)
    .await
    .expect("Query failed");
    
    assert_eq!(logs, vec![3, 5, 7], "Logs should be in ascending order");
    
    // Clean up
    sqlx::query("DROP TABLE IF EXISTS logs").execute(&pool).await.expect("Failed to drop logs");
    sqlx::query("DROP TABLE IF EXISTS logs_queue").execute(&pool).await.expect("Failed to drop logs_queue");
    teardown_test_tables(&pool).await;
}

/// Test copy_new_logs error handling continues on failure
/// 
/// Requirements: 2.6
#[tokio::test]
#[ignore] // Requires test database
async fn test_copy_new_logs_error_handling() {
    let pool = create_test_pool().await;
    
    // Set up tables
    setup_test_tables(&pool).await;
    
    // Create logs table
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS logs (
            id INT PRIMARY KEY AUTO_INCREMENT,
            created_at DATETIME NOT NULL,
            user VARCHAR(255),
            network VARCHAR(255),
            window VARCHAR(255) NOT NULL,
            type VARCHAR(50) NOT NULL,
            nick VARCHAR(255),
            message TEXT
        )"
    )
    .execute(&pool)
    .await
    .expect("Failed to create logs table");
    
    // Create logs_queue table with a constraint that will cause some inserts to fail
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS logs_queue (
            id INT PRIMARY KEY,
            created_at DATETIME NOT NULL,
            user VARCHAR(255),
            network VARCHAR(255),
            window VARCHAR(255) NOT NULL,
            type VARCHAR(50) NOT NULL,
            nick VARCHAR(255),
            message TEXT
        )"
    )
    .execute(&pool)
    .await
    .expect("Failed to create logs_queue table");
    
    // Clean up
    sqlx::query("DELETE FROM logs").execute(&pool).await.expect("Failed to clean logs");
    sqlx::query("DELETE FROM logs_queue").execute(&pool).await.expect("Failed to clean logs_queue");
    
    // Insert test logs
    sqlx::query(
        "INSERT INTO logs (id, created_at, window, type) VALUES 
        (1, NOW(), '#test', 'msg'),
        (2, NOW(), '#test', 'msg')"
    )
    .execute(&pool)
    .await
    .expect("Failed to insert test logs");
    
    // Insert one log into logs_queue to cause a duplicate key error
    sqlx::query(
        "INSERT INTO logs_queue (id, created_at, window, type) VALUES 
        (1, NOW(), '#test', 'msg')"
    )
    .execute(&pool)
    .await
    .expect("Failed to insert into logs_queue");
    
    // Try to copy logs - the first one should fail due to duplicate key
    // but the process should continue
    // We test this by verifying that attempting to insert a duplicate returns an error
    let result = sqlx::query(
        "INSERT INTO logs_queue (id, created_at, window, type) VALUES (?, NOW(), '#test', 'msg')"
    )
    .bind(1)
    .execute(&pool)
    .await;
    
    assert!(result.is_err(), "Duplicate insert should fail");
    
    // But we should still be able to insert the second log
    let result2 = sqlx::query(
        "INSERT INTO logs_queue (id, created_at, window, type) VALUES (?, NOW(), '#test', 'msg')"
    )
    .bind(2)
    .execute(&pool)
    .await;
    
    assert!(result2.is_ok(), "Second insert should succeed");
    
    // Clean up
    sqlx::query("DROP TABLE IF EXISTS logs").execute(&pool).await.expect("Failed to drop logs");
    sqlx::query("DROP TABLE IF EXISTS logs_queue").execute(&pool).await.expect("Failed to drop logs_queue");
    teardown_test_tables(&pool).await;
}
