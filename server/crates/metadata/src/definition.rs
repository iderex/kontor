// SPDX-License-Identifier: AGPL-3.0-only
//! An object definition and a field definition, and the rules a definition has
//! to satisfy before anything tries to realise it.
//!
//! Everything here runs before a migration is generated. That is the whole
//! point of it: a definition refused at this layer costs an error message, and
//! the same definition refused by the database costs a migration that has
//! already taken a lock on a table other transactions are using, which
//! `docs/decisions/0003-custom-fields.md` names as the cost of the shape this
//! project chose.
//!
//! A refusal names the rule it broke. Not a sentence about the value, which
//! tells an operator that something is wrong and not which of the rules they
//! were reading is the one they missed.

use crate::field_type::FieldType;
use core::fmt;

/// The most custom fields one object may carry.
///
/// `docs/decisions/0003-custom-fields.md` is where the number is argued: it
/// leaves room under PostgreSQL's column limit for the object's own columns and
/// for five complete turnovers of every custom field before the table has to be
/// rewritten.
pub const FIELDS_PER_OBJECT: usize = 250;

/// The most columns PostgreSQL will hold on one table.
///
/// A dropped column keeps its slot against this until the table is rewritten,
/// which is why a definition is checked against live columns and dead slots
/// separately.
pub const COLUMNS_PER_TABLE: usize = 1600;

/// The longest identifier PostgreSQL keeps.
///
/// It does not refuse a longer one. It truncates it, which is worse: two field
/// names differing only after this point become one column, and the second
/// definition either fails for a reason naming neither field or silently takes
/// the first one's column.
pub const IDENTIFIER_CHARACTERS: usize = 63;

/// Which rule a definition broke.
///
/// One variant per rule, and the name is what a refusal quotes. A rule that
/// cannot be named is a rule an operator cannot look up.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rule {
    /// An identifier that is not a lower case unquoted PostgreSQL identifier.
    NameShape,
    /// An identifier longer than PostgreSQL keeps.
    NameLength,
    /// Two fields on one object with one identifier.
    NameRepeated,
    /// A definition with nothing for a person to read on a screen.
    LabelBlank,
    /// A type that constrains its values, carrying none.
    NoValuesToChooseFrom,
    /// Values on a type that does not constrain its values.
    ValuesWhereTheTypeTakesNone,
    /// A value that is empty or is only spaces.
    ValueBlank,
    /// One value written twice.
    ValueRepeated,
    /// A reference naming no object.
    ReferenceNamesNoObject,
    /// An object named by a type that does not refer to one.
    ObjectNamedWhereTheTypeTakesNone,
    /// More fields than one object may carry.
    FieldCountBound,
    /// More columns than the table has slots for, counting the dead ones.
    ColumnSlotsExhausted,
}

impl Rule {
    /// The name a refusal quotes, and the name to look the rule up under.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::NameShape => "name-shape",
            Self::NameLength => "name-length",
            Self::NameRepeated => "name-repeated",
            Self::LabelBlank => "label-blank",
            Self::NoValuesToChooseFrom => "no-values-to-choose-from",
            Self::ValuesWhereTheTypeTakesNone => "values-where-the-type-takes-none",
            Self::ValueBlank => "value-blank",
            Self::ValueRepeated => "value-repeated",
            Self::ReferenceNamesNoObject => "reference-names-no-object",
            Self::ObjectNamedWhereTheTypeTakesNone => "object-named-where-the-type-takes-none",
            Self::FieldCountBound => "field-count-bound",
            Self::ColumnSlotsExhausted => "column-slots-exhausted",
        }
    }
}

/// One thing wrong with a definition: which rule, what it was about, and what
/// the reader has to know to fix it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Refusal {
    /// The rule that was broken.
    pub rule: Rule,
    /// What it was about, which is a field's identifier where there is one and
    /// the object's otherwise.
    pub subject: String,
    /// What a reader needs beyond the rule name, including every number the
    /// rule is about.
    pub detail: String,
}

impl fmt::Display for Refusal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{}: {}: {}",
            self.subject,
            self.rule.name(),
            self.detail
        )
    }
}

/// What the table this object is realised as already carries.
///
/// Read from the database rather than from a definition, because both numbers
/// are facts about a table at a moment and neither can be derived from what an
/// operator typed. The metadata layer takes them and does not go looking for
/// them; #18 is what supplies them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TableSlots {
    /// The columns the object carries before any custom field.
    pub core_columns: usize,
    /// Slots held by columns that were dropped and have not been returned,
    /// which happens only when the table is rewritten.
    pub dead_column_slots: usize,
}

/// One field of an object.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldDefinition {
    /// The identifier everything else refers to this field by. It is what a
    /// rename may not change, which is #17's clause and is not enforced here
    /// because nothing here performs a rename yet.
    pub name: String,
    /// What a person reads on a screen.
    pub label: String,
    /// The type.
    pub field_type: FieldType,
    /// Whether a record may exist without a value in it.
    pub required: bool,
    /// The values, in the order they are offered and ordered by, for a type
    /// that constrains its values. Empty for every other type.
    pub values: Vec<String>,
    /// The object a reference points at. `None` for every other type.
    pub names_object: Option<String>,
}

/// One object.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectDefinition {
    /// The identifier everything else refers to this object by.
    pub name: String,
    /// What a person reads on a screen.
    pub label: String,
    /// The custom fields. The object's own columns are not here; they are
    /// counted through [`TableSlots::core_columns`].
    pub fields: Vec<FieldDefinition>,
}

impl ObjectDefinition {
    /// Every rule this definition breaks, or nothing.
    ///
    /// Every rule rather than the first, because an operator fixing a
    /// definition one refusal at a time makes one round trip per mistake, and
    /// the round trip here is a migration window.
    ///
    /// # Errors
    ///
    /// The refusals, in the order the rules are checked, when the definition
    /// cannot be realised.
    pub fn check(&self, slots: TableSlots) -> Result<(), Vec<Refusal>> {
        let mut refusals = Vec::new();

        check_name(&self.name, &self.name, &mut refusals);
        check_label(&self.label, &self.name, &mut refusals);

        for field in &self.fields {
            check_field(field, &mut refusals);
        }

        check_names_are_not_repeated(&self.fields, &self.name, &mut refusals);
        check_the_bounds(self, slots, &mut refusals);

        if refusals.is_empty() {
            Ok(())
        } else {
            Err(refusals)
        }
    }

    /// The columns this object's custom fields take, which is not the number of
    /// fields: a money field is an amount and a currency.
    #[must_use]
    pub fn columns(&self) -> usize {
        self.fields
            .iter()
            .map(|field| field.field_type.spec().columns)
            .sum()
    }
}

fn check_field(field: &FieldDefinition, refusals: &mut Vec<Refusal>) {
    check_name(&field.name, &field.name, refusals);
    check_label(&field.label, &field.name, refusals);
    check_values(field, refusals);
    check_named_object(field, refusals);
}

/// A lower case unquoted PostgreSQL identifier, and nothing else.
///
/// Unquoted, because a generated migration full of quoted identifiers is one
/// nobody reads, and an identifier that needs quotes is one whose case has to be
/// carried correctly by every consumer that ever writes SQL by hand.
fn check_name(name: &str, subject: &str, refusals: &mut Vec<Refusal>) {
    let characters = name.chars().count();

    if characters > IDENTIFIER_CHARACTERS {
        refusals.push(Refusal {
            rule: Rule::NameLength,
            subject: subject.to_owned(),
            detail: format!(
                "{characters} characters, and PostgreSQL keeps {IDENTIFIER_CHARACTERS}. It truncates rather than refusing, so two names differing after that point become one column."
            ),
        });
    }

    let shaped = name
        .chars()
        .next()
        .is_some_and(|first| first.is_ascii_lowercase())
        && name.chars().all(|character| {
            character.is_ascii_lowercase() || character.is_ascii_digit() || character == '_'
        })
        && !name.ends_with('_')
        && !name.contains("__");

    if !shaped {
        refusals.push(Refusal {
            rule: Rule::NameShape,
            subject: subject.to_owned(),
            detail: format!(
                "\"{name}\" is not a lower case unquoted identifier. It starts with a letter, carries only letters, digits and single underscores, and does not end with one."
            ),
        });
    }
}

fn check_label(label: &str, subject: &str, refusals: &mut Vec<Refusal>) {
    if label.trim().is_empty() {
        refusals.push(Refusal {
            rule: Rule::LabelBlank,
            subject: subject.to_owned(),
            detail: "there is nothing here for a person to read on a screen.".to_owned(),
        });
    }
}

fn check_values(field: &FieldDefinition, refusals: &mut Vec<Refusal>) {
    if !field.field_type.constrains_its_values() {
        if !field.values.is_empty() {
            refusals.push(Refusal {
                rule: Rule::ValuesWhereTheTypeTakesNone,
                subject: field.name.clone(),
                detail: format!(
                    "{} values are declared and a {} field takes its values from nothing but the type.",
                    field.values.len(),
                    field.field_type
                ),
            });
        }
        return;
    }

    if field.values.is_empty() {
        refusals.push(Refusal {
            rule: Rule::NoValuesToChooseFrom,
            subject: field.name.clone(),
            detail: format!(
                "a {} field offers a choice, and this one offers none, so no value could ever be accepted for it.",
                field.field_type
            ),
        });
        return;
    }

    for (position, value) in field.values.iter().enumerate() {
        if value.trim().is_empty() {
            refusals.push(Refusal {
                rule: Rule::ValueBlank,
                subject: field.name.clone(),
                detail: format!(
                    "value {} is empty or is only spaces, which is indistinguishable from the field being unset.",
                    position + 1
                ),
            });
        }

        if let Some(earlier) = field.values[..position]
            .iter()
            .position(|seen| seen == value)
        {
            refusals.push(Refusal {
                rule: Rule::ValueRepeated,
                subject: field.name.clone(),
                detail: format!(
                    "\"{value}\" is value {} and value {}, and the order values are declared in is the order they sort in, so one of the two would never be reached.",
                    earlier + 1,
                    position + 1
                ),
            });
        }
    }
}

fn check_named_object(field: &FieldDefinition, refusals: &mut Vec<Refusal>) {
    match (field.field_type.names_another_object(), &field.names_object) {
        (true, None) => refusals.push(Refusal {
            rule: Rule::ReferenceNamesNoObject,
            subject: field.name.clone(),
            detail: format!(
                "a {} field points at a record of some object, and this one says which object nowhere.",
                field.field_type
            ),
        }),
        (false, Some(named)) => refusals.push(Refusal {
            rule: Rule::ObjectNamedWhereTheTypeTakesNone,
            subject: field.name.clone(),
            detail: format!(
                "\"{named}\" is named and a {} field points at no record.",
                field.field_type
            ),
        }),
        (true, Some(named)) => check_name(named, &field.name, refusals),
        (false, None) => {}
    }
}

fn check_names_are_not_repeated(
    fields: &[FieldDefinition],
    subject: &str,
    refusals: &mut Vec<Refusal>,
) {
    for (position, field) in fields.iter().enumerate() {
        if let Some(earlier) = fields[..position]
            .iter()
            .position(|seen| seen.name == field.name)
        {
            refusals.push(Refusal {
                rule: Rule::NameRepeated,
                subject: subject.to_owned(),
                detail: format!(
                    "\"{}\" is field {} and field {}, and both would be one column.",
                    field.name,
                    earlier + 1,
                    position + 1
                ),
            });
        }
    }
}

/// The two bounds, and both messages carry all three numbers.
///
/// `docs/decisions/0003-custom-fields.md` asks for that by name: a message
/// saying only that a limit was reached teaches an operator nothing about what
/// to do next, and what to do next is different depending on which of the two
/// numbers ran out. Dropping a field frees a slot only when the table is
/// rewritten, so an operator at the field bound with few dead slots has a
/// different problem from one whose slots are gone.
fn check_the_bounds(object: &ObjectDefinition, slots: TableSlots, refusals: &mut Vec<Refusal>) {
    let fields = object.fields.len();
    let columns = object.columns();

    if fields > FIELDS_PER_OBJECT {
        refusals.push(Refusal {
            rule: Rule::FieldCountBound,
            subject: object.name.clone(),
            detail: format!(
                "{fields} fields against a bound of {FIELDS_PER_OBJECT}, in a table carrying {} dead column slots. Removing a field frees a slot only when the table is rewritten.",
                slots.dead_column_slots
            ),
        });
    }

    let wanted = slots.core_columns + columns + slots.dead_column_slots;
    if wanted > COLUMNS_PER_TABLE {
        refusals.push(Refusal {
            rule: Rule::ColumnSlotsExhausted,
            subject: object.name.clone(),
            detail: format!(
                "{wanted} column slots wanted against {COLUMNS_PER_TABLE}: {} for the object itself, {columns} for {fields} custom fields, and {} already dead. Only rewriting the table returns the dead ones.",
                slots.core_columns, slots.dead_column_slots
            ),
        });
    }
}
