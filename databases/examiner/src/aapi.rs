//! Schema `aapi`: aAPI's tables. eDd reads, never writes, those granted to it.
mod candidates;

pub use candidates::{Candidate, NewCandidate};
