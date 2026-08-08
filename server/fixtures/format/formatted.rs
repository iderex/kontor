// SPDX-License-Identifier: AGPL-3.0-only
// The one-change neighbour of unformatted.rs, and the leg that stops the
// evidence from being satisfied by a formatter that refuses everything. The
// same function, after rustfmt has been over it, has to pass under exactly the
// same configuration.

pub fn total(values: &[u32]) -> u32 {
    let mut sum = 0;
    for value in values {
        sum += value;
    }
    sum
}
