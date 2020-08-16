#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    Ok(())
}

#[derive(ormx::Entity, Debug)]
#[ormx(
    table = "clubs",
    id = club_id,
    insertable,
    patchable,
    deletable
)]
struct Club {
    #[ormx(get_one = by_id, delete = delete_by_id)]
    club_id: u32,
    #[ormx(
        get_optional = by_name,
        set = update_name
    )]
    name: String,
    #[ormx(generated)]
    created_at: chrono::NaiveDateTime,
}
