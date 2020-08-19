use anyhow::Result;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    // let patch = PatchClub::default()
    //     .set_name("Hello".into())
    //     .set_test1("World".into())
    //     .set_test3(Some(false));

    let database_url = "postgres://postgres@127.0.0.1/beta".to_string();
    let mut db_pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    let mut club = InsertClub {
        name: "test 4".into(),
        test1: "test 4".into(),
        test2: TestEnum::Test2,
        test3: Some(true),
        test4: None,
    }
    .insert(&mut db_pool)
    .await?;

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

#[allow(unused)]
fn enum_convert(val: &TestEnum) -> i32 {
    *val as i32
}

#[derive(ormx::Entity, sqlx::FromRow, Debug)]
#[ormx(
    table = "clubs",
    id = id,
    insertable,
    patchable,
    deletable
)]
struct Club {
    #[ormx(get_optional = "by_id")]
    id: i32,
    name: String,
    test1: String,
    #[ormx(get_optional = "by_name", set = "update_name")]
    #[ormx(custom_type, convert = "enum_convert")]
    test2: TestEnum,
    test3: Option<bool>,
    #[ormx(convert = "my_convert")]
    test4: Option<Vec<i32>>,
}

#[allow(unused)]
fn my_convert(t: &Option<Vec<i32>>) -> Option<&[i32]> {
    t.as_ref().map(|the_t| the_t.as_slice())
}

// #[derive(ormx::Patch)]
// #[ormx(table = "clubs")]
// struct PatchClubName {
//     name: String,
//     test1: String,
// }

// trait Patch {
//     type RowType;

//     fn execute(...) -> Result<RowType>;
// }

// impl Club {
//     fn patch(patch: impl Patch<RowType = Self>) -> Result<Self> {
//         patch.execute(...)
//     }
// }

// fn main() {
//     let club = Club::fetch(1);
//     // let mut patch = club.to_patch();
//     // patch.name = "Hello".into();
//     // patch.test1 = "World".into();
//     let patch = PatchClub {
//         name: Some("Hello".into()),
//         test1: Some("World".into()),
//         ..Default::default()
//     };

//     let patch = PatchClub::new()
//         .set_name("Hello".into())
//         .set_test1("World".into())
//         .set_test3(None); // sets test3 to Some(None)

//     club.patch(patch); // Also updates test2, test3
// }
