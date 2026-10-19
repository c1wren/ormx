use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use sqlx::{postgres::PgPoolOptions, Connection, PgConnection, PgPool, Postgres, Transaction};
use tokio::time::{sleep, timeout};

#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type)]
#[repr(i32)]
enum Kind {
    First = 1,
    Second = 2,
}

fn label_suffix() -> (String,) {
    ("!".to_owned(),)
}

fn as_text(value: &str) -> &str {
    value
}

#[derive(Debug, ormx::Entity)]
#[ormx(
    table = "locking_records",
    get_all_for_update,
    get_all_for_no_key_update,
    get_all_for_share,
    get_all_for_key_share
)]
struct Record {
    #[ormx(
        key,
        rename = "record_id",
        get_one = "plain",
        get_one_for_update = "one_update",
        get_one_for_no_key_update = "one_no_key_update",
        get_one_for_share = "one_share",
        get_one_for_key_share = "one_key_share",
        get_optional_for_update = "optional_update",
        get_optional_for_no_key_update = "optional_no_key_update",
        get_optional_for_share = "optional_share",
        get_optional_for_key_share = "optional_key_share"
    )]
    id: i32,
    #[ormx(key)]
    position: i32,
    #[ormx(
        get_many_for_update = "many_update",
        get_many_for_no_key_update = "many_no_key_update",
        get_many_for_share = "many_share",
        get_many_for_key_share = "many_key_share"
    )]
    group_id: i32,
    #[ormx(
        transform_get = "concat($field, $param1::text)",
        transform_get_params = "label_suffix"
    )]
    label: String,
    #[ormx(
        custom_type,
        convert_as = "i32",
        get_optional_for_key_share = "by_kind"
    )]
    kind: Kind,
    #[ormx(convert = "as_text", get_one_for_share = "by_tag")]
    tag: String,
}

#[derive(Debug, thiserror::Error)]
enum TestError {
    #[error(transparent)]
    Database(#[from] sqlx::Error),
}

#[derive(Debug, ormx::Entity)]
#[ormx(table = "locking_records", error_type = "TestError")]
struct CustomRecord {
    #[ormx(
        key,
        rename = "record_id",
        get_one_for_update = "required",
        get_optional_for_share = "maybe"
    )]
    id: i32,
}

static GET_PARAMS_CALLS: AtomicU64 = AtomicU64::new(0);
static SET_PARAMS_CALLS: AtomicU64 = AtomicU64::new(0);

fn counted_get_params() -> (String, String) {
    GET_PARAMS_CALLS.fetch_add(1, Ordering::SeqCst);
    ("@".to_owned(), String::new())
}

fn counted_set_params() -> (String, String) {
    SET_PARAMS_CALLS.fetch_add(1, Ordering::SeqCst);
    ("@".to_owned(), "@".to_owned())
}

fn take_param_calls() -> (u64, u64) {
    (
        GET_PARAMS_CALLS.swap(0, Ordering::SeqCst),
        SET_PARAMS_CALLS.swap(0, Ordering::SeqCst),
    )
}

#[derive(Debug, Clone, ormx::Entity)]
#[ormx(
    table = "transform_records",
    insertable,
    patchable,
    after_update = "TransformRecord::after_change",
    after_patch = "TransformRecord::after_change"
)]
struct TransformRecord {
    #[ormx(key, default, get_one = "read", get_one_for_update = "locked")]
    id: i32,
    #[ormx(
        set = "set_value",
        transform_get = "replace($field, $param1::text, $param2::text)",
        transform_get_params = "counted_get_params",
        transform_set = "concat($param1::text, $field::text, $param2::text)",
        transform_set_params = "counted_set_params"
    )]
    value: String,
}

impl TransformRecord {
    async fn after_change(
        _model: &Self,
        _previous: Self,
        _context: Option<()>,
        _conn: &mut PgConnection,
    ) -> Result<(), sqlx::Error> {
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
enum Mode {
    Update,
    NoKeyUpdate,
    Share,
    KeyShare,
}

const MODES: [Mode; 4] = [Mode::Update, Mode::NoKeyUpdate, Mode::Share, Mode::KeyShare];

async fn one(
    mode: Mode,
    tx: &mut Transaction<'_, Postgres>,
    id: i32,
) -> Result<Record, sqlx::Error> {
    match mode {
        Mode::Update => Record::one_update(tx, &id).await,
        Mode::NoKeyUpdate => Record::one_no_key_update(tx, &id).await,
        Mode::Share => Record::one_share(tx, &id).await,
        Mode::KeyShare => Record::one_key_share(tx, &id).await,
    }
}

async fn optional(
    mode: Mode,
    tx: &mut Transaction<'_, Postgres>,
    id: i32,
) -> Result<Option<Record>, sqlx::Error> {
    match mode {
        Mode::Update => Record::optional_update(tx, &id).await,
        Mode::NoKeyUpdate => Record::optional_no_key_update(tx, &id).await,
        Mode::Share => Record::optional_share(tx, &id).await,
        Mode::KeyShare => Record::optional_key_share(tx, &id).await,
    }
}

async fn many(
    mode: Mode,
    tx: &mut Transaction<'_, Postgres>,
    group_id: i32,
) -> Result<Vec<Record>, sqlx::Error> {
    match mode {
        Mode::Update => Record::many_update(tx, &group_id).await,
        Mode::NoKeyUpdate => Record::many_no_key_update(tx, &group_id).await,
        Mode::Share => Record::many_share(tx, &group_id).await,
        Mode::KeyShare => Record::many_key_share(tx, &group_id).await,
    }
}

async fn all(mode: Mode, tx: &mut Transaction<'_, Postgres>) -> Result<Vec<Record>, sqlx::Error> {
    match mode {
        Mode::Update => Record::get_all_for_update(tx).await,
        Mode::NoKeyUpdate => Record::get_all_for_no_key_update(tx).await,
        Mode::Share => Record::get_all_for_share(tx).await,
        Mode::KeyShare => Record::get_all_for_key_share(tx).await,
    }
}

struct Database {
    pool: PgPool,
    admin: PgConnection,
    schema: String,
}

impl Database {
    async fn new() -> Self {
        let url = std::env::var("TEST_DATABASE_URL")
            .expect("locking tests require TEST_DATABASE_URL pointing to disposable PostgreSQL");
        static NEXT_SCHEMA: AtomicU64 = AtomicU64::new(0);
        let schema = format!(
            "ormx_lock_{}_{}",
            std::process::id(),
            NEXT_SCHEMA.fetch_add(1, Ordering::Relaxed)
        );
        let mut admin = PgConnection::connect(&url).await.unwrap();
        // Identifiers contain only this process ID, an atomic counter, and a fixed prefix.
        sqlx::raw_sql(sqlx::AssertSqlSafe(format!(
            "CREATE SCHEMA {schema}; SET search_path TO {schema}"
        )))
        .execute(&mut admin)
        .await
        .unwrap();
        sqlx::raw_sql(include_str!("fixtures/locking.sql"))
            .execute(&mut admin)
            .await
            .unwrap();
        let search_path = format!("SET search_path TO {schema}; SET statement_timeout = '5s'");
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .after_connect(move |conn, _| {
                let search_path = search_path.clone();
                Box::pin(async move {
                    sqlx::raw_sql(sqlx::AssertSqlSafe(search_path))
                        .execute(conn)
                        .await?;
                    Ok(())
                })
            })
            .connect(&url)
            .await
            .unwrap();
        Self {
            pool,
            admin,
            schema,
        }
    }

    async fn wait_for_block(&mut self, holder: i32, waiter: i32) {
        timeout(Duration::from_secs(3), async {
            loop {
                let blocked = sqlx::query_scalar!(
                    r#"SELECT $1::integer = ANY(pg_blocking_pids($2)) AS "blocked!""#,
                    holder,
                    waiter
                )
                .fetch_one(&mut self.admin)
                .await
                .unwrap();
                if blocked {
                    break;
                }
                sleep(Duration::from_millis(10)).await;
            }
        })
        .await
        .expect("contender did not wait on the holder's PostgreSQL lock");
    }

    async fn finish(mut self) {
        self.pool.close().await;
        sqlx::raw_sql(sqlx::AssertSqlSafe(format!(
            "DROP SCHEMA {} CASCADE",
            self.schema
        )))
        .execute(&mut self.admin)
        .await
        .unwrap();
    }
}

async fn backend_pid(tx: &mut Transaction<'_, Postgres>) -> i32 {
    sqlx::query_scalar!("SELECT pg_backend_pid()")
        .fetch_one(&mut **tx)
        .await
        .unwrap()
        .unwrap()
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL pointing to disposable PostgreSQL"]
async fn every_getter_preserves_results_missing_rows_and_composite_order() {
    let db = Database::new().await;
    for mode in MODES {
        let mut tx = db.pool.begin().await.unwrap();
        let record = one(mode, &mut tx, 2).await.unwrap();
        assert_eq!((record.id, record.position, record.group_id), (2, 1, 10));
        assert_eq!(record.label, "second!");
        assert_eq!(record.kind, Kind::Second);
        assert_eq!(record.tag, "two");
        assert_eq!(optional(mode, &mut tx, 2).await.unwrap().unwrap().id, 2);
        assert!(optional(mode, &mut tx, 999).await.unwrap().is_none());
        assert!(matches!(
            one(mode, &mut tx, 999).await,
            Err(sqlx::Error::RowNotFound)
        ));
        assert!(many(mode, &mut tx, 999).await.unwrap().is_empty());
        let rows = many(mode, &mut tx, 10).await.unwrap();
        assert_eq!(
            rows.iter()
                .map(|row| (row.id, row.position))
                .collect::<Vec<_>>(),
            [(1, 1), (1, 2), (2, 1)]
        );
        let rows = all(mode, &mut tx).await.unwrap();
        assert_eq!(
            rows.iter()
                .map(|row| (row.id, row.position))
                .collect::<Vec<_>>(),
            [(1, 1), (1, 2), (2, 1), (3, 1)]
        );
        assert_eq!(
            Record::by_kind(&mut tx, &Kind::First)
                .await
                .unwrap()
                .unwrap()
                .id,
            3
        );
        assert_eq!(
            Record::by_tag(&mut tx, &"two".to_owned()).await.unwrap().id,
            2
        );
        sqlx::query!("DELETE FROM locking_records")
            .execute(&mut *tx)
            .await
            .unwrap();
        assert!(all(mode, &mut tx).await.unwrap().is_empty());
        tx.rollback().await.unwrap();
    }
    db.finish().await;
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL pointing to disposable PostgreSQL"]
async fn custom_error_type_preserves_missing_row_behavior() {
    let db = Database::new().await;
    let mut tx = db.pool.begin().await.unwrap();
    assert_eq!(CustomRecord::required(&mut tx, &2).await.unwrap().id, 2);
    assert!(matches!(
        CustomRecord::required(&mut tx, &999).await,
        Err(TestError::Database(sqlx::Error::RowNotFound))
    ));
    assert!(CustomRecord::maybe(&mut tx, &999).await.unwrap().is_none());
    tx.rollback().await.unwrap();
    db.finish().await;
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL pointing to disposable PostgreSQL"]
async fn multi_parameter_transform_providers_run_once_per_query() {
    let db = Database::new().await;
    let mut tx = db.pool.begin().await.unwrap();
    take_param_calls();

    let mut record = InsertTransformRecord {
        value: "inserted".to_owned(),
    }
    .insert(&mut tx)
    .await
    .unwrap();
    assert_eq!(record.value, "inserted");
    assert_eq!(
        take_param_calls(),
        (1, 1),
        "INSERT and RETURNING have separate providers"
    );
    let stored = sqlx::query_scalar!(
        "SELECT value FROM transform_records WHERE id = $1",
        record.id
    )
    .fetch_one(&mut *tx)
    .await
    .unwrap();
    assert_eq!(stored, "@inserted@");

    assert_eq!(
        TransformRecord::read(&mut tx, &record.id)
            .await
            .unwrap()
            .value,
        "inserted"
    );
    assert_eq!(take_param_calls(), (1, 0));
    assert_eq!(
        TransformRecord::locked(&mut tx, &record.id)
            .await
            .unwrap()
            .value,
        "inserted"
    );
    assert_eq!(take_param_calls(), (1, 0));

    record.value = "updated".to_owned();
    record.update(&mut tx).await.unwrap();
    assert_eq!(
        take_param_calls(),
        (1, 1),
        "after-update hook loads the previous row"
    );
    record.set_value(&mut tx, "set".to_owned()).await.unwrap();
    assert_eq!(take_param_calls(), (0, 1));
    record
        .patch(
            &mut tx,
            PatchTransformRecord::default().set_value("patched".to_owned()),
        )
        .await
        .unwrap();
    assert_eq!(
        take_param_calls(),
        (1, 1),
        "after-patch hook loads the previous row"
    );

    record.no_trigger_update(&mut tx).await.unwrap();
    assert_eq!(take_param_calls(), (0, 1));
    record
        .no_trigger_set_value(&mut tx, "without-hook".to_owned())
        .await
        .unwrap();
    assert_eq!(take_param_calls(), (0, 1));
    record
        .no_trigger_patch(
            &mut tx,
            PatchTransformRecord::default().set_value("final".to_owned()),
        )
        .await
        .unwrap();
    assert_eq!(take_param_calls(), (0, 1));
    assert_eq!(
        TransformRecord::read(&mut tx, &record.id)
            .await
            .unwrap()
            .value,
        "final"
    );
    assert_eq!(take_param_calls(), (1, 0));

    tx.rollback().await.unwrap();
    db.finish().await;
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL pointing to disposable PostgreSQL"]
async fn all_sixteen_lock_pairs_match_postgresql_and_release_on_transaction_end() {
    let mut db = Database::new().await;
    // Rows are held modes, columns are requested modes, in MODES order.
    let conflicts = [
        [true, true, true, true],
        [true, true, true, false],
        [true, true, false, false],
        [true, false, false, false],
    ];
    for (held_index, held) in MODES.into_iter().enumerate() {
        for (requested_index, requested) in MODES.into_iter().enumerate() {
            let mut holder = db.pool.begin().await.unwrap();
            let holder_pid = backend_pid(&mut holder).await;
            one(held, &mut holder, 2).await.unwrap();
            let mut waiter = db.pool.begin().await.unwrap();
            let waiter_pid = backend_pid(&mut waiter).await;
            let mut task = tokio::spawn(async move {
                let result = one(requested, &mut waiter, 2).await;
                (result, waiter)
            });
            if conflicts[held_index][requested_index] {
                db.wait_for_block(holder_pid, waiter_pid).await;
                if (held_index + requested_index) % 2 == 0 {
                    holder.commit().await.unwrap();
                } else {
                    holder.rollback().await.unwrap();
                }
                let (result, waiter) = timeout(Duration::from_secs(3), &mut task)
                    .await
                    .unwrap()
                    .unwrap();
                assert_eq!(result.unwrap().id, 2, "{held:?} -> {requested:?}");
                waiter.rollback().await.unwrap();
            } else {
                let (result, waiter) = timeout(Duration::from_secs(2), &mut task)
                    .await
                    .unwrap_or_else(|_| {
                        task.abort();
                        panic!("{held:?} must allow {requested:?}")
                    })
                    .unwrap();
                assert_eq!(result.unwrap().id, 2);
                waiter.rollback().await.unwrap();
                holder.rollback().await.unwrap();
            }
        }
    }
    db.finish().await;
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL pointing to disposable PostgreSQL"]
async fn row_locks_allow_plain_reads_and_unrelated_rows() {
    let db = Database::new().await;
    for mode in MODES {
        let mut holder = db.pool.begin().await.unwrap();
        one(mode, &mut holder, 2).await.unwrap();
        let mut reader = db.pool.begin().await.unwrap();
        assert_eq!(
            timeout(Duration::from_secs(2), Record::plain(&mut reader, &2))
                .await
                .unwrap()
                .unwrap()
                .id,
            2
        );
        assert_eq!(
            timeout(Duration::from_secs(2), Record::one_update(&mut reader, &3))
                .await
                .unwrap()
                .unwrap()
                .id,
            3
        );
        reader.rollback().await.unwrap();
        holder.rollback().await.unwrap();
    }
    db.finish().await;
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL pointing to disposable PostgreSQL"]
async fn collection_getters_lock_every_returned_row() {
    let db = Database::new().await;
    for mode in MODES {
        for whole_table in [false, true] {
            let mut holder = db.pool.begin().await.unwrap();
            let records = if whole_table {
                all(mode, &mut holder).await
            } else {
                many(mode, &mut holder, 10).await
            }
            .unwrap();
            for record in records {
                let mut probe = db.pool.begin().await.unwrap();
                let error = sqlx::query!(
                    "SELECT record_id FROM locking_records WHERE record_id = $1 AND position = $2 FOR UPDATE NOWAIT",
                    record.id, record.position
                ).fetch_one(&mut *probe).await.unwrap_err();
                assert_eq!(
                    error
                        .as_database_error()
                        .and_then(|error| error.code())
                        .as_deref(),
                    Some("55P03")
                );
                probe.rollback().await.unwrap();
            }
            if !whole_table {
                let mut probe = db.pool.begin().await.unwrap();
                timeout(Duration::from_secs(2), Record::one_update(&mut probe, &3))
                    .await
                    .unwrap()
                    .unwrap();
                probe.rollback().await.unwrap();
            }
            holder.rollback().await.unwrap();
        }
    }
    db.finish().await;
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL pointing to disposable PostgreSQL"]
async fn waiting_getter_returns_the_committed_row() {
    let mut db = Database::new().await;
    let mut holder = db.pool.begin().await.unwrap();
    let holder_pid = backend_pid(&mut holder).await;
    one(Mode::Update, &mut holder, 2).await.unwrap();
    sqlx::query!("UPDATE locking_records SET label = 'changed' WHERE record_id = 2")
        .execute(&mut *holder)
        .await
        .unwrap();
    let mut waiter = db.pool.begin().await.unwrap();
    let waiter_pid = backend_pid(&mut waiter).await;
    let mut task = tokio::spawn(async move {
        let row = Record::one_share(&mut waiter, &2).await.unwrap();
        waiter.rollback().await.unwrap();
        row
    });
    db.wait_for_block(holder_pid, waiter_pid).await;
    holder.commit().await.unwrap();
    let row = timeout(Duration::from_secs(3), &mut task)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(row.label, "changed!");
    db.finish().await;
}
