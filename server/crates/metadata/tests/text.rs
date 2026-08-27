// SPDX-License-Identifier: AGPL-3.0-only
//! What the schema file carries, and what reading one refuses.
//!
//! Two kinds of test, and they answer different questions.
//!
//! The round trip is the property the format exists for: a definition written
//! and read back is the definition that was written. It is driven over a set of
//! values chosen to be hostile rather than typical - a line ending inside a
//! label, a lone backslash, a value that is one space, an empty string, a
//! script that is not Latin - because the ordinary label round trips under
//! every format anybody would write, including the ones that lose the rest.
//!
//! The refusals are one test per rule, and each asserts on the SET of rules a
//! file is refused under rather than on the presence of one, which is the rule
//! `definitions.rs` in this crate already follows: a reader that starts
//! refusing for a second reason reddens the test that was about the first,
//! instead of passing because the rule it names is still in the list somewhere.
//! Deleting any one refusal leaves exactly one test red.

use kontor_metadata::definition::{FieldDefinition, ObjectDefinition, TableSlots};
use kontor_metadata::field_type::FieldType;
use kontor_metadata::text::{self, Refusal};

/// Values chosen for what they break rather than for what they look like.
///
/// Every one of them is a thing an operator can type into a label or a picklist
/// value, and every one of them is lost or mangled by at least one obvious way
/// of writing this format.
const HOSTILE: [&str; 14] = [
    "",
    " ",
    "  ",
    " leading",
    "trailing ",
    " both ",
    "a\nb",
    "a\r\nb",
    "a\tb",
    "back\\slash",
    "ends on a backslash \\",
    "\\n is not a newline here",
    "Umlaut \u{e4}\u{f6}\u{fc} and \u{df}",
    "\u{5ba2}\u{6237}\u{540d}\u{79f0}",
];

fn field(name: &str, field_type: FieldType) -> FieldDefinition {
    FieldDefinition {
        name: name.to_owned(),
        label: "A field".to_owned(),
        field_type,
        required: false,
        values: Vec::new(),
        names_object: None,
    }
}

/// One object carrying one field of every kind of shape the format has to
/// write: values, a named object, and both of the answers `required` takes.
fn an_object() -> ObjectDefinition {
    let mut stage = field("stage", FieldType::Picklist);
    stage.label = "Stage".to_owned();
    stage.required = true;
    stage.values = vec![
        "Qualification".to_owned(),
        "Proposal".to_owned(),
        "Closed won".to_owned(),
    ];

    let mut owner = field("owner", FieldType::Reference);
    owner.label = "Owner".to_owned();
    owner.names_object = Some("person".to_owned());

    let mut amount = field("amount", FieldType::Money);
    amount.label = "Amount".to_owned();

    ObjectDefinition {
        name: "opportunity".to_owned(),
        label: "Opportunity".to_owned(),
        fields: vec![stage, owner, amount],
    }
}

/// The rules a file is refused under, sorted and deduplicated, which is what
/// every refusal test below compares.
fn refused_under(text: &str) -> Vec<&'static str> {
    match text::read(text) {
        Ok(_) => Vec::new(),
        Err(refusals) => {
            let mut names: Vec<&'static str> =
                refusals.iter().map(|refusal| refusal.rule.name()).collect();
            names.sort_unstable();
            names.dedup();
            names
        }
    }
}

fn refusals_of(text: &str) -> Vec<Refusal> {
    text::read(text).expect_err("this file is meant to be refused")
}

fn read_back(definition: &ObjectDefinition) -> ObjectDefinition {
    let written = text::write(definition);
    match text::read(&written) {
        Ok(read) => read,
        Err(refusals) => {
            let named: Vec<String> = refusals.iter().map(ToString::to_string).collect();
            panic!(
                "what write produced was refused by read: {}",
                named.join("; ")
            );
        }
    }
}

#[test]
fn an_ordinary_definition_survives_the_round_trip() {
    let definition = an_object();

    assert_eq!(read_back(&definition), definition);
}

#[test]
fn every_hostile_value_survives_the_round_trip_in_every_place_one_can_be_typed() {
    for hostile in HOSTILE {
        let mut stage = field(hostile, FieldType::Picklist);
        stage.label = hostile.to_owned();
        stage.values = vec![hostile.to_owned(), format!("{hostile}!")];

        let mut owner = field("owner", FieldType::Reference);
        owner.names_object = Some(hostile.to_owned());

        let definition = ObjectDefinition {
            name: hostile.to_owned(),
            label: hostile.to_owned(),
            fields: vec![stage, owner],
        };

        assert_eq!(
            read_back(&definition),
            definition,
            "the round trip lost {hostile:?}"
        );
    }
}

#[test]
fn a_definition_nothing_could_realise_still_round_trips() {
    // Refused by check for a blank label and for an identifier that is not one,
    // and carried by the file regardless. A schema is reviewed before it is
    // applied, so the file has to be able to hold the one being argued about.
    let mut broken = field("Not An Identifier", FieldType::Text);
    broken.label = "   ".to_owned();

    let definition = ObjectDefinition {
        name: "not an identifier either".to_owned(),
        label: String::new(),
        fields: vec![broken],
    };

    let slots = TableSlots {
        core_columns: 20,
        dead_column_slots: 0,
    };
    assert!(
        definition.check(slots).is_err(),
        "the fixture is not broken"
    );
    assert_eq!(read_back(&definition), definition);
}

#[test]
fn a_definition_with_no_fields_round_trips() {
    let definition = ObjectDefinition {
        name: "note".to_owned(),
        label: "Note".to_owned(),
        fields: Vec::new(),
    };

    assert_eq!(read_back(&definition), definition);
}

#[test]
fn the_written_file_is_what_a_reviewer_would_read() {
    let definition = an_object();

    assert_eq!(
        text::write(&definition),
        concat!(
            "object opportunity\n",
            "label Opportunity\n",
            "\n",
            "field stage\n",
            "label Stage\n",
            "type picklist\n",
            "required yes\n",
            "value Qualification\n",
            "value Proposal\n",
            "value Closed won\n",
            "\n",
            "field owner\n",
            "label Owner\n",
            "type reference\n",
            "required no\n",
            "references person\n",
            "\n",
            "field amount\n",
            "label Amount\n",
            "type money\n",
            "required no\n",
        )
    );
}

#[test]
fn nothing_written_ends_in_a_space_the_reader_would_refuse() {
    for hostile in HOSTILE {
        let mut only = field(hostile, FieldType::Text);
        only.label = hostile.to_owned();

        let written = text::write(&ObjectDefinition {
            name: hostile.to_owned(),
            label: hostile.to_owned(),
            fields: vec![only],
        });

        for line in written.lines() {
            assert!(
                !line.starts_with(' ') && !line.ends_with(' '),
                "writing {hostile:?} produced the line {line:?}, which read refuses"
            );
        }
    }
}

#[test]
fn a_comment_and_a_blank_line_are_passed_over() {
    let read = text::read(concat!(
        "# The object a deal is kept as.\n",
        "object opportunity\n",
        "\n",
        "label Opportunity\n",
        "#\n",
    ))
    .expect("comments and blank lines are not part of the definition");

    assert_eq!(
        read,
        ObjectDefinition {
            name: "opportunity".to_owned(),
            label: "Opportunity".to_owned(),
            fields: Vec::new(),
        }
    );
}

#[test]
fn a_word_that_is_not_a_key_is_refused() {
    assert_eq!(
        refused_under("object opportunity\nlabel Opportunity\ndescription A deal\n"),
        ["unknown-keyword"]
    );
}

#[test]
fn a_key_before_the_object_it_would_belong_to_is_refused() {
    assert_eq!(
        refused_under("label Opportunity\nobject opportunity\nlabel Opportunity\n"),
        ["keyword-out-of-place"]
    );
}

#[test]
fn a_field_key_outside_a_field_is_refused() {
    assert_eq!(
        refused_under(concat!(
            "object opportunity\n",
            "label Opportunity\n",
            "type text\n",
            "required yes\n",
            "value one\n",
            "references person\n",
        )),
        ["keyword-out-of-place"]
    );
}

#[test]
fn a_second_object_in_one_file_is_refused() {
    assert_eq!(
        refused_under(concat!(
            "object opportunity\n",
            "label Opportunity\n",
            "object person\n",
        )),
        ["object-repeated"]
    );
}

#[test]
fn a_file_that_opens_no_object_is_refused() {
    assert_eq!(refused_under("# nothing here\n"), ["object-missing"]);
}

#[test]
fn an_empty_file_is_refused_rather_than_read_as_an_empty_definition() {
    assert_eq!(refused_under(""), ["object-missing"]);
}

#[test]
fn a_key_given_twice_in_one_block_is_refused() {
    assert_eq!(
        refused_under(concat!(
            "object opportunity\n",
            "label Opportunity\n",
            "label Deal\n",
        )),
        ["key-repeated"]
    );
}

#[test]
fn a_second_key_in_a_field_is_refused_rather_than_overwriting_the_first() {
    assert_eq!(
        refused_under(concat!(
            "object opportunity\n",
            "label Opportunity\n",
            "field stage\n",
            "label Stage\n",
            "type text\n",
            "type picklist\n",
            "required no\n",
        )),
        ["key-repeated"]
    );
}

#[test]
fn an_object_with_no_label_is_refused() {
    assert_eq!(refused_under("object opportunity\n"), ["key-missing"]);
}

#[test]
fn a_field_missing_a_key_it_has_to_carry_is_refused() {
    assert_eq!(
        refused_under(concat!(
            "object opportunity\n",
            "label Opportunity\n",
            "field stage\n",
        )),
        ["key-missing"]
    );
}

#[test]
fn a_type_this_build_does_not_have_is_refused() {
    assert_eq!(
        refused_under(concat!(
            "object opportunity\n",
            "label Opportunity\n",
            "field stage\n",
            "label Stage\n",
            "type dropdown\n",
            "required no\n",
        )),
        ["type-unknown"]
    );
}

#[test]
fn a_required_line_that_is_neither_yes_nor_no_is_refused() {
    assert_eq!(
        refused_under(concat!(
            "object opportunity\n",
            "label Opportunity\n",
            "field stage\n",
            "label Stage\n",
            "type text\n",
            "required true\n",
        )),
        ["not-yes-or-no"]
    );
}

#[test]
fn an_escape_outside_the_five_is_refused() {
    assert_eq!(
        refused_under("object opportunity\nlabel A \\q here\n"),
        ["escape-unknown"]
    );
}

#[test]
fn a_line_ending_on_a_backslash_is_refused() {
    assert_eq!(
        refused_under("object opportunity\nlabel A trailing \\\n"),
        ["escape-unfinished"]
    );
}

#[test]
fn a_line_that_starts_or_ends_in_loose_whitespace_is_refused() {
    assert_eq!(
        refused_under("object opportunity\n  label Opportunity\n"),
        // The indented line is passed over as refused rather than read, so the
        // object is left with no label, which is the second rule here.
        ["key-missing", "loose-whitespace"]
    );

    assert_eq!(
        refused_under("object opportunity\nlabel Opportunity \n"),
        ["key-missing", "loose-whitespace"]
    );
}

/// The near miss worth the most. An interior space is ordinary and a space at
/// either end is the one that a line cannot carry, so a reader that escaped
/// every space or none of them passes the ordinary label and loses this one.
#[test]
fn a_value_that_is_only_spaces_is_neither_lost_nor_trimmed() {
    let written = text::write(&ObjectDefinition {
        name: "note".to_owned(),
        label: " ".to_owned(),
        fields: Vec::new(),
    });

    assert_eq!(written, "object note\nlabel \\s\n");
    assert_eq!(
        text::read(&written)
            .expect("a label of one space is a label")
            .label,
        " "
    );
}

/// The other near miss. `\\n` in a file is a backslash followed by an `n`, and
/// a reader that unescaped its input twice would turn it into a line ending and
/// then refuse the rest of the line as a key it does not have.
#[test]
fn a_written_backslash_does_not_become_an_escape_when_it_is_read() {
    let definition = ObjectDefinition {
        name: "note".to_owned(),
        label: "\\n".to_owned(),
        fields: Vec::new(),
    };

    assert_eq!(text::write(&definition), "object note\nlabel \\\\n\n");
    assert_eq!(read_back(&definition), definition);
}

#[test]
fn a_refusal_names_the_line_it_was_found_on() {
    let refusals = refusals_of(concat!(
        "object opportunity\n",
        "label Opportunity\n",
        "\n",
        "description A deal\n",
    ));

    assert_eq!(refusals.len(), 1, "{refusals:?}");
    assert_eq!(refusals[0].line, 4);
    assert!(
        refusals[0]
            .to_string()
            .starts_with("line 4: unknown-keyword: "),
        "{}",
        refusals[0]
    );
}

#[test]
fn a_refusal_about_the_whole_file_names_no_line() {
    let refusals = refusals_of("# a file with no object in it\n");

    assert_eq!(refusals.len(), 1, "{refusals:?}");
    assert_eq!(refusals[0].line, 0);
    assert!(
        refusals[0]
            .to_string()
            .starts_with("the file: object-missing: "),
        "{}",
        refusals[0]
    );
}

/// Every rule at once rather than the first, which is what
/// `ObjectDefinition::check` already promises and what stops an operator making
/// one round trip per mistake.
#[test]
fn every_rule_a_file_breaks_comes_back_at_once() {
    assert_eq!(
        refused_under(concat!(
            "object opportunity\n",
            "label Opportunity\n",
            "description A deal\n",
            "field stage\n",
            "label Stage\n",
            "type dropdown\n",
            "required perhaps\n",
            "field owner\n",
        )),
        [
            "key-missing",
            "not-yes-or-no",
            "type-unknown",
            "unknown-keyword"
        ]
    );
}

/// The near miss the rule above is written against. A type this build does not
/// have is one mistake on one line, and a reader that also reported the field
/// as carrying no type line would send an operator looking for a second line
/// that is not missing.
#[test]
fn a_line_refused_for_its_value_is_not_also_reported_as_a_line_that_is_not_there() {
    assert_eq!(
        refused_under(concat!(
            "object opportunity\n",
            "label Opportunity\n",
            "field stage\n",
            "label Stage\n",
            "type dropdown\n",
            "required perhaps\n",
        )),
        ["not-yes-or-no", "type-unknown"]
    );
}

/// The type names the file is written in are the register's, read from it
/// rather than repeated here, so a type renamed there renames in the format at
/// the same moment.
#[test]
fn every_type_is_written_and_read_back_under_the_name_the_register_gives_it() {
    for spec in kontor_metadata::field_type::FIELD_TYPES {
        let definition = ObjectDefinition {
            name: "opportunity".to_owned(),
            label: "Opportunity".to_owned(),
            fields: vec![field("a_field", spec.field_type)],
        };

        let written = text::write(&definition);
        assert!(
            written.contains(&format!("\ntype {}\n", spec.written_as)),
            "{} is not written as the register says: {written}",
            spec.written_as
        );
        assert_eq!(read_back(&definition), definition);
    }
}
