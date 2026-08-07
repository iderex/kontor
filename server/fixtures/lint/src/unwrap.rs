// Refused by clippy::unwrap_used, which server/Cargo.toml denies.
//
// The mistake is the one everybody makes under time pressure. It compiles, it
// is correct whenever the slice is not empty, and the day it is empty the
// operator's log says a thread panicked and nothing about which invariant
// broke. neighbours::first is the same read written the way the rule asks for.

/// Reads the first element and panics when there is none.
///
/// # Panics
///
/// When `values` is empty.
#[must_use]
pub fn first(values: &[u32]) -> u32 {
    values.first().copied().unwrap()
}
