# What this project is not for

`NOTICE.md` says the general thing, which is that the software is for lawful use
and that the operator is responsible for their own deployment. This document is
the specific half, and it exists because a CRM is a product where the most
damaging uses are reachable with the shipped features and no modification at
all. Nothing here restates the notice or the licence, and where the three
overlap the licence is the one with legal effect.

It names classes of use rather than screens, because a screen that does not
exist yet cannot be listed and a class survives the feature that arrives next
year.

## The uses this project does not support

### A profile database of people who never dealt with the operator

The product stores people, organisations and the activity between them. It is
for recording a relationship the operator actually has. Assembling records about
people who have not dealt with the operator, from a purchased list, from scraped
pages, from an enriched export or from a mailbox that happened to contain them,
is outside what this is for. The line is whether the person is somebody the
operator deals with, not whether the record is well formed.

### Automated contact at a volume, or a repetition, that is harassment

The workflow engine will send mail on a trigger and a report will be deliverable
by mail. That makes an instance a sending system, and a sending system pointed at
a list of people who did not ask to hear from the operator is the case this
project does not support. Volume is one half of it and repetition is the other:
contacting one person who has said no, on a schedule, is the same misuse at a
count of one.

### Scoring an individual to decide something about their employment

The reporting engine computes attainment against a quota per person, and a
coverage ratio, and a fitted expectation. Those numbers are for managing a
pipeline. They are an arithmetic summary of a sales process with somebody's name
attached to it, and they are not a measurement of a person's performance, their
competence or their future. Using them as the input to a decision about hiring,
promotion, discipline or dismissal is a use this project does not support, and
the arithmetic being correct is not a defence of the decision.

### A store for special category data about people

A custom field is a real column, so an operator can define a field and put
anything in it, including a person's health, their trade union membership, their
religion, their sexual orientation or their ethnicity. The product will accept
it. It is not built for it, it has no controls proportionate to it, and the
moment such a field exists the operator has changed what their instance is.

## What the software does not do about any of this

Nothing. There is no control anywhere in this product that refuses any of the
four, and this section exists so that no reader takes the section above as a
description of a mechanism.

The permission model decides which people inside the operator's own organisation
may see and change which records. It has no opinion about whether a record
should exist, and it says nothing about the person the record is about.

The change log records who changed a field, when, and what caused the change. It
does not record why a person is in the database, and it cannot distinguish a
record typed from a business card from one imported out of a purchased list.
Both arrive as a create.

The import path validates shape. It checks that what arrives can be stored
without being mangled, which is a different question from whether it should have
been collected.

The sending work is about not becoming a technically bad sender, meaning a
configured relay, per recipient and per period limits, bounce handling and
unsubscribe handling. A rate limit is not consent. An instance that stays within
every limit and sends to people who never agreed to hear from it has passed
every check this project will ever run.

The reporting engine will show the query behind every number, which makes a
figure checkable. It does not make the decision somebody takes from that figure
a good one, and nothing in the product knows what the figure was used for.

## Where the operator's own obligations are easiest to breach

Two features make it easy, and both are named here rather than described in
general terms.

### Mailbox ingestion

Connecting a mailbox turns a stream of correspondence into records, and
correspondence contains people who never dealt with the operator: everybody
copied on a thread, everybody who wrote once to ask a question, everybody in a
forwarded chain. Whether message content is copied into the instance or only
referenced is not settled and is one of the open questions in #129, so this
document is written to be true under either answer. Under both, the operator is
the one who decides which mailboxes, which folders and which direction are in
scope, and the operator is the one who has to be able to say on what basis each
resulting record exists. The connector work under #83 makes the narrow default
and the visible rule its own requirement. It does not make the answer for the
operator.

### Bulk sending

The moment a workflow sends to a list rather than to one person, the operator is
running a campaign, and every obligation that attaches to unsolicited contact in
their jurisdiction attaches to them and not to this project. The unsubscribe
handling under #84 is a mechanism, not a discharge, and that issue requires the
documentation to say exactly that.

Two more are worth naming because they look like features rather than
obligations. Telephony under #86 logs a call and does not record it by default,
and turning recording on is a decision with consent obligations attached in most
places it would be turned on. Activity capture under #69 records what staff did
and when, and an operator who reads it as an attendance record has built
something they have to be able to justify.

## The state of all of this today

None of the features named above exists. The tree holds two modules with
behaviour in them and a set of crates that compile and do nothing, which is
readable rather than asserted. Read against
`6f894536d33f7cab3eb0594e851d6d17b550d045`, the tip of the default branch as
this was written:

    for p in $(git ls-tree -r --name-only 6f89453 -- server/crates client/packages \
                 | grep -E '\.(rs|ts)$'); do
      printf '%6s %s\n' "$(git show "6f89453:$p" | wc -l)" "$p"
    done
        12 client/packages/app/fixtures/checked-index.ts
        14 client/packages/app/fixtures/unchecked-index.ts
         7 client/packages/app/src/main.ts
         6 server/crates/api/src/lib.rs
         7 server/crates/kontor/src/main.rs
       403 server/crates/metadata/src/definition.rs
       295 server/crates/metadata/src/field_type.rs
        23 server/crates/metadata/src/lib.rs
       574 server/crates/metadata/tests/definitions.rs
       490 server/crates/money/src/lib.rs
       358 server/crates/money/tests/properties.rs
         4 server/crates/record/src/lib.rs
         4 server/crates/reporting/src/lib.rs
         5 server/crates/store/src/lib.rs
        80 server/crates/store/tests/needs_postgres.rs
         4 server/crates/workflow/src/lib.rs

So every sentence above is about a product being built rather than one being
misused, which is the point of writing it now. A statement of what a thing is
not for is worth something before the thing exists and is an apology afterwards.

## How this stays true as features are added

A new feature is measured against this document with one question. Does it make
one of the four uses above easier to reach, cheaper to reach, or harder to see
once it is happening?

Where the answer is yes, the issue that adds the feature says so and says what
it does about it, and if it does nothing, it says that instead. That is a rule
about how the work is argued rather than a check that refuses anything, and
nothing in this tree reads it. There is no mechanism here, and the sentence
saying there is none is the whole of the enforcement.
