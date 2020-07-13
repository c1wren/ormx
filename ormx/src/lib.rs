pub use macros::Entity;
pub use futures;
use sqlx::FromRow;

#[derive(sqlx::Type)]
#[repr(i64)]
enum UserRole {
    Guest = 0,
    Member = 1,
    Admin = 2,
}

#[derive(Entity, Debug)]
#[ormx(table = "users")]
struct User {
    #[ormx(
        get_optional = by_id,
        get_one,
        get_many,
        set = asdf,
        rename = "rowid",
        generated)
    ]
    id: i64,

    first_name: String,
    last_name: String,
    email: String,
}

#[cfg(test)]
mod tests {
    use sqlx::Connect;
    use crate::User;

    #[async_std::test]
    async fn test() {
        let mut con = sqlx::SqliteConnection::connect("sqlite:///home/realwork/ormx/test.sqlite").await.unwrap();
        println!("{:?}", User::by_id(&mut con, &1).await);
    }
}