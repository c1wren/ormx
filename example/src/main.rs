#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    let pool = sqlx::MySqlPool::connect("mysql://root:admin@172.17.0.2/yourclub_backend").await?;

    let mut club = Club::by_id(&pool, &1).await?;
    println!("queried: {:?}", club);

    println!("after patch: {:?}", club);

    Ok(())
}

#[derive(ormx::Entity, Debug)]
#[ormx(
    table = "clubs",
    id = club_id,
    insertable,
    patchable
)]
struct Club {
    #[ormx(get_one = by_id)]
    club_id: u32,
    #[ormx(
        get_optional = by_name,
        set = update_name
    )]
    name: String,
    #[ormx(generated)]
    created_at: chrono::NaiveDateTime
}