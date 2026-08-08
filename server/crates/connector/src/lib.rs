// SPDX-License-Identifier: AGPL-3.0-only
//! The connector boundary, where a machine refuses a connector that has not
//! declared what crosses it.
//!
//! `docs/decisions/0011-connectors.md` is the decision this implements, and it
//! is where every rule below is argued. The short form is that a connector is a
//! hole in the sentence this project is sold on, so each one is a deliberate act
//! by the operator, bounded and disclosed, and the disclosure is a declaration
//! in the tree rather than a paragraph somebody remembers to write.
//!
//! The declaration lives in the connector crate's own `Cargo.toml`, under
//! `[package.metadata.connector]`. It sits there rather than in a file of its
//! own for two reasons. It is next to the dependency list that authorises the
//! reaching, so a connector that grows a second destination and a second client
//! crate cannot move one without seeing the other. And cargo already carries it:
//! `cargo metadata` prints the table, so the operator facing documentation of
//! #100 and #106 can be generated from what the build already resolves rather
//! than from a register somebody keeps in step by hand.
//!
//! What this crate does not do is read TOML. [`judge`] reads a closed line
//! shape, `key = "value"`, and refuses anything else in the table by name. That
//! is a real bound and it is stated rather than left to be discovered: a
//! declaration written with a multi line string, an inline table or an escaped
//! quote is valid TOML and is refused here. It fails closed, which is the
//! direction a boundary check has to fail in, and the message says which line
//! it could not read.
//!
//! Nothing here reaches a network, a database, a display or a clock. It is a
//! function over the text of a manifest, so the suite that proves it is the
//! default one.

use core::fmt;

/// The prefix that makes a workspace crate a connector.
///
/// A connector is identified by its name because the check has to be able to
/// ask the question of a crate that has declared nothing, and a crate that has
/// declared nothing has only its name. The residual is stated in
/// `docs/decisions/0011-connectors.md`: a connector crate named something else
/// is not seen by this check, and a reader is what catches it.
pub const CRATE_PREFIX: &str = "kontor-connector-";

/// The table a declaration lives in, as it appears in the manifest.
pub const TABLE: &str = "[package.metadata.connector]";

/// Every key a declaration carries, all of them required.
///
/// The set is closed, so a connector cannot answer a question nobody asked
/// instead of the ones that were asked. `outbound` and `inbound` take the
/// literal `none` where nothing crosses in that direction, which is an answer
/// rather than an absence.
pub const KEYS: [&str; 6] = [
    "id",
    "purpose",
    "destination",
    "outbound",
    "inbound",
    "credential",
];

/// Which rule a manifest broke.
///
/// One variant per rule, and the name is what a refusal quotes, following
/// `kontor_metadata::Rule`. A rule that cannot be named is a rule nobody can
/// look up.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rule {
    /// A crate named as a connector, carrying no declaration.
    DeclarationMissing,
    /// A declaration in a crate that is not named as a connector.
    DeclarationWithoutAConnector,
    /// A declaration with one of the required keys absent.
    KeyMissing,
    /// A key that is not one of the ones asked for.
    KeyUnknown,
    /// One key written twice.
    KeyRepeated,
    /// A key present, carrying nothing.
    ValueBlank,
    /// A declaration whose identifier is not the crate it sits in.
    IdDoesNotMatchTheCrate,
    /// A line in the table this check cannot read.
    LineNotADeclaration,
}

impl Rule {
    /// The name a refusal quotes, and the name to look the rule up under.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::DeclarationMissing => "declaration-missing",
            Self::DeclarationWithoutAConnector => "declaration-without-a-connector",
            Self::KeyMissing => "key-missing",
            Self::KeyUnknown => "key-unknown",
            Self::KeyRepeated => "key-repeated",
            Self::ValueBlank => "value-blank",
            Self::IdDoesNotMatchTheCrate => "id-does-not-match-the-crate",
            Self::LineNotADeclaration => "line-not-a-declaration",
        }
    }
}

/// One refusal, naming the rule, what was judged, and what was wrong with it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Refusal {
    /// The rule that was broken.
    pub rule: Rule,
    /// The manifest the rule was broken in.
    pub subject: String,
    /// What was wrong, in terms that name the repair.
    pub detail: String,
}

impl fmt::Display for Refusal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{}: {}: {}",
            self.rule.name(),
            self.subject,
            self.detail
        )
    }
}

/// One manifest as the check reads it: where it is, and what is in it.
///
/// The check takes the text rather than the path so that the rules are a
/// function of bytes and the walk over the tree is somebody else's job. That is
/// what lets the proof judge fixtures it wrote in memory instead of judging the
/// state of this repository on the day it ran.
#[derive(Debug, Clone)]
pub struct Manifest {
    /// Where the manifest is, as a refusal should name it.
    pub path: String,
    /// The whole manifest.
    pub text: String,
}

/// Every refusal the manifests earn, in the order they were given.
///
/// An empty result is the whole of what passing means. A manifest with no
/// `[package]` name is passed over rather than refused: the workspace root has
/// none, and inventing a rule about it here would be a rule about cargo rather
/// than about the boundary.
#[must_use]
pub fn judge(manifests: &[Manifest]) -> Vec<Refusal> {
    let mut refusals = Vec::new();

    for manifest in manifests {
        let Some(name) = package_name(&manifest.text) else {
            continue;
        };
        let declaration = declaration(&manifest.text);
        let is_connector = name.starts_with(CRATE_PREFIX);

        match (is_connector, declaration) {
            (true, None) => refusals.push(Refusal {
                rule: Rule::DeclarationMissing,
                subject: manifest.path.clone(),
                detail: format!(
                    "{name} is named as a connector and carries no {TABLE}. \
                     A connector that has not said where it reaches and what crosses \
                     is one nothing can disclose to an operator."
                ),
            }),
            (false, Some(_)) => refusals.push(Refusal {
                rule: Rule::DeclarationWithoutAConnector,
                subject: manifest.path.clone(),
                detail: format!(
                    "{name} carries {TABLE} and is not named {CRATE_PREFIX}something. \
                     Either it reaches outside and its name has to say so, or the table \
                     is left over and has to go."
                ),
            }),
            (true, Some(lines)) => keys(&manifest.path, name, &lines, &mut refusals),
            (false, None) => {}
        }
    }

    refusals
}

/// The name in the `[package]` table, or nothing where there is no such table.
fn package_name(text: &str) -> Option<&str> {
    let mut inside = false;
    for line in text.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            inside = line == "[package]";
            continue;
        }
        if !inside {
            continue;
        }
        let Some((key, rest)) = line.split_once('=') else {
            continue;
        };
        if key.trim() != "name" {
            continue;
        }
        return rest
            .trim()
            .strip_prefix('"')
            .and_then(|value| value.strip_suffix('"'));
    }
    None
}

/// The lines of the declaration table, with their numbers, or nothing where the
/// table is absent.
///
/// The table ends where the next one begins, which is how TOML ends a table and
/// therefore where a reader of this file will expect it to end.
fn declaration(text: &str) -> Option<Vec<(usize, &str)>> {
    let mut lines = Vec::new();
    let mut inside = false;
    for (number, line) in text.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            if inside {
                break;
            }
            inside = trimmed == TABLE;
            continue;
        }
        if inside {
            lines.push((number + 1, trimmed));
        }
    }
    if inside { Some(lines) } else { None }
}

/// Judge the body of one declaration.
fn keys(path: &str, name: &str, lines: &[(usize, &str)], refusals: &mut Vec<Refusal>) {
    let mut seen: Vec<&str> = Vec::new();

    for &(number, line) in lines {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let Some(value) = read(line) else {
            refusals.push(Refusal {
                rule: Rule::LineNotADeclaration,
                subject: path.to_owned(),
                detail: format!(
                    "line {number} is not `key = \"value\"`, and this check reads no other \
                     shape. Rewrite it as one line per key with the value in one pair of \
                     quotes."
                ),
            });
            continue;
        };
        let (key, value) = value;

        if !KEYS.contains(&key) {
            refusals.push(Refusal {
                rule: Rule::KeyUnknown,
                subject: path.to_owned(),
                detail: format!(
                    "line {number} declares `{key}`, which is not one of the keys asked for. \
                     The set is closed so that a connector answers the questions that were \
                     asked rather than the ones it prefers."
                ),
            });
            continue;
        }

        if seen.contains(&key) {
            refusals.push(Refusal {
                rule: Rule::KeyRepeated,
                subject: path.to_owned(),
                detail: format!(
                    "`{key}` is declared twice, the second time on line {number}, so which \
                     of the two an operator is told depends on which reader they used."
                ),
            });
            continue;
        }
        seen.push(key);

        if value.trim().is_empty() {
            refusals.push(Refusal {
                rule: Rule::ValueBlank,
                subject: path.to_owned(),
                detail: format!(
                    "`{key}` on line {number} carries nothing. A key present and empty reads \
                     as answered and says less than one that is missing."
                ),
            });
            continue;
        }

        if key == "id" {
            let expected = name.strip_prefix(CRATE_PREFIX).unwrap_or(name);
            if value.trim() != expected {
                refusals.push(Refusal {
                    rule: Rule::IdDoesNotMatchTheCrate,
                    subject: path.to_owned(),
                    detail: format!(
                        "`id` on line {number} is `{}` and the crate is `{name}`, which is \
                         `{expected}` with the prefix off. A declaration filed under a name \
                         that is not the crate's is a declaration nobody finds from the crate.",
                        value.trim()
                    ),
                });
            }
        }
    }

    for key in KEYS {
        if !seen.contains(&key) {
            refusals.push(Refusal {
                rule: Rule::KeyMissing,
                subject: path.to_owned(),
                detail: format!(
                    "`{key}` is not declared. Every key is required, because a boundary \
                     described in part is one an operator cannot act on."
                ),
            });
        }
    }
}

/// A key and its value out of one line, or nothing where the line is not the
/// one shape this check reads.
///
/// A value holding a quote of its own is refused with the rest, rather than
/// guessed at. Escaping is where a reader of a subset and a reader of the whole
/// language stop agreeing, and the two disagreeing silently is worse than a
/// refusal naming the line.
fn read(line: &str) -> Option<(&str, &str)> {
    let (key, rest) = line.split_once('=')?;
    let value = rest.trim().strip_prefix('"')?.strip_suffix('"')?;
    if value.contains('"') {
        return None;
    }
    Some((key.trim(), value))
}
