# 0011. What a connector may do, and what it may never do

Status: accepted
Issue: #82
Supersedes: nothing
Superseded by: nothing

## The decision

A connector is off until an operator configures it, reaches only the service
named in that configuration, and declares in the tree what crosses the boundary
in each direction. No connector sends anything to a service this project chose
rather than the operator. Nothing about a connector is enabled by an update.

The declaration is data in the connector crate's own manifest, and a refusal in
the tree reads it, so a connector that has not said what crosses cannot land.

## Why the boundary is decided here rather than per connector

Every connector is a hole in the sentence this project is sold on. `README.md`
says the customer's data lives on the operator's own infrastructure, and a
mailbox connector by definition talks to a server that is not theirs.

The position is not that connectors are forbidden. A CRM that cannot see a
mailbox or a calendar is a CRM somebody keeps a spreadsheet beside. The position
is that each one is a deliberate act by the operator, bounded and disclosed, and
that the boundary is agreed once, in front of the first connector, rather than
negotiated with each one while somebody is trying to ship it.

Negotiating per connector has a predictable outcome. The first one is argued
carefully, the second one cites the first, and the tenth is approved because it
looks like the nine before it. What is being approved by then is not the case in
front of anybody.

## The rules

**A connector is off until an operator configures it.** No connector has a
default that reaches anything. An instance with no configuration reaches nothing,
which is what makes the claim in `docs/data-protection.md` under #100 a statement
about the software rather than about how carefully it was set up.

**A connector reaches the service named in its configuration and nothing else.**
Not a second address for a lookup, not an update endpoint, not a diagnostic
report. The address is the operator's answer and the connector has no other one.

**A connector declares what crosses, in each direction, where a machine reads
it.** The declaration is below. It is required rather than encouraged, because a
boundary that is disclosed when somebody remembers is disclosed for the
connectors nobody was worried about.

**No connector sends customer data to a service this project chose.** That rules
out a hosted relay, a hosted extraction or matching service, and any measurement
of connector usage that leaves the host. Where an operator wants one of those,
they configure the address themselves and the first rule applies to it like any
other.

**Nothing about a connector is enabled by an update.** A new version may add a
connector to the code and may not turn one on, may not widen what an existing one
reaches, and may not add a direction to what crosses. Where a version needs one
of those, it is a configuration change the operator makes, and the release notes
say so before the upgrade rather than after.

**Credentials are held the way every other secret is.** A connector does not
invent storage, does not write a credential to a log or a bug report, and holds
one credential scoped to one connection so that revoking a connection under #88
revokes something rather than everything. #91 is where secret handling is
decided, and this record depends on it rather than restating it. Until #91 lands
there is no store for a connector to use, which is one of the reasons no
connector has landed.

## What a connector may never do

Stated in terms a review can refuse a proposal with, rather than as a principle.

Reach an address the operator did not name. Reach a second address for any
reason, including a lookup, an update check, a licence check or a report.

Send anything outward that its declaration does not list under `outbound`.

Be on in a fresh installation, or be turned on by an upgrade.

Write a record by any route other than the single write path the record module
owns, so that everything a connector brought in is in the change log with the
connector as its cause. `docs/decisions/0004-change-log.md` is where that rule is
recorded and this is an instance of it rather than a new rule.

Hold a credential outside the secret handling of #91, or hold one credential that
covers more than the connection it belongs to.

Reach the platform of another connector. Two connectors that need to talk to each
other are one connector with two destinations, declared as two destinations.

## The declaration

One table in the connector crate's own `Cargo.toml`:

    [package.metadata.connector]
    id = "mailbox-imap"
    purpose = "match messages in the mailbox the operator named to the records they concern"
    destination = "the IMAP server named in this connector's configuration, and no other address"
    outbound = "none"
    inbound = "message metadata, being sender, recipients, subject and time"
    credential = "the mailbox credential the operator entered, scoped to this connector"

Six keys, all required, and the set is closed. `outbound` and `inbound` take the
literal `none` where nothing crosses in that direction, because an answer and an
absence are different things and only one of them can be reviewed.

It sits in the manifest rather than in a file of its own for two reasons. It is
next to the dependency list that authorises the reaching, so a connector growing
a second destination and a second client crate cannot move one without seeing the
other. And cargo already carries it: `cargo metadata` prints the table, so the
operator facing documents of #100 and #106 can be generated from what the build
resolves rather than from a register somebody keeps in step by hand. That is the
requirement those two issues carry, and it is met by the declarations being the
only place the answer is written down.

## What refuses a departure from this

`kontor-connector` holds the rules and `server/crates/connector/tests/boundary.rs`
runs them, in the default suite, with nothing stubbed because there is nothing to
stub. Eight refusals, each named:

    declaration-missing
    declaration-without-a-connector
    key-missing
    key-unknown
    key-repeated
    value-blank
    id-does-not-match-the-crate
    line-not-a-declaration

The tests are of two kinds and the difference is the point of having both. Most
of them judge manifests written in the test, so what they prove is that the rule
bites. Two judge the manifests this repository actually holds, so what they prove
is the state of the tree. A suite with only the second kind goes on passing
forever the moment the rule stops working, because a tree with no connector in it
earns no refusal either way.

What the rules do not cover is stated here rather than left to be found.

A connector crate that is not named `kontor-connector-something` is not seen. The
check identifies a connector by its name, because it has to be able to ask the
question of a crate that has declared nothing, and a crate that has declared
nothing has only its name. A reader is the whole mechanism for that case. The
opposite direction is covered: a declaration in a crate outside the naming rule
is refused, so the way to escape the check is to name a connector something else
entirely rather than to half rename one.

Nothing reads the declaration against what the connector's code does. `outbound
= "none"` beside a client that posts a record is refused by a reader or not at
all. The declaration is a statement somebody makes and the check is that it was
made, in full, in the closed vocabulary. That bound is the same one
`licences-allowed` carries and it is stated for the same reason.

Nothing yet generates operator facing documentation from the declarations. #100
and #106 hold that, and the declaration exists in this shape so that when they
land the two cannot disagree.

The reader is not a TOML parser. It reads `key = "value"` and refuses every other
line in the table by name, so a declaration written with a multi line string, an
inline table or an escaped quote is valid TOML and is refused here. It fails
closed, which is the direction this check has to fail in.

## The case this record does not close

Where a service's own design forces data outward, a general sentence is not
enough and the case is named with what necessarily leaves. Push delivery is the
standing example and #78 holds it: a notification carries an identifier and no
customer data, and the platform still sees that a message went to a device.

That is not an exception to the rules above. It is a destination like any other,
declared like any other, and the honest part is that its `outbound` is not
`none`. #100 is where the operator reads the whole list, and a route that cannot
be closed belongs on that list rather than in a footnote to it.

## What this record does not decide

Whether message bodies are copied onto the host or only referenced. That is an
entry in #129 and it changes what a mailbox connector's `inbound` says rather
than whether it has to say it, which is why this record does not wait on it.

Whether any feature may send data to a service outside the host at all. Also an
entry in #129. The rules above are written so that either answer is a
configuration the operator makes and a declaration a reviewer reads, rather than
a rewrite of this record.

Which connector is built first, and what its dependency edges are. #83 is the
first one and it decides them. `kontor-connector` depends on nothing today
because it holds the vocabulary and the refusal, and a dependency taken here to
anticipate a connector would be a placement decided by this record instead of by
the work.
