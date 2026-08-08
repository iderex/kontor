// SPDX-License-Identifier: AGPL-3.0-only
// The one-change neighbour of every arm in this crate, written the way the
// shipped configuration asks for. These are compiled by every run, so a change
// that made the lint gate refuse everything would redden the ordinary build
// rather than only the proof.

/// The neighbour of `unwrap.rs`. The same read, with the empty case named
/// instead of panicking with no message.
#[must_use]
pub fn first(values: &[u32]) -> Option<u32> {
    values.first().copied()
}

/// The neighbour of `warning.rs`. The binding is used, so nothing is unused.
#[must_use]
pub fn labelled(value: u32) -> String {
    let scaled = value.saturating_mul(2);
    format!("{scaled}")
}

/// The neighbour of `suppression.rs`. The same suppression, carrying the reason
/// the shipped configuration requires.
#[expect(
    dead_code,
    reason = "the fixture is about the shape of the suppression rather than about the item it covers"
)]
fn unused_helper() {}
