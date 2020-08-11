use ormx::sqlx;
use sqlx::Connection;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut con = sqlx::SqliteConnection::connect("sqlite://../test.sqlite").await?;

    User::insert(
        &mut con,
        &InsertUser {
            first_name: "asdf".to_owned(),
            last_name: "xxx".to_owned(),
            email: "asdfxx".to_owned(),
        },
    )
    .await?;

    let (id,): (i64,) = sqlx::query_as("SELECT last_insert_rowid();")
        .fetch_one(&mut con)
        .await?;

    let inserted = User::get_by_id(&mut con, &id).await?;
    println!("{:?}", inserted);

    Ok(())
}

#[derive(ormx::Entity, Debug)]
#[ormx(table = "users")]
struct User {
    #[ormx(get_one = get_by_id, generated)]
    user_id: i64,
    first_name: String,
    last_name: String,
    email: String,
}
