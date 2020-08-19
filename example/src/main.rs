use anyhow::Result;
use sqlx::postgres::PgPoolOptions;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let database_url = "postgres://postgres@127.0.0.1/beta".to_string();
    let db_pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    let mut club = InsertClub {
        name: "test 4".into(),
        test1: "test 4".into(),
        test2: TestEnum::Test2,
        test3: Some(false),
        test4: Some(vec![1, 2, 3, 4]),
        r#type: 3,
    }
    .insert(&db_pool)
    .await?;
    dbg!("first", &club);

    let patch = PatchClub::default().set_name("New Name".into());
    club.patch(&db_pool, patch).await?;

    dbg!(club);

    let club = Club::by_name(&db_pool, &TestEnum::Test1).await?;
    dbg!(club);

    if let Some(club) = Club::by_id(&db_pool, &1).await? {
        club.delete(&db_pool).await?;
        println!("deleted");
    }

    if let Some(mut club) = Club::by_id(&db_pool, &2).await? {
        club.update_name(&db_pool, TestEnum::Test4).await?;
        dbg!(club);
    } else {
        println!("club not found")
    }

    Ok(())
}

#[derive(
    serde::Serialize,
    serde::Deserialize,
    PartialEq,
    PartialOrd,
    Eq,
    Ord,
    Clone,
    Copy,
    Debug,
    sqlx::Type,
)]
#[repr(i32)]
pub enum TestEnum {
    Test1 = 1,
    Test2 = 2,
    Test3 = 3,
    Test4 = 4,
}

#[derive(ormx::Entity, sqlx::FromRow, Debug)]
#[ormx(table = "clubs", id = "id", insertable, patchable, deletable)]
struct Club {
    #[ormx(get_optional = "by_id")]
    id: i32,
    name: String,
    test1: String,
    #[ormx(get_optional = "by_name", set = "update_name")]
    #[ormx(custom_type, convert_as = "i32")]
    test2: TestEnum,
    test3: Option<bool>,
    #[ormx(convert = "my_convert")]
    test4: Option<Vec<i32>>,
    #[ormx(set = "set_type", rename = "type")]
    r#type: i32,
}

#[allow(unused)]
fn my_convert(t: &Option<Vec<i32>>) -> Option<&[i32]> {
    t.as_ref().map(|the_t| the_t.as_slice())
}
