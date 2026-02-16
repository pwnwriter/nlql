// tests for sql safety checks

use nlql::Safety;

#[test]
fn test_safe_select() {
    let safety = Safety::check("SELECT * FROM users");
    assert!(!safety.is_dangerous);
    assert!(safety.warning.is_none());
}

#[test]
fn test_dangerous_drop() {
    let safety = Safety::check("DROP TABLE users");
    assert!(safety.is_dangerous);
    assert!(safety.reason.contains("DROP"));
}

#[test]
fn test_dangerous_truncate() {
    let safety = Safety::check("TRUNCATE TABLE users");
    assert!(safety.is_dangerous);
    assert!(safety.reason.contains("TRUNCATE"));
}

#[test]
fn test_dangerous_delete_no_where() {
    let safety = Safety::check("DELETE FROM users");
    assert!(safety.is_dangerous);
    assert!(safety.reason.contains("DELETE"));
}

#[test]
fn test_safe_delete_with_where() {
    let safety = Safety::check("DELETE FROM users WHERE id = 1");
    assert!(!safety.is_dangerous);
    // should have a warning though
    assert!(safety.warning.is_some());
}

#[test]
fn test_dangerous_update_no_where() {
    let safety = Safety::check("UPDATE users SET name = 'x'");
    assert!(safety.is_dangerous);
    assert!(safety.reason.contains("UPDATE"));
}

#[test]
fn test_safe_update_with_where() {
    let safety = Safety::check("UPDATE users SET name = 'x' WHERE id = 1");
    assert!(!safety.is_dangerous);
    assert!(safety.warning.is_some());
}

#[test]
fn test_insert_warning() {
    let safety = Safety::check("INSERT INTO users (name) VALUES ('test')");
    assert!(!safety.is_dangerous);
    assert!(safety.warning.is_some());
    assert!(safety.warning.unwrap().contains("insert"));
}

#[test]
fn test_sql_injection_pattern() {
    let safety = Safety::check("SELECT * FROM users; DROP TABLE users");
    assert!(safety.is_dangerous);
}

#[test]
fn test_comment_injection() {
    let safety = Safety::check("SELECT * FROM users -- comment");
    assert!(safety.is_dangerous);
}
