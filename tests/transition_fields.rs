//! Downstream-shaped tests for the published transition tracing fields.
//!
//! `tests/transition_fields/contract.rs` pins the exported constants to
//! independently written names and checks the grammar and manifest facts the
//! contract relies on. `tests/transition_fields/downstream.rs` holds the
//! application-shaped fixtures the remaining modules instrument. The suite
//! compiles as an external crate, so it can only use the public `statelet`
//! API.

#[path = "transition_fields/contract.rs"]
mod contract;
#[path = "transition_fields/downstream.rs"]
mod downstream;
