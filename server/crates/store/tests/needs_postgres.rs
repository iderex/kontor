// SPDX-License-Identifier: AGPL-3.0-only
//! The integration suite, and today it holds the harness rather than a test of
//! this project's own code.
//!
//! The store module is empty until #17, so there is nothing here yet to exercise
//! against a real database. What this file does now is make the split real: it
//! is a test target that cargo will not build unless the `needs-postgres`
//! feature is on, which is what `./test-needs-a-database` turns on and what
//! `./test` does not. A split declared in prose and in no manifest is a split
//! that the first person in a hurry runs straight through.
//!
//! It reaches a database and nothing else. No client crate is used, because
//! choosing one is #17's decision and a dependency taken here to prove a harness
//! would be a decision made by the harness.

use std::env;
use std::io::ErrorKind;
use std::net::TcpStream;
use std::time::Duration;

/// Where the database is. The same name the ecosystem already uses, so an
/// operator and a workflow do not each learn a private one.
const URL: &str = "KONTOR_TEST_DATABASE_URL";

/// Why a value could not be read as a URL, said without saying what the value
/// held.
///
/// A connection string carries a password, and the output of a failing test
/// goes wherever test output goes: a terminal somebody screenshots, a workflow
/// log that is kept, a bug report. So this reads the value and repeats none of
/// it. It is a function returning a string rather than the argument of a
/// `panic!` so that a test can read what it says.
fn why_it_is_not_a_url(url: &str) -> String {
    let seen = if url.is_empty() {
        "set to nothing at all"
    } else {
        "set to a value carrying no \"://\""
    };
    format!(
        "{URL} is {seen}, so there is no host and port to read out of it. What \
         it holds is not repeated here, because a connection string carries a \
         password. The shape this suite wants is \
         postgres://user:password@host:port/name."
    )
}

/// The host and port out of a `postgres://user:pass@host:port/name` string.
///
/// Deliberately not a URL parser. What this file needs is the two components a
/// socket takes, and a crate that understands every other part of the string
/// would be a dependency taken to prove a harness.
fn host_and_port(url: &str) -> Option<String> {
    let after_scheme = url.split_once("://")?.1;
    let authority = after_scheme.split(['/', '?']).next()?;
    let host_port = authority.rsplit_once('@').map_or(authority, |(_, h)| h);
    if host_port.contains(':') {
        Some(host_port.to_owned())
    } else {
        Some(format!("{host_port}:5432"))
    }
}

#[test]
fn the_database_this_suite_needs_is_reachable() {
    let url = env::var(URL).unwrap_or_else(|_| {
        panic!(
            "{URL} is not set. This suite is the one that needs a database; \
             ./test-needs-a-database says what to point it at, and ./test is \
             the command that does not need one."
        )
    });

    let address = host_and_port(&url).unwrap_or_else(|| panic!("{}", why_it_is_not_a_url(&url)));

    let socket = address
        .parse()
        .unwrap_or_else(|error| panic!("{URL} names {address}, which is not an address: {error}"));

    match TcpStream::connect_timeout(&socket, Duration::from_secs(10)) {
        Ok(_) => {}
        Err(error) if error.kind() == ErrorKind::TimedOut => {
            panic!("nothing answered at {address} within ten seconds")
        }
        Err(error) => panic!("nothing answered at {address}: {error}"),
    }
}

/// The near miss this is written against is one edit: putting the value back
/// into the sentence it was taken out of. The last assertion is what stops the
/// first two being facts about a constant, since a message that never reads its
/// argument cannot repeat it and cannot be made to.
#[test]
fn the_refusal_for_a_value_that_is_not_a_url_repeats_none_of_it() {
    let password = "9tKmNotForALog";
    let value = format!("kontor:{password}@db.example:5432/kontor");

    assert!(
        host_and_port(&value).is_none(),
        "the fixture has to be a value this refusal is reached for"
    );

    let refusal = why_it_is_not_a_url(&value);
    assert!(
        !refusal.contains(password),
        "the refusal carries the password out of the value: {refusal}"
    );
    assert!(
        !refusal.contains(&value),
        "the refusal carries the value: {refusal}"
    );
    assert_ne!(
        why_it_is_not_a_url(""),
        refusal,
        "the refusal does not read the value at all, so nothing above could have failed"
    );
}

#[test]
fn a_url_without_a_port_gets_the_default_one() {
    assert_eq!(
        host_and_port("postgres://kontor:secret@db.example/kontor").as_deref(),
        Some("db.example:5432")
    );
}

#[test]
fn a_url_with_a_port_keeps_it() {
    assert_eq!(
        host_and_port("postgres://kontor:secret@127.0.0.1:5433/kontor").as_deref(),
        Some("127.0.0.1:5433")
    );
}
