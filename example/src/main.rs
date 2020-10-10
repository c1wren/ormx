use anyhow::Result;
use sqlx::postgres::PgPoolOptions;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    // setup connection pool
    let database_url = "postgres://postgres@127.0.0.1/beta".to_string();
    let db_pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    // test insertion
    // returns a Club
    let mut club: Club = InsertClub {
        name: "test 4".into(),
        test1: "test 4".into(),
        test2: TestEnum::Test2,
        test4: Some(vec![1, 2, 3, 4]),
        r#type: 3,
    }
    .insert(&mut *db_pool.acquire().await?)
    .await?;
    dbg!(&club);

    // test patching
    let patch = PatchClub::default().set_name("New Name".into());
    club.patch(&mut *db_pool.acquire().await?, patch).await?;
    dbg!(club);

    // test get_optional
    let club = Club::by_name(&mut *db_pool.acquire().await?, &TestEnum::Test1).await?;
    dbg!(club);

    // test get_optional
    if let Some(club) = Club::by_id(&mut *db_pool.acquire().await?, &1).await? {
        // fetch by_id and then delete that club
        club.delete(&mut *db_pool.acquire().await?).await?;
        println!("deleted");
    }

    // fetch a different club and then use the 'set' update_name
    if let Some(mut club) = Club::by_id(&mut *db_pool.acquire().await?, &2).await? {
        club.update_name(&mut *db_pool.acquire().await?, TestEnum::Test4)
            .await?;
        dbg!(club);
    } else {
        println!("club not found")
    }

    // find all clubs
    let clubs = Club::find_all_clubs(&mut *db_pool.acquire().await?).await?;
    dbg!(clubs);

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
#[ormx(
    table = "clubs",
    id = "id",
    insertable,
    patchable,
    deletable,
    get_all = "find_all_clubs"
)]
struct Club {
    #[ormx(get_optional = "by_id")]
    id: i32,
    name: String,
    test1: String,
    #[ormx(get_optional = "by_name", set = "update_name")]
    // use a custom type that is really an i32
    // custom_type really includes the type in the sqlx query type checking
    #[ormx(custom_type, convert_as = "i32")]
    test2: TestEnum,
    #[ormx(default)]
    test3: Option<bool>,
    // only need special convert here because of Option<Vec>
    // if not Option<Vec>, you can use #[ormx(convert = "Vec::as_slice")]
    #[ormx(convert = "my_convert")]
    test4: Option<Vec<i32>>,
    r#type: i32,
}

#[allow(unused)]
fn my_convert(t: &Option<Vec<i32>>) -> Option<&[i32]> {
    t.as_ref().map(|the_t| the_t.as_slice())
}
