// `sqlx::migrate!()` only tracks migration files that existed when it last expanded. Rebuild when
// one is added, or a cached build (Docker's target/ cache) ships the old migration list.
fn main() {
    println!("cargo:rerun-if-changed=migrations");
}
