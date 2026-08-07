// Refused by clippy::allow_attributes_without_reason, which server/Cargo.toml
// denies.
//
// The near miss, and the reason this arm exists rather than an `#[allow]` one.
// Somebody who has read the rule reaches for `#[expect]`, which is the spelling
// the configuration asks for, and stops there. The attribute names its lint and
// says nothing about why, so a later reader can see what was silenced and not
// whether it should still be. neighbours::unused_helper is the same suppression
// with the missing half.

#[expect(dead_code)]
fn unused_helper() {}
