pub use futures;
pub use macros::Entity;
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
        get_one = by_id,
        rename = "rowid",
        generated,
        primary_key
    )]
    id: i64,
    first_name: String,
    last_name: String,
    #[ormx(
        get_optional = find_by_email,
        set = change_email

    )]
    email: String,
}

#[cfg(test)]
mod tests {
    use crate::User;
    use sqlx::Connection;
    use sqlx::Result;
    use sqlx::SqliteConnection;

    #[async_std::test]
    async fn test() -> Result<()> {
        let mut con = SqliteConnection::connect("sqlite:///home/realwork/ormx/test.sqlite").await?;

        let user = User::insert(&mut con, "Lana", "Rey")
            .await?;

        println!("{:?}", User::by_id(&mut con, &1).await);
        Ok(())
    }
}
