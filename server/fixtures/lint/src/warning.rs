// Refused because server/Cargo.toml sets `warnings = "deny"`.
//
// unused_variables is a warning in every Rust toolchain and is fatal only
// because of that line. This arm is the one that says the level was raised,
// which is a different statement from the one the other two arms make: those
// are about which lints are on, this is about what happens when one fires.

/// Doubles a value, having bound a name it never reads.
#[must_use]
pub fn labelled(value: u32) -> String {
    let scaled = value.saturating_mul(2);
    format!("{value}")
}
