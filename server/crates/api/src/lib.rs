//! The API layer. Parses, authenticates, authorises and calls down.
//!
//! It holds no business rule of its own, and nothing depends on it. A rule that
//! appears here is a rule the record module cannot enforce, which means
//! something reaching the record module by another route can break it. Empty.
//! M3 fills it.
