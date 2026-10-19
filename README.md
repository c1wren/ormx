# ORMX

ORMX derives PostgreSQL CRUD methods backed by SQLx compile-time query checks.
See `example/src/main.rs` for inserts, patches, lifecycle hooks, custom types,
conversions, SQL transformations, and all four locking getter shapes and modes.
The executable accepts `DATABASE_URL` and otherwise connects to
`postgres://postgres@127.0.0.1/ormx`. It writes example data, so run it against a
disposable database initialized with `example/schema.sql`.

## Locking getters

Request a locking getter explicitly with an `#[ormx(...)]` attribute:

```rust,ignore
#[derive(ormx::Entity)]
#[ormx(table = "users", get_all_for_key_share)]
pub struct User {
    #[ormx(
        key,
        get_one = "with_id",
        get_one_for_update = "with_id_for_update",
        get_optional_for_share = "maybe_with_id_for_share"
    )]
    pub id: i32,
    pub name: String,
}

let mut tx = pool.begin().await?;
let user = User::with_id_for_update(&mut tx, &user_id).await?;
// Read, validate, and persist while the transaction holds the lock.
tx.commit().await?;
```

Each getter family supports all four suffixes: `for_update`,
`for_no_key_update`, `for_share`, and `for_key_share`.

| Attribute family | Placement | Return value inside `Result` |
| --- | --- | --- |
| `get_one_for_<mode>` | Field | `Self` |
| `get_optional_for_<mode>` | Field | `Option<Self>` |
| `get_many_for_<mode>` | Field | `Vec<Self>` |
| `get_all_for_<mode>` | Entity | `Vec<Self>` |

Bare field attributes generate `get_by_<field>_for_<mode>`; bare entity
attributes generate `get_all_for_<mode>`. An explicit name replaces the entire
method name. Declare different names when requesting multiple getter shapes
for the same field and mode. Each attribute generates only its requested method;
no unlocked getter is required. Duplicate attributes and conflicting generated
method names are rejected.

Locking methods accept `&mut sqlx::Transaction<'_, sqlx::Postgres>` first,
followed by a borrowed field value for field getters. Field getters retain a
single equality condition; `get_many` means all rows matching one value, not
an array of IDs. Use `get_one`/`get_optional` on unique lookups. Existing unlocked
getters continue to accept `&mut sqlx::PgConnection` and keep their current SQL.

The selected model is returned by the locking SELECT itself. Missing rows
produce `sqlx::Error::RowNotFound`, `None`, or an empty vector according to getter
shape. Configured `error_type` conversions, visibility, renamed columns, custom
types, conversions, and transformation parameters work as in ordinary getters.

Transformation parameter providers run once per transformed field and SQL
statement, even when their tuple supplies multiple parameters. An insert with
`RETURNING` evaluates its write and read providers separately.

| Mode | Conflicts with other transactions holding |
| --- | --- |
| `UPDATE` | Any row-lock mode |
| `NO KEY UPDATE` | `UPDATE`, `NO KEY UPDATE`, `SHARE` |
| `SHARE` | `UPDATE`, `NO KEY UPDATE` |
| `KEY SHARE` | `UPDATE` |

Ordinary SELECTs remain permitted. PostgreSQL also acquires these lock modes
for writes; see its [row-lock documentation](https://www.postgresql.org/docs/current/explicit-locking.html#LOCKING-ROWS).
Methods wait normally and propagate database errors without retries. ORMX never
begins or ends the supplied transaction. Locks last until transaction end, or
rollback of the savepoint that acquired them.

Locking collection getters order by the physical `#[ormx(key)]` columns,
ascending in struct declaration order, before the locking clause. This includes
composite keys. Keep key declarations consistent between models of the same
table and keep ordering keys stable during concurrent operations. Callers still
own cross-table lock order, lock upgrades, tenant filters, and business
validation. Row locks protect existing rows; they do not protect absent rows or
prevent new matching rows. Joins, multi-field predicates, `OF`, `NOWAIT`, and
`SKIP LOCKED` require custom SQL.

## Verification

With committed `.sqlx` metadata, normal checks need no database:

```sh
cargo fmt --all -- --check
SQLX_OFFLINE=true cargo test --workspace
```

PostgreSQL integration tests are explicitly ignored in normal runs. To invoke
them, set `TEST_DATABASE_URL` to a disposable database whose user can create and
drop schemas:

```sh
TEST_DATABASE_URL=postgresql://localhost/ormx_test SQLX_OFFLINE=true \
    cargo test -p example --test locking -- --ignored
```

Explicit invocation without `TEST_DATABASE_URL` fails. Each test creates its own
schema and drops it on success. Failed tests can leave schemas for inspection.
The suite checks all 16 lock-mode pairings with PostgreSQL-observed waiters,
collection coverage, normal reads, unrelated rows, transaction-end release,
missing-row behavior, projections, and conversions.

To regenerate all query metadata, create a fresh disposable database and load
the example schemas before preparing all targets:

```sh
export DATABASE_URL=postgresql://localhost/ormx_prepare
psql "$DATABASE_URL" -v ON_ERROR_STOP=1 -f example/schema.sql \
    -f example/tests/fixtures/locking.sql
cargo sqlx prepare --workspace -- --all-targets
```
