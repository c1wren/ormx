#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    let patch = PatchClub::default().set_name("Hello".into());
    Ok(())
}

#[derive(ormx::Entity, Debug)]
#[ormx(
    table = "clubs",
    id = id,
    insertable,
    patchable,
    deletable
)]
struct Club {
    #[ormx(get_one = by_id, delete = delete_by_id)]
    id: i32,
    #[ormx(
        get_optional = by_name,
        set = update_name
    )]
    name: String,
    test1: String,
    test2: i32,
    test3: Option<bool>,
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
