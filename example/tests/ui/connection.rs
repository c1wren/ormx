#[derive(ormx::Entity)]
#[ormx(table = "locking_records")]
struct Record {
    #[ormx(key, rename = "record_id", get_one_for_update = "locked")]
    id: i32,
}

async fn rejects_connection(conn: &mut sqlx::PgConnection) {
    let _ = Record::locked(conn, &1).await;
}

fn main() {}
