# Database

---

## `sqlx`

[`sqlx`](https://docs.rs/sqlx/latest/sqlx/) is the database driver. It has two main APIs:

- macros for compile time queries
- functions for runtime queries

### Compile Time vs Run Time

Using the macro API, queries are statically checked during compilation.

```rust
// let pool = <impl sqlx::Executor>;
let user = sqlx::query!("SELECT id, name FROM users WHERE email = $1", email)
    .fetch_one(&pool)
    .await?;

// Typed fields, known at compile time
let name: String = user.name;
```

Using the function API, invalid queries will result in runtime errors.

```rust
// let pool = <impl sqlx::Executor>;
let user = sqlx::query("SELECT id, name FROM users WHERE email = $1")
    .bind(email)
    .fetch_one(&pool)
    .await?;

// Untyped `PgRow`, column and type checked at runtime
let name: String = user.try_get("name")?;
```

### Macro API

The macros check every query against the database schema at compile time. SQL syntax, table, and column names, and the number and types of parameters are all validated by the build. Output types are inferred from the schema, including nullability: a nullable column becomes an `Option`, so a missing `NULL` check cannot compile.

```rust
pub struct User {
    pub id: Uuid,
    pub name: String,
    /// Nullable column, so must be `Option`.
    pub google_id: Option<String>,
}

let user = sqlx::query_as!(
    User,
    "SELECT id, name, google_id FROM users WHERE email = $1",
    email
)
.fetch_optional(&pool)
.await?;
```

No `FromRow` derive is needed. If a column is renamed, dropped, or changes type in a migration, every query using it fails to build, rather than failing at runtime. Tests are still needed for behaviour, but not to catch typos or schema drift.

The cost is that compilation needs the schema. The macros can be configured to not require a live database connection for compilation, but it requires a couple extra steps:

1. `cargo install sqlx-cli`
2. Set `DATABASE_URL`
3. Run database server
4. `cargo sqlx prepare`
5. `git commit` `.sqlx/`

Project can now be built without a database connection (`DATABASE_URL` must be unset, otherwise it will still try to connect). To update the generated file:

```bash
cargo sqlx prepare
```

To ensure `.sqlx/` is kept up-to-date, both with the queries in project and database schema itself, run `cargo install sqlx-cli && cargo sqlx prepare --check` in CI.

### Function API

The functions need no extra steps. Project builds without `sqlx-cli`, `DATABASE_URL`, a running database, or a committed `.sqlx/`, so there is no offline mode to keep up-to-date.

The cost is that nothing is checked at compile time. SQL syntax, column names, and types are only validated when the query runs. Rows are decoded into types with `#[derive(sqlx::FromRow)]`, which must be kept in sync with the schema by hand:

```rust
#[derive(sqlx::FromRow)]
struct User {
    id: Uuid,
    name: String,
}

let user: Option<User> = sqlx::query_as("SELECT id, name FROM users WHERE email = $1")
    .bind(email)
    .fetch_optional(&pool)
    .await?;
```

A typo in the SQL, a renamed column, or a `.bind` of the wrong type all compile, then fail at runtime. To catch these before release, every query should be exercised by a `#[sqlx::test]`, and CI must run the tests against a live database.
