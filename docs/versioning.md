# What a version number promises

A version number is a promise about compatibility, and one made loosely is worse
than none, because operators plan around it. This document says what the number
this project issues means, before the first one is issued.

## The short answer, for an operator

The number on a release is about what an upgrade costs you, and nothing else.

A change in the first part means there is something you have to do, and the
release notes say what. A change in the second means new behaviour and no action
on your side. A change in the third means fixes only.

The number is not the version of the API your integration is written against, and
it is not the version of the protocol your phone speaks. Those two have versions
of their own, they move on their own schedule, and each of them tells you its own
version at the moment you ask. Nothing about this number lets you predict either.

## Why the number cannot mean one thing

Five things change independently in this product and an operator cares about each
of them separately.

The API an integrator writes against. The sync protocol a client speaks. The
database schema underneath. The configuration format the operator edits. And the
metric definitions the reports are computed from.

A single number that tried to cover all five would move for whichever of them
moved, and an operator reading it would learn that something changed without
learning which. Worse, it would be read as a promise about the two surfaces that
already have compatibility promises of their own, and a number that appears to
promise something it does not is the failure this document exists against.

So the number covers exactly one thing, being the cost of the upgrade, and the
five surfaces each declare their own compatibility. What holds them together is
the release notes, which say for every release which of the five moved.

## The scheme

Three parts, separated by dots, and each part is a whole number that only ever
increases.

    MAJOR.MINOR.PATCH

**MAJOR changes when the upgrade requires an act by the operator.** Not when
something internal was rewritten, not when a dependency moved, and not on a
calendar. The list of what counts is below and it is closed: an operator reading
it can tell whether a release will cost them an afternoon.

**MINOR changes when there is new behaviour and nothing to do.** A new endpoint,
a new field, a new report, a new action, a relaxed limit. Installing it and
changing nothing is a supported outcome.

**PATCH changes when nothing is added.** Corrections, including security
corrections, and nothing else. A PATCH release exists so that an operator who
cannot take new behaviour this quarter can still take a fix.

A pre-release is the same number with a hyphen and an identifier after it, as
`3.1.0-rc.1`, and it is a release an operator may install to test an upgrade
before it is general. It carries the same promise about the number and no promise
that it will be followed by an identical general release.

There is no fourth part and no build metadata. A number an operator cannot read
aloud over the telephone to somebody helping them is a number that gets written
down wrongly.

## What makes a release MAJOR

Closed, and each entry is a thing the operator has to do rather than a thing this
project finds inconvenient.

**A configuration key is removed, renamed, or has its meaning changed.** The
operator edits that file, so this is the entry they meet most often. Adding a key
with a default is not this; requiring one that had a default is.

**A metric definition changes.** This is the sharpest entry and the reason it is
not folded into the one above. A definition change makes a number an operator
reported last quarter different from the same number computed today, and nothing
about the instance looks different afterwards. `docs/decisions/0007-reporting.md`
requires every number to state the definition it came from, which is what makes
the change traceable at all, and the release notes name the definition and the
direction the number moves.

**A migration cannot be reversed.** `docs/decisions/0003-custom-fields.md` makes
dropping a column a separate act days after hiding the field, and the same shape
applies to a release: where an upgrade destroys something a downgrade would need,
the operator has to decide before it runs rather than discover afterwards.

**An API major version is retired.** Retiring one is what ends the support window
below, and the window is what makes it survivable.

**A sync protocol version this release no longer speaks.** A client that spoke
only that version stops syncing, which for a phone in a field somewhere is an
outage rather than a notice.

**A supported version floor rises.** The floors are stated in
`docs/decisions/0001-means.md` and a rise there is an act by whoever runs the
database or the host.

**A default changes in a way that changes behaviour on an existing instance.** A
default that only applies to a fresh installation is not this.

Not MAJOR, stated because each of these is asked about: adding a key, an
endpoint, a field, an action, a report or a protocol version; relaxing a limit;
correcting a defect, including one whose correction changes a wrong number to a
right one; and anything an operator can turn off.

## Where each of the five surfaces states its own version

Read from the records rather than restated here, because a copy of a rule in a
second document drifts against the first.

**The API.** `docs/decisions/0006-api-shape.md` carries the whole of it: a major
version in the path for the fixed core, the description version for the generated
part, a closed list of what is breaking and what is not, and the support window.
That record is the authority and this document adds nothing to it. What this
document adds is the join: a release that retires an API major is MAJOR here, and
a release that adds an endpoint is not.

**The sync protocol.** `docs/sync-protocol.md` negotiates per session. A client
offers the versions it speaks, most preferred first, and the server answers with
the one it chose, so a client learns what it is talking to rather than inferring
it from a release number. That document also states that the protocol version and
the description version never move together.

**The database schema.** It has no version an operator reads and it is not a
compatibility surface for anybody outside this software. What an operator sees of
it is the migration, which is #18, and the direction a restore can go, which is
#95. Neither is stated yet and both are named here so that the absence is
readable.

**The configuration format.** #90 owns it and there is no format yet. Until there
is, the MAJOR entry above is a promise about a file that does not exist, which is
the right order: the rule is written before the first key rather than after the
first rename.

**The metric definitions.** #36 makes a definition exist once and be served from
one place, which is what makes a change to one detectable rather than diffuse.
`docs/decisions/0007-reporting.md` is where the definitions are argued.

## The support window

The API's window is stated in the record that owns it, and it is read rather than
copied:

    git show origin/main:docs/decisions/0006-api-shape.md | sed -n '130,133p'
    A major version is supported for at least twenty four months after its successor
    becomes the default on a release of this project. The instance document names the
    deprecation date of a version being retired, so the promise is readable from the
    instance rather than from a document somebody has to find.

That covers an integrator whose code is written against a major. It does not
cover an operator asking how long the release they installed goes on receiving
fixes, and that half is not stated in this document. The reason is that it is a
commitment about how long somebody keeps working on a line of releases rather
than a property of any artefact, it is attached to a person in the same way the
store accounts and the review cycle in #129 are, and a document that named a
number here would be making that commitment on their behalf. #125 carries it, and
what is written down until it is answered is that no window is promised for a
release line and that an operator should read that as no promise rather than as a
short one.

What is decided here, because it is a rule rather than a commitment: the window,
once stated, ends only by the successor becoming the default, and never by a
release being superseded by a PATCH of its own line. An operator on a MAJOR that
is still supported may take a PATCH without taking the successor.

## What this document does not decide

The release route. Building, testing, signing, attesting, publishing and writing
the notes as one workflow with no manual step is the rest of #125, it needs an
artefact to publish, and #119 builds the artefact.

Which gates a release may not be published without. `docs/quality-parity.md`
argues which checks are merge conditions and which are advisory, the set standing
in front of the branch is #108, and a release gate written against an empty set
would be a green tick meaning nothing.

The form of the release notes. That they name whether a release contains a
breaking change and what an operator must do is a done condition of #125; how
they are generated from the merged work is part of the route above.

The support window for a release line, which is the paragraph above rather than a
gap somebody forgot.

## What refuses a departure from this

Nothing. This is prose, and no check in this tree reads a version number, a
release note or a configuration key. `docs-scan` judges that the references here
resolve and that the words are the ones the register names, which is a statement
about the bytes rather than about the promise.

What would give the MAJOR list force is a route that reads a release against it,
and there is no release. Until then the list is what a reviewer refuses a release
with, and this sentence is the whole of the enforcement.
