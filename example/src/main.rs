use anyhow::Result;
use serde::Serialize;
use sqlx::{postgres::PgPoolOptions, PgConnection};

mod error;
use error::TestError;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    // setup connection pool
    let database_url = "postgres://postgres@127.0.0.1/ormx".to_string();
    let db_pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    // test insertion
    // let club_ctx: Club = InsertClub {
    //     name: "test 4".into(),
    //     test_rename: "test 4".into(),
    //     test2: TestEnum::Test2,
    //     test4: Some(vec![1, 2, 3, 4]),
    //     r#type: 3,
    // }
    // .insert_with_context(&mut *db_pool.acquire().await?, Some(&6))
    // .await?;
    // dbg!(&club_ctx);

    // returns a Club
    let mut club: Club = InsertClub {
        name: "test 4".into(),
        test_rename: "test 4".into(),
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
        println!("deleted club");
    }

    // fetch a different club and then use the 'set' update_name
    if let Some(mut club) = Club::by_id(&mut *db_pool.acquire().await?, &2).await? {
        club.update_enum(&mut *db_pool.acquire().await?, TestEnum::Test4)
            .await?;
        dbg!(&club);
        club.name = "Testing 123".into();
        club.my_update(&mut *db_pool.acquire().await?).await?;
        dbg!(club);
    } else {
        println!("club not found")
    }

    // find all clubs
    let clubs = Club::find_all_clubs(&mut *db_pool.acquire().await?).await?;

    for club in clubs {
        dbg!(&club);
        club.delete(&mut *db_pool.acquire().await?).await?;
    }

    Ok(())
}

#[derive(Clone, Copy, Debug, sqlx::Type, serde::Serialize)]
#[repr(i32)]
pub enum TestEnum {
    Test1 = 1,
    Test2 = 2,
    Test3 = 3,
    Test4 = 4,
}

// by default, when you derive Entity, you only get the functionality of updating a model
// derive insertable, patchable, and deletable to have the respective functionality
#[derive(ormx::Entity, sqlx::FromRow, Debug, Clone)]
#[ormx(
    table = "clubs",
    update = "my_update",
    insertable,
    patchable,
    deletable,
    get_all = "find_all_clubs",
    context_type = "i32",
    error_type = "TestError",
    before_patch = "Club::before_patch",
    after_patch = "Club::after_patch",
    before_update = "Club::before_update",
    after_update = "Club::after_update",
    before_insert = "Club::before_insert",
    after_insert = "Club::after_insert",
    before_delete = "Club::before_delete",
    after_delete = "Club::after_delete"
)]
struct Club {
    #[ormx(key, default, get_optional = "by_id")]
    id: i32,
    name: String,
    #[ormx(rename = "test1")]
    test_rename: String,
    #[ormx(get_optional = "by_name", set = "update_enum")]
    // use a custom type that is really an i32
    // custom_type forces type inference by sqlx to eliminate errors
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

impl Club {
    async fn before_patch(
        _model: &Club,
        _patch: &PatchClub,
        _context: Option<&i32>,
        _db_pool: &mut PgConnection,
    ) -> Result<(), TestError> {
        println!("before patch");
        Ok(())
    }

    async fn after_patch(
        _model: &Club,
        _previous: Club,
        _context: Option<&i32>,
        _db_pool: &mut PgConnection,
    ) -> Result<(), TestError> {
        println!("after patch");
        dbg!(_model);
        dbg!(_previous);
        Ok(())
    }

    async fn before_update(
        _model: &Club,
        _context: Option<&i32>,
        _db_pool: &mut PgConnection,
    ) -> Result<(), TestError> {
        println!("before update");
        Ok(())
    }

    async fn after_update(
        _model: &Club,
        _previous: Club,
        _context: Option<&i32>,
        _db_pool: &mut PgConnection,
    ) -> Result<(), TestError> {
        println!("after update");
        dbg!(_model);
        dbg!(_previous);
        Ok(())
    }

    async fn before_delete(
        _model: &Club,
        _context: Option<&i32>,
        _db_pool: &mut PgConnection,
    ) -> Result<(), TestError> {
        println!("before delete");
        Ok(())
    }

    async fn after_delete(
        _model: Club,
        _context: Option<&i32>,
        _db_pool: &mut PgConnection,
    ) -> Result<(), TestError> {
        println!("after delete");
        Ok(())
    }

    async fn before_insert(
        _model: &InsertClub,
        _context: Option<&i32>,
        _db_pool: &mut PgConnection,
    ) -> Result<(), TestError> {
        println!("before insert");
        Ok(())
    }

    async fn after_insert(
        _model: &Club,
        _context: Option<&i32>,
        _db_pool: &mut PgConnection,
    ) -> Result<(), TestError> {
        println!("after insert");
        Ok(())
    }
}

#[allow(unused)]
fn my_convert(t: &Option<Vec<i32>>) -> Option<&[i32]> {
    t.as_ref().map(Vec::as_slice)
}

// by default, when you derive Entity, you only get the functionality of updating a model
// derive insertable, patchable, and deletable to have the respective functionality
#[derive(ormx::Entity, sqlx::FromRow, Debug)]
#[ormx(
    table = "composite",
    insertable,
    patchable,
    deletable,
    context_type = "i32",
    error_type = "TestError",
    before_patch = "Composite::before_patch",
    after_patch = "Composite::after_patch",
    before_update = "Composite::before_update",
    after_update = "Composite::after_update",
    before_insert = "Composite::before_insert",
    after_insert = "Composite::after_insert",
    before_delete = "Composite::before_delete",
    after_delete = "Composite::after_delete"
)]
struct Composite {
    #[ormx(key)]
    name: String,
    #[ormx(key, patchable)]
    other: String,
}

impl Composite {
    async fn before_patch(
        _model: &Composite,
        _patch: &PatchComposite,
        _context: Option<&i32>,
        _db_pool: &mut PgConnection,
    ) -> Result<(), TestError> {
        println!("before patch");
        Ok(())
    }

    async fn after_patch(
        _model: &Composite,
        _previous: Composite,
        _context: Option<&i32>,
        _db_pool: &mut PgConnection,
    ) -> Result<(), TestError> {
        println!("after patch");
        dbg!(_model);
        dbg!(_previous);
        Ok(())
    }

    async fn before_update(
        _model: &Composite,
        _context: Option<&i32>,
        _db_pool: &mut PgConnection,
    ) -> Result<(), TestError> {
        println!("before update");
        Ok(())
    }

    async fn after_update(
        _model: &Composite,
        _previous: Composite,
        _context: Option<&i32>,
        _db_pool: &mut PgConnection,
    ) -> Result<(), TestError> {
        println!("after update");
        dbg!(_model);
        dbg!(_previous);
        Ok(())
    }

    async fn before_delete(
        _model: &Composite,
        _context: Option<&i32>,
        _db_pool: &mut PgConnection,
    ) -> Result<(), TestError> {
        println!("before delete");
        Ok(())
    }

    async fn after_delete(
        _model: Composite,
        _context: Option<&i32>,
        _db_pool: &mut PgConnection,
    ) -> Result<(), TestError> {
        println!("after delete");
        Ok(())
    }

    async fn before_insert(
        _model: &InsertComposite,
        _context: Option<&i32>,
        _db_pool: &mut PgConnection,
    ) -> Result<(), TestError> {
        println!("before insert");
        Ok(())
    }

    async fn after_insert(
        _model: &Composite,
        _context: Option<&i32>,
        _db_pool: &mut PgConnection,
    ) -> Result<(), TestError> {
        println!("after insert");
        Ok(())
    }
}
