//! Tables only eDd writes. Other apps may read them, e.g. aAPI serving a question's objects.
mod objects;

pub use objects::{InvalidObjectHash, NewObject, Object, ObjectHash, ObjectKey};
