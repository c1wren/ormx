//! PostgreSQL entity methods generated with SQLx compile-time query checks.
//!
//! Locking getters are explicitly requested using `get_one_for_<mode>`,
//! `get_optional_for_<mode>`, or `get_many_for_<mode>` on a field, and
//! `get_all_for_<mode>` on the entity. Modes are `update`, `no_key_update`,
//! `share`, and `key_share`. Attributes can be bare or specify a method name.
//!
//! ```rust,ignore
//! #[derive(ormx::Entity)]
//! #[ormx(table = "users")]
//! pub struct User {
//!     #[ormx(key, get_one_for_update = "with_id_for_update")]
//!     pub id: i32,
//!     pub name: String,
//! }
//!
//! let mut tx = pool.begin().await?;
//! let user = User::with_id_for_update(&mut tx, &id).await?;
//! tx.commit().await?;
//! ```
//!
//! Locking methods require `&mut sqlx::Transaction<'_, sqlx::Postgres>` and
//! retain ordinary getters' return and error types. Collection getters order
//! by physical key columns, ascending in declaration order. Callers own the
//! transaction lifetime, cross-table lock ordering, and business validation.
//! Row locks protect existing rows; absent rows and new matches are unprotected.

pub use macros::Entity;
