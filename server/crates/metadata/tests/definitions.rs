//! What the metadata layer refuses, and what it does not.
//!
//! Every rule has a test that trips exactly it. The assertion is on the SET of
//! rules a definition is refused under and not on the presence of one, so a
//! check that starts firing for a second reason reddens the test that was about
//! the first, rather than passing because the rule it names is still in the
//! list somewhere. Deleting any one check leaves exactly one test red, which is
//! what makes each one evidence that its rule bites.
//!
//! The register of types is tested for the property the whole design rests on:
//! that it is one place. A type with no row, two rows, or a name shared with
//! another type is refused here, and a type added to the enum and nowhere else
//! does not compile.

use kontor_metadata::definition::{
    COLUMNS_PER_TABLE, FIELDS_PER_OBJECT, FieldDefinition, IDENTIFIER_CHARACTERS, ObjectDefinition,
    Rule, TableSlots,
};
use kontor_metadata::field_type::{FIELD_TYPES, FieldType, Sort, Validation};

/// Every type, written out. What keeps it honest is `index` below rather than
/// anybody's care.
const EVERY_TYPE: [FieldType; 12] = [
    FieldType::Text,
    FieldType::LongText,
    FieldType::WholeNumber,
    FieldType::Money,
    FieldType::Date,
    FieldType::Moment,
    FieldType::TrueOrFalse,
    FieldType::Picklist,
    FieldType::Email,
    FieldType::Telephone,
    FieldType::WebAddress,
    FieldType::Reference,
];

/// A distinct number per variant, through a match with no catch all arm. A
/// thirteenth type stops this compiling, which is how `EVERY_TYPE` is kept
/// complete without a second list anybody has to remember.
fn index(field_type: FieldType) -> usize {
    match field_type {
        FieldType::Text => 0,
        FieldType::LongText => 1,
        FieldType::WholeNumber => 2,
        FieldType::Money => 3,
        FieldType::Date => 4,
        FieldType::Moment => 5,
        FieldType::TrueOrFalse => 6,
        FieldType::Picklist => 7,
        FieldType::Email => 8,
        FieldType::Telephone => 9,
        FieldType::WebAddress => 10,
        FieldType::Reference => 11,
    }
}

fn text_field(name: &str) -> FieldDefinition {
    FieldDefinition {
        name: name.to_owned(),
        label: "A field".to_owned(),
        field_type: FieldType::Text,
        required: false,
        values: Vec::new(),
        names_object: None,
    }
}

fn object(fields: Vec<FieldDefinition>) -> ObjectDefinition {
    ObjectDefinition {
        name: "invoice".to_owned(),
        label: "Invoice".to_owned(),
        fields,
    }
}

const EMPTY_TABLE: TableSlots = TableSlots {
    core_columns: 20,
    dead_column_slots: 0,
};

/// The rules a definition is refused under, sorted and deduplicated, which is
/// what every assertion below compares.
fn refused_under(definition: &ObjectDefinition, slots: TableSlots) -> Vec<&'static str> {
    match definition.check(slots) {
        Ok(()) => Vec::new(),
        Err(refusals) => {
            let mut names: Vec<&'static str> =
                refusals.iter().map(|refusal| refusal.rule.name()).collect();
            names.sort_unstable();
            names.dedup();
            names
        }
    }
}

fn detail_of(definition: &ObjectDefinition, slots: TableSlots, rule: Rule) -> String {
    let refusals = definition
        .check(slots)
        .expect_err("this definition is meant to be refused");
    refusals
        .iter()
        .find(|refusal| refusal.rule == rule)
        .map(|refusal| refusal.detail.clone())
        .expect("the rule under test is meant to be among the refusals")
}

#[test]
fn every_type_is_in_the_list_these_tests_use() {
    let mut seen = [false; EVERY_TYPE.len()];
    for field_type in EVERY_TYPE {
        seen[index(field_type)] = true;
    }
    assert!(
        seen.iter().all(|found| *found),
        "a type reachable through index() is missing from EVERY_TYPE"
    );
}

#[test]
fn every_type_has_exactly_one_row_in_the_register() {
    for field_type in EVERY_TYPE {
        let rows = FIELD_TYPES
            .iter()
            .filter(|spec| spec.field_type == field_type)
            .count();
        assert_eq!(rows, 1, "{field_type} has {rows} rows and should have one");
    }
    assert_eq!(
        FIELD_TYPES.len(),
        EVERY_TYPE.len(),
        "the register carries a row for a type that is not a type"
    );
}

#[test]
fn no_two_types_are_written_the_same_way() {
    for (position, spec) in FIELD_TYPES.iter().enumerate() {
        let earlier = FIELD_TYPES[..position]
            .iter()
            .find(|seen| seen.written_as == spec.written_as);
        assert!(
            earlier.is_none(),
            "\"{}\" is the written name of two types",
            spec.written_as
        );
    }
}

#[test]
fn a_type_round_trips_through_the_name_it_is_written_as() {
    for field_type in EVERY_TYPE {
        let written = field_type.spec().written_as;
        assert_eq!(
            FieldType::from_written(written),
            Some(field_type),
            "{field_type} is written as \"{written}\" and does not read back"
        );
        assert_eq!(field_type.to_string(), written);
    }
}

#[test]
fn a_name_no_type_is_written_as_is_no_type() {
    assert_eq!(FieldType::from_written("percentage"), None);
    assert_eq!(FieldType::from_written(""), None);
    assert_eq!(FieldType::from_written("TEXT"), None);
}

#[test]
fn every_row_says_what_it_stores_and_in_how_many_columns() {
    for spec in FIELD_TYPES {
        assert!(
            !spec.storage.is_empty(),
            "{} says nothing about what it stores",
            spec.field_type
        );
        assert!(
            spec.columns >= 1,
            "{} claims to need no column",
            spec.field_type
        );
    }
}

#[test]
fn money_is_the_only_type_that_is_two_columns() {
    for spec in FIELD_TYPES {
        let expected = usize::from(spec.field_type == FieldType::Money) + 1;
        assert_eq!(
            spec.columns, expected,
            "{} takes {} columns",
            spec.field_type, spec.columns
        );
    }
}

#[test]
fn only_a_picklist_constrains_its_values() {
    for field_type in EVERY_TYPE {
        assert_eq!(
            field_type.constrains_its_values(),
            field_type == FieldType::Picklist,
            "{field_type} disagrees with itself about constraining its values"
        );
    }
}

#[test]
fn only_a_reference_names_another_object() {
    for field_type in EVERY_TYPE {
        assert_eq!(
            field_type.names_another_object(),
            field_type == FieldType::Reference,
            "{field_type} disagrees with itself about naming another object"
        );
    }
}

#[test]
fn a_picklist_sorts_by_the_order_its_values_were_declared_in() {
    assert_eq!(
        FieldType::Picklist.spec().sort,
        Sort::ByTheOrderTheValuesWereDeclaredIn,
        "an alphabetical pipeline is what this rule exists against"
    );
    assert_eq!(
        FieldType::Money.spec().sort,
        Sort::ByAmountWithinOneCurrency
    );
    assert_eq!(
        FieldType::Text.spec().validation,
        Validation::AtMostCharacters(255)
    );
    assert_eq!(FieldType::LongText.spec().validation, Validation::AnyLength);
}

#[test]
fn a_definition_that_breaks_no_rule_is_accepted() {
    let definition = object(vec![
        text_field("purchase_order"),
        FieldDefinition {
            name: "stage".to_owned(),
            label: "Stage".to_owned(),
            field_type: FieldType::Picklist,
            required: true,
            values: vec!["Qualification".to_owned(), "Proposal".to_owned()],
            names_object: None,
        },
        FieldDefinition {
            name: "billed_to".to_owned(),
            label: "Billed to".to_owned(),
            field_type: FieldType::Reference,
            required: false,
            values: Vec::new(),
            names_object: Some("organisation".to_owned()),
        },
    ]);

    assert_eq!(refused_under(&definition, EMPTY_TABLE), Vec::<&str>::new());
}

#[test]
fn a_name_that_is_not_an_unquoted_identifier_is_refused() {
    for name in ["Purchase", "purchase order", "1st", "purchase-order", ""] {
        let definition = object(vec![text_field(name)]);
        assert_eq!(
            refused_under(&definition, EMPTY_TABLE),
            vec!["name-shape"],
            "\"{name}\" was not refused for its shape alone"
        );
    }
}

#[test]
fn a_name_ending_in_an_underscore_or_carrying_two_is_refused() {
    for name in ["purchase_", "purchase__order"] {
        let definition = object(vec![text_field(name)]);
        assert_eq!(
            refused_under(&definition, EMPTY_TABLE),
            vec!["name-shape"],
            "\"{name}\" was not refused for its shape alone"
        );
    }
}

#[test]
fn a_name_longer_than_postgresql_keeps_is_refused() {
    let name = "a".repeat(IDENTIFIER_CHARACTERS + 1);
    let definition = object(vec![text_field(&name)]);

    assert_eq!(refused_under(&definition, EMPTY_TABLE), vec!["name-length"]);

    let detail = detail_of(&definition, EMPTY_TABLE, Rule::NameLength);
    assert!(
        detail.contains(&(IDENTIFIER_CHARACTERS + 1).to_string())
            && detail.contains(&IDENTIFIER_CHARACTERS.to_string()),
        "the refusal names neither the length nor the limit: {detail}"
    );

    let at_the_limit = object(vec![text_field(&"a".repeat(IDENTIFIER_CHARACTERS))]);
    assert_eq!(
        refused_under(&at_the_limit, EMPTY_TABLE),
        Vec::<&str>::new()
    );
}

#[test]
fn one_name_on_two_fields_is_refused() {
    let definition = object(vec![text_field("amount_due"), text_field("amount_due")]);

    assert_eq!(
        refused_under(&definition, EMPTY_TABLE),
        vec!["name-repeated"]
    );
    assert!(
        detail_of(&definition, EMPTY_TABLE, Rule::NameRepeated).contains("amount_due"),
        "the refusal does not say which name"
    );
}

#[test]
fn a_definition_with_nothing_to_read_on_a_screen_is_refused() {
    let mut field = text_field("amount_due");
    field.label = "   ".to_owned();
    let definition = object(vec![field]);
    assert_eq!(refused_under(&definition, EMPTY_TABLE), vec!["label-blank"]);

    let mut unlabelled = object(vec![text_field("amount_due")]);
    unlabelled.label = String::new();
    assert_eq!(refused_under(&unlabelled, EMPTY_TABLE), vec!["label-blank"]);
}

#[test]
fn a_picklist_with_no_values_is_refused() {
    let definition = object(vec![FieldDefinition {
        name: "stage".to_owned(),
        label: "Stage".to_owned(),
        field_type: FieldType::Picklist,
        required: false,
        values: Vec::new(),
        names_object: None,
    }]);

    assert_eq!(
        refused_under(&definition, EMPTY_TABLE),
        vec!["no-values-to-choose-from"]
    );
}

#[test]
fn values_on_a_type_that_takes_none_are_refused() {
    let mut field = text_field("stage");
    field.values = vec!["Open".to_owned()];
    let definition = object(vec![field]);

    assert_eq!(
        refused_under(&definition, EMPTY_TABLE),
        vec!["values-where-the-type-takes-none"]
    );
}

#[test]
fn a_blank_or_repeated_value_is_refused() {
    let blank = object(vec![FieldDefinition {
        name: "stage".to_owned(),
        label: "Stage".to_owned(),
        field_type: FieldType::Picklist,
        required: false,
        values: vec!["Open".to_owned(), "  ".to_owned()],
        names_object: None,
    }]);
    assert_eq!(refused_under(&blank, EMPTY_TABLE), vec!["value-blank"]);

    let repeated = object(vec![FieldDefinition {
        name: "stage".to_owned(),
        label: "Stage".to_owned(),
        field_type: FieldType::Picklist,
        required: false,
        values: vec!["Open".to_owned(), "Won".to_owned(), "Open".to_owned()],
        names_object: None,
    }]);
    assert_eq!(
        refused_under(&repeated, EMPTY_TABLE),
        vec!["value-repeated"]
    );
    assert!(
        detail_of(&repeated, EMPTY_TABLE, Rule::ValueRepeated).contains("Open"),
        "the refusal does not say which value"
    );
}

#[test]
fn a_reference_naming_no_object_is_refused() {
    let definition = object(vec![FieldDefinition {
        name: "billed_to".to_owned(),
        label: "Billed to".to_owned(),
        field_type: FieldType::Reference,
        required: false,
        values: Vec::new(),
        names_object: None,
    }]);

    assert_eq!(
        refused_under(&definition, EMPTY_TABLE),
        vec!["reference-names-no-object"]
    );
}

#[test]
fn an_object_named_by_a_type_that_points_at_nothing_is_refused() {
    let mut field = text_field("billed_to");
    field.names_object = Some("organisation".to_owned());
    let definition = object(vec![field]);

    assert_eq!(
        refused_under(&definition, EMPTY_TABLE),
        vec!["object-named-where-the-type-takes-none"]
    );
}

#[test]
fn the_object_a_reference_names_is_held_to_the_same_shape() {
    let definition = object(vec![FieldDefinition {
        name: "billed_to".to_owned(),
        label: "Billed to".to_owned(),
        field_type: FieldType::Reference,
        required: false,
        values: Vec::new(),
        names_object: Some("Organisation".to_owned()),
    }]);

    assert_eq!(refused_under(&definition, EMPTY_TABLE), vec!["name-shape"]);
}

#[test]
fn more_fields_than_an_object_may_carry_is_refused_and_the_bound_is_not() {
    let at_the_bound = object(
        (0..FIELDS_PER_OBJECT)
            .map(|position| text_field(&format!("field_{position}")))
            .collect(),
    );
    assert_eq!(
        refused_under(&at_the_bound, EMPTY_TABLE),
        Vec::<&str>::new()
    );

    let over = object(
        (0..=FIELDS_PER_OBJECT)
            .map(|position| text_field(&format!("field_{position}")))
            .collect(),
    );
    assert_eq!(refused_under(&over, EMPTY_TABLE), vec!["field-count-bound"]);

    let detail = detail_of(&over, EMPTY_TABLE, Rule::FieldCountBound);
    assert!(
        detail.contains(&(FIELDS_PER_OBJECT + 1).to_string())
            && detail.contains(&FIELDS_PER_OBJECT.to_string())
            && detail.contains(&EMPTY_TABLE.dead_column_slots.to_string()),
        "the refusal does not carry all three numbers: {detail}"
    );
}

#[test]
fn a_table_whose_slots_are_gone_is_refused_separately_from_the_field_bound() {
    let definition = object(vec![text_field("purchase_order")]);
    let crowded = TableSlots {
        core_columns: 20,
        dead_column_slots: COLUMNS_PER_TABLE,
    };

    assert_eq!(
        refused_under(&definition, crowded),
        vec!["column-slots-exhausted"],
        "a table out of slots is a different problem from an object at its field bound"
    );

    let detail = detail_of(&definition, crowded, Rule::ColumnSlotsExhausted);
    assert!(
        detail.contains(&COLUMNS_PER_TABLE.to_string()) && detail.contains("20"),
        "the refusal does not carry the numbers an operator would act on: {detail}"
    );
}

#[test]
fn a_money_field_is_counted_as_the_two_columns_it_is() {
    let money = |name: &str| FieldDefinition {
        name: name.to_owned(),
        label: "Amount".to_owned(),
        field_type: FieldType::Money,
        required: false,
        values: Vec::new(),
        names_object: None,
    };

    let definition = object(vec![money("agreed"), money("invoiced"), text_field("note")]);
    assert_eq!(definition.columns(), 5);

    // The case a bound counting fields rather than columns would let through:
    // the object is inside its field bound and the table is not inside its slot
    // limit, and only the column arithmetic can tell.
    let slots = TableSlots {
        core_columns: 20,
        dead_column_slots: COLUMNS_PER_TABLE - 24,
    };
    assert_eq!(
        refused_under(&definition, slots),
        vec!["column-slots-exhausted"]
    );
}

#[test]
fn a_definition_breaking_several_rules_reports_all_of_them() {
    let mut field = FieldDefinition {
        name: "Stage".to_owned(),
        label: String::new(),
        field_type: FieldType::Picklist,
        required: false,
        values: Vec::new(),
        names_object: Some("organisation".to_owned()),
    };
    field.required = true;
    let definition = object(vec![field]);

    assert_eq!(
        refused_under(&definition, EMPTY_TABLE),
        vec![
            "label-blank",
            "name-shape",
            "no-values-to-choose-from",
            "object-named-where-the-type-takes-none",
        ],
        "an operator fixing one refusal per migration window pays for each one"
    );
}

#[test]
fn every_rule_has_its_own_name_and_a_refusal_prints_it() {
    let rules = [
        Rule::NameShape,
        Rule::NameLength,
        Rule::NameRepeated,
        Rule::LabelBlank,
        Rule::NoValuesToChooseFrom,
        Rule::ValuesWhereTheTypeTakesNone,
        Rule::ValueBlank,
        Rule::ValueRepeated,
        Rule::ReferenceNamesNoObject,
        Rule::ObjectNamedWhereTheTypeTakesNone,
        Rule::FieldCountBound,
        Rule::ColumnSlotsExhausted,
    ];

    for (position, rule) in rules.iter().enumerate() {
        assert!(!rule.name().is_empty());
        let earlier = rules[..position]
            .iter()
            .find(|seen| seen.name() == rule.name());
        assert!(earlier.is_none(), "\"{}\" names two rules", rule.name());
    }

    let definition = object(vec![text_field("Purchase")]);
    let refusals = definition
        .check(EMPTY_TABLE)
        .expect_err("this definition is meant to be refused");
    let printed = refusals
        .first()
        .expect("a refusal was returned")
        .to_string();
    assert!(
        printed.contains("name-shape") && printed.contains("Purchase"),
        "a printed refusal names neither the rule nor its subject: {printed}"
    );
}
