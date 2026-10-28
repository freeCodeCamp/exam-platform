//! Schema `edd`: eDd's tables. aAPI has no access.
mod sessions;
mod users;

pub use sessions::Session;
pub use users::{NewUser, User};
