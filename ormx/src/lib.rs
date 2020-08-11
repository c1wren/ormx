pub use macros::Entity;

#[cfg(feature = "sqlite")]
type Con = sqlx::SqliteConnection;
#[cfg(feature = "mysql")]
type Con = sqlx::MySqlConnection;
#[cfg(feature = "postgres")]
type Con = sqlx::PostgresConnection;

#[derive(Entity)]
#[ormx(table = "users")]
pub struct User {
    #[ormx(generated, primary_key, get_one = by_id)]
    user_id: i64,
    first_name: String,
    last_name: String,
    email: String,
}
