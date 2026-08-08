// SPDX-License-Identifier: AGPL-3.0-only
//! The connector boundary, shown to refuse what it names, and then asked about
//! this tree.
//!
//! Two kinds of test, and the difference is the whole reason there are two. The
//! first kind judges manifests written here, so what it proves is the rule. The
//! second kind judges the manifests this repository actually holds, so what it
//! proves is the state of the tree on the day it ran. A suite with only the
//! second kind passes forever the moment the rule stops working, because a tree
//! with no connectors in it earns no refusals either way.
//!
//! Each refusal has a near miss beside it wherever one exists, because a fixture
//! that could not have passed proves less than one that nearly did: the key that
//! is present and empty, the identifier that is nearly the crate name, the
//! second copy of a key.

use kontor_connector::{CRATE_PREFIX, KEYS, Manifest, Rule, TABLE, judge};
use std::fs;
use std::path::{Path, PathBuf};

/// A manifest for a crate with the six keys filled in, so that a case can take
/// one away and change nothing else.
fn declared(name: &str, id: &str) -> Manifest {
    Manifest {
        path: format!("server/crates/{id}/Cargo.toml"),
        text: format!(
            "[package]\n\
             name = \"{name}\"\n\
             version = \"0.0.0\"\n\
             \n\
             {TABLE}\n\
             id = \"{id}\"\n\
             purpose = \"match messages to the records they concern\"\n\
             destination = \"the mail server named in this connector's configuration, and no other address\"\n\
             outbound = \"none\"\n\
             inbound = \"message metadata, being sender, recipients, subject and time\"\n\
             credential = \"the mailbox credential the operator entered, scoped to this connector\"\n\
             \n\
             [dependencies]\n"
        ),
    }
}

/// The rules a set of manifests earned, as names, so a case states the set it
/// expected rather than an index into a list.
fn rules(manifests: &[Manifest]) -> Vec<&'static str> {
    judge(manifests)
        .iter()
        .map(|refusal| refusal.rule.name())
        .collect()
}

#[test]
fn a_complete_declaration_on_a_connector_crate_is_accepted() {
    let manifests = [declared("kontor-connector-mailbox", "mailbox")];
    assert_eq!(rules(&manifests), Vec::<&str>::new());
}

#[test]
fn a_crate_that_is_not_a_connector_and_declares_nothing_is_accepted() {
    let manifests = [Manifest {
        path: "server/crates/money/Cargo.toml".to_owned(),
        text: "[package]\nname = \"kontor-money\"\n\n[dependencies]\njiff = \"0.2.35\"\n"
            .to_owned(),
    }];
    assert_eq!(rules(&manifests), Vec::<&str>::new());
}

#[test]
fn the_crate_holding_the_check_is_not_itself_a_connector() {
    // The near miss on the prefix. `kontor-connector` is one hyphen away from
    // being judged as a connector by its own rule, which would make this crate
    // owe a declaration for reaching nothing.
    assert!(!"kontor-connector".starts_with(CRATE_PREFIX));
    assert!("kontor-connector-mailbox".starts_with(CRATE_PREFIX));
}

#[test]
fn a_connector_crate_with_no_declaration_is_refused() {
    let manifests = [Manifest {
        path: "server/crates/connector-mailbox/Cargo.toml".to_owned(),
        text: "[package]\nname = \"kontor-connector-mailbox\"\n\n[dependencies]\n".to_owned(),
    }];
    assert_eq!(rules(&manifests), ["declaration-missing"]);
}

#[test]
fn a_declaration_on_a_crate_that_is_not_a_connector_is_refused() {
    let mut manifest = declared("kontor-connector-mailbox", "mailbox");
    manifest.text = manifest
        .text
        .replace("kontor-connector-mailbox", "kontor-reporting");
    // The identifier still says mailbox, so this case could earn two refusals.
    // It earns the one about the name, because a crate that is not a connector
    // is not asked what its declaration says.
    assert_eq!(
        rules(&[manifest]),
        ["declaration-without-a-connector"],
        "a crate outside the naming rule is judged on the name and not on the table"
    );
}

#[test]
fn every_required_key_is_refused_when_it_is_absent() {
    for key in KEYS {
        let complete = declared("kontor-connector-mailbox", "mailbox");
        let text = complete
            .text
            .lines()
            .filter(|line| !line.trim_start().starts_with(&format!("{key} =")))
            .collect::<Vec<_>>()
            .join("\n");
        let manifests = [Manifest {
            path: complete.path.clone(),
            text,
        }];
        assert_eq!(
            rules(&manifests),
            ["key-missing"],
            "taking {key} away has to be refused, and by that rule"
        );
    }
}

#[test]
fn a_key_present_and_empty_is_refused_as_the_absent_one_is() {
    // The near miss. Somebody who has read the rule and typed the key without
    // answering it has satisfied a check that reads only the key names.
    for key in KEYS {
        let complete = declared("kontor-connector-mailbox", "mailbox");
        let text = complete
            .text
            .lines()
            .map(|line| {
                if line.trim_start().starts_with(&format!("{key} =")) {
                    format!("{key} = \"\"")
                } else {
                    line.to_owned()
                }
            })
            .collect::<Vec<_>>()
            .join("\n");
        let manifests = [Manifest {
            path: complete.path.clone(),
            text,
        }];
        assert!(
            rules(&manifests).contains(&Rule::ValueBlank.name()),
            "{key} present and empty has to be refused"
        );
    }
}

#[test]
fn a_value_that_is_only_spaces_is_refused_too() {
    let complete = declared("kontor-connector-mailbox", "mailbox");
    let text = complete
        .text
        .replace("outbound = \"none\"", "outbound = \"   \"");
    assert_eq!(
        rules(&[Manifest {
            path: complete.path,
            text
        }]),
        ["value-blank"]
    );
}

#[test]
fn a_key_nobody_asked_for_is_refused() {
    let complete = declared("kontor-connector-mailbox", "mailbox");
    let text = complete.text.replace(
        "outbound = \"none\"",
        "outbound = \"none\"\nanalytics = \"usage counts\"",
    );
    assert_eq!(
        rules(&[Manifest {
            path: complete.path,
            text
        }]),
        ["key-unknown"]
    );
}

#[test]
fn one_key_written_twice_is_refused() {
    let complete = declared("kontor-connector-mailbox", "mailbox");
    let text = complete.text.replace(
        "outbound = \"none\"",
        "outbound = \"none\"\noutbound = \"the record the message was matched to\"",
    );
    assert_eq!(
        rules(&[Manifest {
            path: complete.path,
            text
        }]),
        ["key-repeated"]
    );
}

#[test]
fn an_identifier_that_is_not_the_crate_is_refused() {
    // The near miss. One character out, which is what a copied declaration looks
    // like, and it is the case that files a real disclosure under a name nobody
    // reaches it by.
    let complete = declared("kontor-connector-mailbox", "mailbox");
    let text = complete
        .text
        .replace("id = \"mailbox\"", "id = \"mailboxes\"");
    assert_eq!(
        rules(&[Manifest {
            path: complete.path,
            text
        }]),
        ["id-does-not-match-the-crate"]
    );
}

#[test]
fn a_line_the_check_cannot_read_is_refused_rather_than_guessed_at() {
    let complete = declared("kontor-connector-mailbox", "mailbox");
    for line in [
        "destination = ['a', 'b']",
        "destination = \"\"\"a\nb\"\"\"",
        "destination = \"the server the operator called \"theirs\"\"",
        "destination",
    ] {
        let text = complete.text.replace(
            "destination = \"the mail server named in this connector's configuration, and no other address\"",
            line,
        );
        assert!(
            rules(&[Manifest {
                path: complete.path.clone(),
                text
            }])
            .contains(&Rule::LineNotADeclaration.name()),
            "the check has to say it cannot read `{line}` rather than pass it"
        );
    }
}

#[test]
fn a_comment_and_a_blank_line_inside_the_table_are_not_refused() {
    let complete = declared("kontor-connector-mailbox", "mailbox");
    let text = complete.text.replace(
        "outbound = \"none\"",
        "# Nothing leaves. The connector reads and writes inward only.\n\noutbound = \"none\"",
    );
    assert_eq!(
        rules(&[Manifest {
            path: complete.path,
            text
        }]),
        Vec::<&str>::new()
    );
}

#[test]
fn a_table_after_the_declaration_ends_it() {
    // A key of the same name in a later table is not this declaration's, and a
    // reader that ran past the table boundary would take `id` out of
    // `[dependencies]` and refuse a declaration that is correct.
    let complete = declared("kontor-connector-mailbox", "mailbox");
    let text = format!("{}id = \"something-else\"\n", complete.text);
    assert_eq!(
        rules(&[Manifest {
            path: complete.path,
            text
        }]),
        Vec::<&str>::new()
    );
}

#[test]
fn a_manifest_with_no_package_table_is_passed_over() {
    let manifests = [Manifest {
        path: "server/Cargo.toml".to_owned(),
        text: "[workspace]\nmembers = [\"crates/*\"]\n".to_owned(),
    }];
    assert_eq!(rules(&manifests), Vec::<&str>::new());
}

// Below here the subject is this repository rather than the rule.

/// Every manifest under `server/`, which is the set the boundary applies to.
///
/// The walk is over the directory rather than over the workspace member list,
/// because a manifest that is not a member is still a crate somebody can build,
/// and a member list is a place a connector could be left out of.
fn manifests_under(directory: &Path) -> Vec<Manifest> {
    let mut found = Vec::new();
    let mut queue = vec![directory.to_path_buf()];

    while let Some(next) = queue.pop() {
        let entries = fs::read_dir(&next)
            .unwrap_or_else(|error| panic!("{} cannot be read: {error}", next.display()));
        for entry in entries {
            let entry = entry.unwrap_or_else(|error| {
                panic!("an entry under {} cannot be read: {error}", next.display())
            });
            let path = entry.path();
            if path.is_dir() {
                // target/ holds manifests cargo wrote and nobody committed.
                if path.file_name().is_some_and(|name| name == "target") {
                    continue;
                }
                queue.push(path);
            } else if path.file_name().is_some_and(|name| name == "Cargo.toml") {
                let text = fs::read_to_string(&path)
                    .unwrap_or_else(|error| panic!("{} cannot be read: {error}", path.display()));
                // Named as a reader of this repository would name it, rather
                // than as an absolute path with the walk's own detours in it.
                let inside = path.strip_prefix(directory).unwrap_or(path.as_path());
                found.push(Manifest {
                    path: format!("server/{}", inside.display().to_string().replace('\\', "/")),
                    text,
                });
            }
        }
    }

    found
}

/// The server workspace, from this crate's own manifest directory.
fn server() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .canonicalize()
        .expect("the server workspace is the directory two above this crate")
}

#[test]
fn every_manifest_in_this_workspace_satisfies_the_boundary() {
    let manifests = manifests_under(&server());
    assert!(
        manifests.len() > 1,
        "the walk found {} manifests, and a walk that found nothing would pass whatever the tree held",
        manifests.len()
    );

    let refusals = judge(&manifests);
    assert!(
        refusals.is_empty(),
        "{}",
        refusals
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("\n")
    );
}

#[test]
fn this_workspace_holds_no_connector_yet() {
    // Stated as a test rather than left implied, because the test above is green
    // on a tree with no connectors in it and would go on being green if the rule
    // stopped working. When the first connector lands this test is the one that
    // fails, and the answer is to delete it rather than to work around it.
    let named: Vec<String> = manifests_under(&server())
        .iter()
        .filter_map(|manifest| {
            manifest
                .text
                .lines()
                .find_map(|line| line.trim().strip_prefix("name = \""))
                .and_then(|name| name.strip_suffix('"'))
                .filter(|name| name.starts_with(CRATE_PREFIX))
                .map(ToOwned::to_owned)
        })
        .collect();

    assert_eq!(
        named,
        Vec::<String>::new(),
        "a connector crate has landed, so the tests above are now judging a real declaration \
         and this one has served its purpose"
    );
}
