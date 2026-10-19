#[test]
fn locking_getter_rejects_a_plain_connection() {
    // This test has its own process. Trybuild's temporary workspace needs our query cache.
    std::env::set_var("SQLX_OFFLINE", "true");
    std::env::set_var(
        "SQLX_OFFLINE_DIR",
        concat!(env!("CARGO_MANIFEST_DIR"), "/../.sqlx"),
    );
    trybuild::TestCases::new().compile_fail("tests/ui/connection.rs");
}
