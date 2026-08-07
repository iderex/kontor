//! The field types an operator may choose from, and what each one commits the
//! rest of the system to.
//!
//! The set is fixed and it is listed once, in [`FIELD_TYPES`]. Everything that
//! needs to know what a type stores, what it accepts or how it orders reads it
//! from there rather than carrying a second copy, because a second copy is a
//! statement that drifts and the drift is invisible until a report sorts a
//! picklist alphabetically and nobody can say which of the two answers was
//! meant.
//!
//! Three properties per type, and each of the three is a promise to a different
//! layer. The storage type is what #18 realises as a column. The validation is
//! what the API layer refuses a value against. The sort is what M4 orders by,
//! and it is the one people assume is obvious and is not.

use core::fmt;

/// A field type. The set is closed, and adding one is a change here, a row in
/// [`FIELD_TYPES`], and a migration for anything stored.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FieldType {
    /// A single line of text.
    Text,
    /// Text over many lines, with no length an operator would notice.
    LongText,
    /// A whole number.
    WholeNumber,
    /// An amount with a currency, as `kontor_money` holds one.
    Money,
    /// A calendar day with no time and no zone.
    Date,
    /// A moment on the clock.
    Moment,
    /// True or false, and never a third thing.
    TrueOrFalse,
    /// One of a list of values the field itself carries.
    Picklist,
    /// An email address.
    Email,
    /// A telephone number.
    Telephone,
    /// An address on the web.
    WebAddress,
    /// A reference to a record of another object.
    Reference,
}

/// What a type accepts, stated as data so that the API layer and the import of
/// #23 refuse the same value for the same reason.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Validation {
    /// At most this many characters, counted as Unicode scalar values rather
    /// than as bytes, so a name in a non-Latin script is not shortened by the
    /// encoding.
    AtMostCharacters(usize),
    /// Any length. The bound is the database's own and not a rule anybody here
    /// chose.
    AnyLength,
    /// A whole number inside the range the storage type holds, refused rather
    /// than wrapped at the edges.
    WholeNumberInRange,
    /// Minor units and a currency the instance knows, which is
    /// `kontor_money::Money` and its refusal of an unknown currency.
    AmountWithAKnownCurrency,
    /// A calendar date, taken in one written form and never guessed from an
    /// ambiguous one.
    CalendarDate,
    /// A moment on the clock, carrying its offset.
    MomentOnTheClock,
    /// True or false.
    TrueOrFalse,
    /// Exactly one of the values the field declares, compared byte for byte.
    OneOfTheDeclaredValues,
    /// One local part, one at sign, one domain, and the whole compared
    /// lowercased when it is used as identity, which is the rule `docs/model.md`
    /// states for a person.
    EmailAddress,
    /// E.164, which is the form `docs/model.md` names in the identity rule for
    /// a person without an email address.
    TelephoneNumberInE164,
    /// An absolute address with a scheme, so that a value stored here can be
    /// opened rather than guessed at.
    AbsoluteWebAddress,
    /// The identifier of an existing record of the object the field names, and
    /// the database refuses one that does not exist rather than this layer.
    ExistingRecordOfTheNamedObject,
}

/// How a type orders. Stated per type because the answer is not the same for
/// all of them and the wrong one is silent: a report sorted by the wrong rule
/// is a report, not an error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sort {
    /// By the instance's collation. Not by code point, which puts `Z` before
    /// `a` and every accented letter after both.
    ByCollation,
    /// By value, as a number.
    Numerically,
    /// By amount, and only inside one currency. Ordering two currencies against
    /// each other needs a rate, and a rate has a moment, which is
    /// `docs/decisions/0005-money-and-time.md`. A list view over one currency
    /// orders; one over several is a question with no answer until somebody
    /// says which rate.
    ByAmountWithinOneCurrency,
    /// Earlier before later.
    Chronologically,
    /// False before true, so that the unset half of a list comes first.
    FalseBeforeTrue,
    /// By the position the operator gave the value in the list, and never
    /// alphabetically. A pipeline whose stages sort as "Closed, Negotiation,
    /// Proposal, Qualification" is a pipeline nobody can read, and it is what
    /// an alphabetical rule produces from the ordinary names.
    ByTheOrderTheValuesWereDeclaredIn,
    /// By the display name of the record referred to, under the same collation
    /// as text. Not by the identifier, which is an implementation detail and
    /// sorts as noise.
    ByTheNameOfTheRecordReferredTo,
}

/// One row of the register: a type and the three promises it carries.
#[derive(Debug, Clone, Copy)]
pub struct Spec {
    /// The type this row is about.
    pub field_type: FieldType,
    /// The name this type is written as in a definition, and the name a
    /// refusal quotes back.
    pub written_as: &'static str,
    /// The PostgreSQL column type #18 realises this as.
    pub storage: &'static str,
    /// How many columns that takes. One for every type but money, which is an
    /// amount and a currency and is two. It is here because the column limit a
    /// table has is counted in columns and not in fields, so a bound that
    /// counted fields would be wrong by the number of money fields.
    pub columns: usize,
    /// What a value has to be.
    pub validation: Validation,
    /// How a list of these orders.
    pub sort: Sort,
}

/// The whole set, in one place.
///
/// `text` is bounded at 255 characters and `long_text` is not. The bound is not
/// a storage decision, since PostgreSQL stores both the same way: it is what
/// makes a single line field stay a single line in every list view, export and
/// import, and it is the number the surrounding industry has used for so long
/// that data arriving from another system is already cut to it.
pub const FIELD_TYPES: &[Spec] = &[
    Spec {
        field_type: FieldType::Text,
        written_as: "text",
        storage: "text",
        columns: 1,
        validation: Validation::AtMostCharacters(255),
        sort: Sort::ByCollation,
    },
    Spec {
        field_type: FieldType::LongText,
        written_as: "long_text",
        storage: "text",
        columns: 1,
        validation: Validation::AnyLength,
        sort: Sort::ByCollation,
    },
    Spec {
        field_type: FieldType::WholeNumber,
        written_as: "whole_number",
        storage: "bigint",
        columns: 1,
        validation: Validation::WholeNumberInRange,
        sort: Sort::Numerically,
    },
    Spec {
        field_type: FieldType::Money,
        // Two columns rather than one, because an amount without its currency
        // is the defect docs/decisions/0005-money-and-time.md exists against.
        // #18 realises the pair; this row is what tells it there is a pair.
        storage: "bigint and text",
        columns: 2,
        written_as: "money",
        validation: Validation::AmountWithAKnownCurrency,
        sort: Sort::ByAmountWithinOneCurrency,
    },
    Spec {
        field_type: FieldType::Date,
        written_as: "date",
        storage: "date",
        columns: 1,
        validation: Validation::CalendarDate,
        sort: Sort::Chronologically,
    },
    Spec {
        field_type: FieldType::Moment,
        written_as: "moment",
        storage: "timestamptz",
        columns: 1,
        validation: Validation::MomentOnTheClock,
        sort: Sort::Chronologically,
    },
    Spec {
        field_type: FieldType::TrueOrFalse,
        written_as: "true_or_false",
        storage: "boolean",
        columns: 1,
        validation: Validation::TrueOrFalse,
        sort: Sort::FalseBeforeTrue,
    },
    Spec {
        field_type: FieldType::Picklist,
        storage: "text",
        columns: 1,
        written_as: "picklist",
        validation: Validation::OneOfTheDeclaredValues,
        sort: Sort::ByTheOrderTheValuesWereDeclaredIn,
    },
    Spec {
        field_type: FieldType::Email,
        written_as: "email",
        storage: "text",
        columns: 1,
        validation: Validation::EmailAddress,
        sort: Sort::ByCollation,
    },
    Spec {
        field_type: FieldType::Telephone,
        written_as: "telephone",
        storage: "text",
        columns: 1,
        validation: Validation::TelephoneNumberInE164,
        sort: Sort::ByCollation,
    },
    Spec {
        field_type: FieldType::WebAddress,
        written_as: "web_address",
        storage: "text",
        columns: 1,
        validation: Validation::AbsoluteWebAddress,
        sort: Sort::ByCollation,
    },
    Spec {
        field_type: FieldType::Reference,
        written_as: "reference",
        storage: "uuid",
        columns: 1,
        validation: Validation::ExistingRecordOfTheNamedObject,
        sort: Sort::ByTheNameOfTheRecordReferredTo,
    },
];

impl FieldType {
    /// The row of [`FIELD_TYPES`] for this type.
    ///
    /// # Panics
    ///
    /// Never in a build that passes its own suite. A variant with no row is
    /// what `every_type_has_exactly_one_row` refuses, and it runs against the
    /// same table this reads.
    #[must_use]
    pub fn spec(self) -> &'static Spec {
        FIELD_TYPES
            .iter()
            .find(|spec| spec.field_type == self)
            .expect("every field type has a row in FIELD_TYPES, which the suite refuses otherwise")
    }

    /// The type written that way, or nothing.
    #[must_use]
    pub fn from_written(written: &str) -> Option<Self> {
        FIELD_TYPES
            .iter()
            .find(|spec| spec.written_as == written)
            .map(|spec| spec.field_type)
    }

    /// Whether this type constrains its values to a list the field carries.
    ///
    /// The question is asked rather than the variant matched, so that a second
    /// constraining type answers here once instead of at every call site.
    #[must_use]
    pub fn constrains_its_values(self) -> bool {
        self.spec().validation == Validation::OneOfTheDeclaredValues
    }

    /// Whether this type points at a record of another object.
    #[must_use]
    pub fn names_another_object(self) -> bool {
        self.spec().validation == Validation::ExistingRecordOfTheNamedObject
    }
}

impl fmt::Display for FieldType {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.spec().written_as)
    }
}
