// SPDX-License-Identifier: AGPL-3.0-only
//! A definition written to a file and read back, exactly.
//!
//! #17 asks that definitions round trip to a file format, so that a schema can
//! be reviewed in a pull request and applied to another instance. Both halves
//! of that sentence are about a reader: a diff somebody has to be able to argue
//! with, and a file that means the same thing on the instance it is applied to
//! as on the one it came from.
//!
//! The format is one key and one value per line, and `docs/schema-file.md` is
//! where the grammar is stated for the operator who has to write one. Nothing
//! here restates it; what is written here is why the shape is what it is and
//! where it stops.
//!
//! **The round trip is total, and that is the property to hold on to.** Writing
//! any [`ObjectDefinition`] and reading the result back returns a value equal to
//! the one written, whatever it holds, including a definition
//! [`ObjectDefinition::check`] would refuse. That is deliberate: a schema is
//! reviewed before it is applied, and a format that could only carry a valid
//! definition could not carry the one somebody is asking for help with.
//!
//! **Reading refuses the shape of the text and never the meaning of it.** The
//! rules a definition has to satisfy live in [`crate::definition`] and are not
//! repeated here, because a second copy of them is a statement that drifts.
//! Applying a file is two steps rather than one: [`read`] it, then
//! [`ObjectDefinition::check`] what came back. A caller that skips the second
//! step has skipped it visibly.
//!
//! **Every value is escaped and the escapes are five.** A label, a picklist
//! value and an identifier are operator text and may hold anything, including
//! the line ending that would otherwise end the line they are written on. The
//! escapes are the only place this format could silently mangle a value, which
//! is why there are five rather than however many a reader might guess at, and
//! why an escape outside the five is refused rather than passed through.

use crate::definition::{FieldDefinition, ObjectDefinition};
use crate::field_type::{FIELD_TYPES, FieldType};
use core::fmt;

/// Which rule a file broke.
///
/// One variant per rule, and the name is what a refusal quotes, which is the
/// convention [`crate::definition::Rule`] already sets: a rule that cannot be
/// named is a rule an operator cannot look up.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rule {
    /// A word at the start of a line that is not one of this format's keys.
    UnknownKeyword,
    /// A key that is not allowed where it appears.
    KeywordOutOfPlace,
    /// A second `object` line.
    ObjectRepeated,
    /// A file that opens no object.
    ObjectMissing,
    /// A key given twice inside one block, where the block takes it once.
    KeyRepeated,
    /// A key the block has to carry and does not.
    KeyMissing,
    /// A type this build does not have.
    TypeUnknown,
    /// A `required` line answering with something other than yes or no.
    NotYesOrNo,
    /// A backslash followed by a character that is not one of the five escapes.
    EscapeUnknown,
    /// A line ending on a backslash that escapes nothing.
    EscapeUnfinished,
    /// A line starting or ending in a space or a tab.
    LooseWhitespace,
}

impl Rule {
    /// The name a refusal quotes, and the name to look the rule up under.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::UnknownKeyword => "unknown-keyword",
            Self::KeywordOutOfPlace => "keyword-out-of-place",
            Self::ObjectRepeated => "object-repeated",
            Self::ObjectMissing => "object-missing",
            Self::KeyRepeated => "key-repeated",
            Self::KeyMissing => "key-missing",
            Self::TypeUnknown => "type-unknown",
            Self::NotYesOrNo => "not-yes-or-no",
            Self::EscapeUnknown => "escape-unknown",
            Self::EscapeUnfinished => "escape-unfinished",
            Self::LooseWhitespace => "loose-whitespace",
        }
    }
}

/// One thing wrong with a file: which rule, where, and what a reader needs to
/// fix it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Refusal {
    /// The rule that was broken.
    pub rule: Rule,
    /// The line it was broken on, counting from one, and zero where the trouble
    /// is the file rather than any line in it.
    pub line: usize,
    /// What a reader needs beyond the rule name.
    pub detail: String,
}

impl fmt::Display for Refusal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.line == 0 {
            write!(formatter, "the file: {}: {}", self.rule.name(), self.detail)
        } else {
            write!(
                formatter,
                "line {}: {}: {}",
                self.line,
                self.rule.name(),
                self.detail
            )
        }
    }
}

/// The definition as a file.
///
/// What comes out reads back as an equal definition through [`read`], for every
/// definition, which the round trip test in this crate's suite is what says
/// rather than this sentence.
#[must_use]
pub fn write(definition: &ObjectDefinition) -> String {
    let mut text = String::new();

    write_line(&mut text, "object", &definition.name);
    write_line(&mut text, "label", &definition.label);

    for field in &definition.fields {
        // A blank line between fields, because the unit a reviewer reads is one
        // field and a hundred lines with no break in them is a diff nobody
        // reads to the end of.
        text.push('\n');
        write_line(&mut text, "field", &field.name);
        write_line(&mut text, "label", &field.label);
        write_line(&mut text, "type", field.field_type.spec().written_as);
        write_line(
            &mut text,
            "required",
            if field.required { "yes" } else { "no" },
        );
        for value in &field.values {
            write_line(&mut text, "value", value);
        }
        if let Some(named) = &field.names_object {
            write_line(&mut text, "references", named);
        }
    }

    text
}

/// The definition the file holds, or every rule it broke.
///
/// Every rule rather than the first, for the reason
/// [`ObjectDefinition::check`] gives: an operator fixing a file one refusal at
/// a time makes one round trip per mistake.
///
/// What comes back is not checked. A definition read here can still be one that
/// nothing could realise, and the caller applying it runs
/// [`ObjectDefinition::check`] on it.
///
/// # Errors
///
/// The refusals, in the order the lines were read and then the order the blocks
/// were opened, when the file cannot be read as a definition.
pub fn read(text: &str) -> Result<ObjectDefinition, Vec<Refusal>> {
    let mut refusals = Vec::new();
    let mut object: Option<Opened> = None;
    let mut fields: Vec<PartialField> = Vec::new();

    for (index, raw) in text.lines().enumerate() {
        let line = index + 1;

        if raw.is_empty() {
            continue;
        }

        if starts_or_ends_loose(raw) {
            refusals.push(Refusal {
                rule: Rule::LooseWhitespace,
                line,
                detail: LOOSE_WHITESPACE.to_owned(),
            });
            continue;
        }

        if raw.starts_with('#') {
            continue;
        }

        let (keyword, written) = split_off_keyword(raw);
        let value = unescape(written, line, &mut refusals);

        match keyword {
            "object" => open_object(&mut object, &mut refusals, line, value),
            "field" => open_field(&mut fields, object.as_ref(), &mut refusals, line, value),
            "label" => take_label(object.as_mut(), fields.last_mut(), &mut refusals, line, value),
            "type" => take_type(fields.last_mut(), &mut refusals, line, &value),
            "required" => take_required(fields.last_mut(), &mut refusals, line, &value),
            "value" => take_value(fields.last_mut(), &mut refusals, line, value),
            "references" => take_references(fields.last_mut(), &mut refusals, line, value),
            other => refusals.push(Refusal {
                rule: Rule::UnknownKeyword,
                line,
                detail: format!(
                    "\"{other}\" is not a key of this format. The keys are object, label, field, type, required, value and references."
                ),
            }),
        }
    }

    assemble(object, fields, refusals)
}

const LOOSE_WHITESPACE: &str = "the line starts or ends in a space or a tab. A value is everything after the first space, so loose whitespace would join it without showing on the screen. A space at either end of a value is written \\s.";

/// An object line that has been read, and what it has been given since.
struct Opened {
    line: usize,
    name: String,
    label: Option<String>,
}

/// A field block that has been opened, and what it has been given since.
struct PartialField {
    line: usize,
    name: String,
    label: Option<String>,
    field_type: Option<FieldType>,
    required: Option<bool>,
    values: Vec<String>,
    names_object: Option<String>,
    /// Whether a `type` line appeared at all, which is not the same question as
    /// whether a type was read from one. A field whose type is a word this
    /// build does not have is refused for that, and telling the operator in the
    /// same breath that the field carries no type line would name a line they
    /// are looking at.
    saw_type: bool,
    /// The same distinction for `required`.
    saw_required: bool,
}

impl PartialField {
    fn opened(line: usize, name: String) -> Self {
        Self {
            line,
            name,
            label: None,
            field_type: None,
            required: None,
            values: Vec::new(),
            names_object: None,
            saw_type: false,
            saw_required: false,
        }
    }
}

/// The definition, or the refusals gathered so far plus the ones only the whole
/// file can produce: a file that opened no object, and a block missing a key it
/// has to carry.
fn assemble(
    object: Option<Opened>,
    fields: Vec<PartialField>,
    mut refusals: Vec<Refusal>,
) -> Result<ObjectDefinition, Vec<Refusal>> {
    let Some(opened) = object else {
        refusals.push(Refusal {
            rule: Rule::ObjectMissing,
            line: 0,
            detail: "no object line, so this file says nothing about what is being defined. A schema file opens with one.".to_owned(),
        });
        return Err(refusals);
    };

    let label = require(
        &mut refusals,
        opened.label,
        false,
        opened.line,
        "object",
        "label",
    );
    let mut read_fields = Vec::with_capacity(fields.len());

    for field in fields {
        let label = require(
            &mut refusals,
            field.label,
            false,
            field.line,
            "field",
            "label",
        );
        let field_type = require(
            &mut refusals,
            field.field_type,
            field.saw_type,
            field.line,
            "field",
            "type",
        );
        let required = require(
            &mut refusals,
            field.required,
            field.saw_required,
            field.line,
            "field",
            "required",
        );

        if let (Some(label), Some(field_type), Some(required)) = (label, field_type, required) {
            read_fields.push(FieldDefinition {
                name: field.name,
                label,
                field_type,
                required,
                values: field.values,
                names_object: field.names_object,
            });
        }
    }

    match label {
        Some(label) if refusals.is_empty() => Ok(ObjectDefinition {
            name: opened.name,
            label,
            fields: read_fields,
        }),
        _ => Err(refusals),
    }
}

/// The value, or a `key-missing` refusal naming the block it was owed to.
///
/// `seen` is what stops a line that was refused for its value being reported a
/// second time as a line that is not there. A value refused once is one repair;
/// the same value refused twice under two rules is an operator looking for a
/// second mistake that does not exist.
fn require<T>(
    refusals: &mut Vec<Refusal>,
    given: Option<T>,
    seen: bool,
    line: usize,
    block: &str,
    key: &str,
) -> Option<T> {
    if given.is_none() && !seen {
        refusals.push(Refusal {
            rule: Rule::KeyMissing,
            line,
            detail: format!(
                "the {block} opened here carries no {key} line, and a {block} without one is not a definition anything could apply."
            ),
        });
    }

    given
}

fn open_object(
    object: &mut Option<Opened>,
    refusals: &mut Vec<Refusal>,
    line: usize,
    name: String,
) {
    if let Some(already) = object.as_ref() {
        refusals.push(Refusal {
            rule: Rule::ObjectRepeated,
            line,
            detail: format!(
                "\"{}\" was opened on line {} and this file opens a second object. One file is one object, so that a review of the file is a review of one thing.",
                already.name, already.line
            ),
        });
        return;
    }

    *object = Some(Opened {
        line,
        name,
        label: None,
    });
}

fn open_field(
    fields: &mut Vec<PartialField>,
    object: Option<&Opened>,
    refusals: &mut Vec<Refusal>,
    line: usize,
    name: String,
) {
    if object.is_none() {
        refusals.push(out_of_place(
            line,
            "field",
            "no object is open yet, and a field belongs to one. The first key in a file is object.",
        ));
        return;
    }

    fields.push(PartialField::opened(line, name));
}

/// A label belongs to whichever block is open, which is the last field where
/// there is one and the object otherwise.
fn take_label(
    object: Option<&mut Opened>,
    field: Option<&mut PartialField>,
    refusals: &mut Vec<Refusal>,
    line: usize,
    value: String,
) {
    if let Some(field) = field {
        replace_once(&mut field.label, value, refusals, line, "label");
        return;
    }

    match object {
        Some(object) => replace_once(&mut object.label, value, refusals, line, "label"),
        None => refusals.push(out_of_place(
            line,
            "label",
            "no object is open yet, so there is nothing for this label to name. The first key in a file is object.",
        )),
    }
}

fn take_type(
    field: Option<&mut PartialField>,
    refusals: &mut Vec<Refusal>,
    line: usize,
    written: &str,
) {
    let Some(field) = field else {
        refusals.push(out_of_place(
            line,
            "type",
            "no field is open, and only a field has a type.",
        ));
        return;
    };

    field.saw_type = true;

    let Some(field_type) = FieldType::from_written(written) else {
        let known: Vec<&str> = FIELD_TYPES.iter().map(|spec| spec.written_as).collect();
        refusals.push(Refusal {
            rule: Rule::TypeUnknown,
            line,
            detail: format!(
                "\"{written}\" is no type this build has. The set is closed and it is {}.",
                known.join(", ")
            ),
        });
        return;
    };

    replace_once(&mut field.field_type, field_type, refusals, line, "type");
}

fn take_required(
    field: Option<&mut PartialField>,
    refusals: &mut Vec<Refusal>,
    line: usize,
    written: &str,
) {
    let Some(field) = field else {
        refusals.push(out_of_place(
            line,
            "required",
            "no field is open, and only a field is required or not.",
        ));
        return;
    };

    field.saw_required = true;

    let answer = match written {
        "yes" => true,
        "no" => false,
        other => {
            refusals.push(Refusal {
                rule: Rule::NotYesOrNo,
                line,
                detail: format!(
                    "\"{other}\" is neither yes nor no, which are the whole of what this line takes. Every other spelling of true and false has a system somewhere that reads it the other way round."
                ),
            });
            return;
        }
    };

    replace_once(&mut field.required, answer, refusals, line, "required");
}

fn take_value(
    field: Option<&mut PartialField>,
    refusals: &mut Vec<Refusal>,
    line: usize,
    value: String,
) {
    match field {
        Some(field) => field.values.push(value),
        None => refusals.push(out_of_place(
            line,
            "value",
            "no field is open, and a value belongs to the field that offers it.",
        )),
    }
}

fn take_references(
    field: Option<&mut PartialField>,
    refusals: &mut Vec<Refusal>,
    line: usize,
    value: String,
) {
    match field {
        Some(field) => replace_once(&mut field.names_object, value, refusals, line, "references"),
        None => refusals.push(out_of_place(
            line,
            "references",
            "no field is open, and it is a field that points at a record of another object.",
        )),
    }
}

/// Take a key the block carries once, and refuse the second rather than letting
/// it overwrite the first. Which of two values an operator meant is not
/// knowable, and taking the later one quietly is the mangling this format
/// exists against.
fn replace_once<T>(
    slot: &mut Option<T>,
    value: T,
    refusals: &mut Vec<Refusal>,
    line: usize,
    key: &str,
) {
    if slot.is_some() {
        refusals.push(Refusal {
            rule: Rule::KeyRepeated,
            line,
            detail: format!(
                "a second {key} line for the block this belongs to. The block takes one, and which of the two was meant is not something a reader could work out."
            ),
        });
        return;
    }

    *slot = Some(value);
}

fn out_of_place(line: usize, key: &str, why: &str) -> Refusal {
    Refusal {
        rule: Rule::KeywordOutOfPlace,
        line,
        detail: format!("{key} is not allowed here: {why}"),
    }
}

/// Whether the line begins or ends in a space or a tab.
fn starts_or_ends_loose(raw: &str) -> bool {
    let loose = |character: char| character == ' ' || character == '\t';
    raw.starts_with(loose) || raw.ends_with(loose)
}

/// The key and what follows it, which is everything after one space and is
/// empty where the line is the key alone.
fn split_off_keyword(raw: &str) -> (&str, &str) {
    match raw.find(' ') {
        Some(position) => (&raw[..position], &raw[position + 1..]),
        None => (raw, ""),
    }
}

/// One line of the file, which is the key alone where the value is empty, so
/// that nothing this writes ends in the space the reader refuses.
fn write_line(text: &mut String, keyword: &str, value: &str) {
    let written = escape(value);

    text.push_str(keyword);
    if !written.is_empty() {
        text.push(' ');
        text.push_str(&written);
    }
    text.push('\n');
}

/// The five escapes, written.
///
/// A leading or trailing space is written as `\s` and an interior one is left
/// alone, because a line's own edges are the only place a space is lost, and
/// escaping every space would make the ordinary label unreadable.
fn escape(value: &str) -> String {
    let characters: Vec<char> = value.chars().collect();
    let last = characters.len().saturating_sub(1);
    let mut written = String::new();

    for (position, character) in characters.iter().enumerate() {
        match *character {
            '\\' => written.push_str("\\\\"),
            '\n' => written.push_str("\\n"),
            '\r' => written.push_str("\\r"),
            '\t' => written.push_str("\\t"),
            ' ' if position == 0 || position == last => written.push_str("\\s"),
            other => written.push(other),
        }
    }

    written
}

/// The five escapes, read back. An escape outside the five is refused rather
/// than passed through as whatever character followed it, because a format that
/// quietly drops a backslash is one an operator cannot put a path in.
fn unescape(written: &str, line: usize, refusals: &mut Vec<Refusal>) -> String {
    let mut value = String::new();
    let mut characters = written.chars();

    while let Some(character) = characters.next() {
        if character != '\\' {
            value.push(character);
            continue;
        }

        match characters.next() {
            Some('\\') => value.push('\\'),
            Some('n') => value.push('\n'),
            Some('r') => value.push('\r'),
            Some('t') => value.push('\t'),
            Some('s') => value.push(' '),
            Some(other) => refusals.push(Refusal {
                rule: Rule::EscapeUnknown,
                line,
                detail: format!(
                    "\\{other} is not one of the five escapes, which are \\\\, \\n, \\r, \\t and \\s. A backslash that is part of the value is written \\\\."
                ),
            }),
            None => refusals.push(Refusal {
                rule: Rule::EscapeUnfinished,
                line,
                detail: "the line ends on a backslash, which escapes nothing. A backslash that is part of the value is written \\\\.".to_owned(),
            }),
        }
    }

    value
}
