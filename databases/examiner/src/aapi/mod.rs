//! Schema `aapi`: aAPI's tables.
mod candidates;
mod sessions;

pub use candidates::{Candidate, NewCandidate};
pub use sessions::CandidateSession;
