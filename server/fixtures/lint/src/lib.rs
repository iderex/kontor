// The one-change neighbours are always compiled; the arms that must be refused
// are behind features that are off unless ./prove-quality asks for one.
//
// So `cargo build`, `cargo clippy --workspace` and the build workflow all see a
// crate holding only the neighbours, and a run of this crate with no feature is
// the leg that says the gate does not refuse everything put in front of it.

pub mod neighbours;

#[cfg(feature = "unwrap")]
pub mod unwrap;

#[cfg(feature = "warning")]
pub mod warning;

#[cfg(feature = "suppression")]
pub mod suppression;
