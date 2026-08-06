# Repository layout

Two toolchains, two engines and several clients will produce a tree in which
nobody can guess where a change belongs. This note says where things go and why
the boundaries fall where they do, so that a later change either fits or argues
with the reason rather than with a convention.

The layout described here is the one the tree is being built towards. What the
tree holds today is smaller, and reading it is one command:

    git ls-tree --name-only HEAD
    .gitattributes
    .github
    .gitignore
    .nvmrc
    CONTRIBUTING.md
    DCO
    LICENSE
    NOTICE.md
    README.md
    SECURITY.md
    build
    client
    docs
    rust-toolchain.toml
    server

`server/` and `client/` are there since #2, holding workspaces that compile and
nothing more. `mobile/`, `tests/`, `benchmarks/` and `fuzz/` are still names in
this note rather than directories, so part of what follows describes a shape the
tree is being built towards. Where the tree and this note disagree, one of the
two is corrected in the change that creates the disagreement rather than later.

## The top level

`.github/` holds the workflows and the tracker templates. Nothing that a
contributor runs locally belongs here; the commands live in the toolchain
manifests, and the workflow calls them rather than reimplementing them in a
`run:` block. Logic that grows past a few lines of shell in a workflow is a sign
it belongs in the tree as code that the workflow invokes.

`docs/` holds prose for contributors and operators. `docs/decisions/` holds the
decision records, whose numbering and naming are fixed in
`docs/decisions/0001-means.md`. A document under `docs/` describes something
that exists or something that is being decided; it does not hold configuration
that a program reads.

`server/` holds the Rust workspace. Its crates are the server modules described
below. No TypeScript lives here.

`client/` holds the TypeScript web client. It depends on the generated contract
and never on anything under `server/`. No Rust lives here.

`mobile/` holds the shared offline core and the native shells, split the way
`docs/decisions/0010-mobile.md` argues for under #71. The core is the permanent
part; a shell is the reversible part.

`tests/` holds suites that cross a module boundary and therefore have no single
crate to live in: the sync conformance suite of #74, the headless proof of #117,
the restore proof of #95. A test that exercises one module belongs beside that
module, not here. A test that needs a database, a browser, a simulator or a live
network belongs in the marked set defined by #7, wherever it sits.

`benchmarks/` holds the measured budgets, which are the reporting budget of #46,
the workflow limits of #61 and the client budget of #81. A benchmark states the
machine it was measured on, because a number without one is not a budget.

`fuzz/` holds the fuzz targets of #111, which cover the surfaces that take bytes
from strangers: the import parser, the API request path, the expression language
and the sync protocol decoder.

The two toolchain pins sit at the repository root, because that is where the
tools that read them look: `rust-toolchain.toml`, which rustup applies to every
cargo invocation inside the tree, and `.nvmrc`, which nothing applies on its own.
Each workspace's own manifest and lock file sit at the root of that workspace
rather than at the root of the repository, which is `server/Cargo.toml` with
`server/Cargo.lock` and `client/package.json` with `client/package-lock.json`.
The pins are one repository-wide fact; the manifests are two workspaces that
happen to live in one tree.

`build` at the root is the one command that takes a fresh clone to a compiled
tree, and the build workflow runs that script rather than restating its steps.
It is POSIX shell, which `docs/decisions/0001-means.md` allows as a dependency
that is argued where it is added rather than as a second language, and the
argument is that a build driver cannot be written in the toolchain it exists to
invoke. It runs the toolchains' own verbs in order and adds no build rules of
its own, so it is not the third party build system that record refuses. It also
refuses a toolchain that does not match the pins, which is the only thing
standing between `.nvmrc` and a build that quietly used a different Node.

## The direction dependencies run inside the server

The server modules form a stack, and the whole point of the stack is that the
arrows all point one way.

At the bottom is the store, which owns the tables and is the only thing that
talks to the database. Above it the metadata layer, which holds objects, fields,
types and validation as run time data and turns a change to them into a reviewed
migration under #18. Above that the record module, which owns the single write
path: every write to a record and its history commits in one transaction, which
is the rule `docs/decisions/0004-change-log.md` records under #15.

Reading from that, and writing nothing, are the reporting engine and the workflow
engine. Both read the change log. Neither writes it. This is the property the two
differentiators stand on, and it is what makes a workflow unable to fire on a
change reporting cannot see, and reporting unable to show a change no workflow
could have seen.

At the top is the API layer, which holds no business rule of its own. It parses,
it authenticates, it authorises against the one permission model of #21 and #27,
and it calls down. A rule that appears in the API layer is a rule that the record
module cannot enforce, which means something reaching the record module by
another route can break it.

Three directions are refused.

Nothing depends on the API layer. It is the top of the stack and it is the only
module with no dependents inside the server, so a change to a request shape can
never reach the record module.

Nothing reaches the record tables except through the write path that writes the
log. Not the import module, not a workflow action, not a migration that thinks it
is only fixing data. This is the refusal #15 names, and it is the one whose
violation is silent: a write that skips the log leaves a record whose history is
wrong, and nothing about the record looks wrong afterwards.

Reporting and workflow do not depend on each other. They share the log and
nothing else. A metric that a workflow needs is read the same way any other
consumer reads it, through the definition of #36, and never by calling into the
reporting engine's internals.

The client side has one direction of its own. The client packages depend on the
generated contract and never on server internals, which is what makes the
contract of #25 worth generating rather than writing twice.

## Where the four recurring additions go

This plan adds four things repeatedly, and each has one home.

A new object type is normally not a code change. An operator adds a custom object
through the metadata layer while the instance is running, and the migration is
generated. Code changes only for a structural object, meaning one of the five
that #16 says an operator may not remove, and then the change is a built in
definition in the metadata module plus the migration that follows from it.

A new metric is one definition in the metric layer, which #36 requires to be
defined once and served to every consumer from that definition. A metric defined
in a report, in a client component or in a workflow condition is the defect that
issue exists to prevent, and there is no second place to put one.

A new workflow action goes in the action module, registered in the one place the
action set of #54 is assembled, carrying its own input contract and its own
failure behaviour. An action that calls out of the host is bound by #55 and by
`docs/decisions/0011-connectors.md` under #82, not by the action module alone.

A new connector goes under the connector module as its own unit, and what it may
and may not do is decided by #82 rather than by the connector. The rule that
makes this a boundary rather than a folder is that a connector reaches the rest
of the server through the same write path everything else uses.

## What prints the modules

This note does not list them, because a list in a document drifts against the
thing it describes and the drift is invisible until somebody trusts the list.

    cargo tree --manifest-path server/Cargo.toml --workspace --depth 0

    npm ls --workspaces --depth 0 --prefix client

Neither needs a tool beyond the two toolchains a contributor already has to have
installed, which is why they are these commands rather than a `cargo metadata`
or `npm query` pipeline through a JSON filter this tree does not require anyone
to have.

The same tool prints the direction the dependencies run, which is the rule the
next section is about, and prints it out of the compiled graph rather than out
of anybody's memory:

    cargo tree --manifest-path server/Cargo.toml --package kontor

The commands are named here rather than their output, so that this note keeps
working when the output changes.

## What is enforced and what is only written down

Almost nothing in this note is enforced by a check today. The distinction below
is the honest one rather than the flattering one.

Enforced by a check: that the crates named in `server/Cargo.toml` and the
packages named in `client/package.json` compile, which the build workflow
refuses a breach of, and with it that no cycle exists among the server crates,
which cargo refuses on its own. Neither of those is the direction rule. A cycle
is refused; a single arrow pointing the wrong way is not.

Written down only, meaning a person is the whole mechanism: the direction
dependencies run; the refusal of any write path that reaches the record tables
without writing the log; the separation of reporting from workflow; the rule that
the API layer holds no business rule; the rule that the client depends on the
generated contract and not on server internals; and every statement above about
which directory a kind of file belongs in.

What the tree does check is a different set, and it is worth naming so that the
two are not confused:

    ls .github/workflows/
    build.yml
    dco.yml
    dependency-review.yml
    scorecard.yml
    text-determinism.yml
    unicode-guard.yml
    zizmor.yml

Those judge sign off, dependency advisories, supply chain hygiene, line endings
and encoding in tracked text, dangerous Unicode, the workflow files themselves,
and whether both layers compile against the pinned toolchains. None of them
reads a module boundary.

The build one is the closest, and the distance is worth stating rather than
blurring. Cargo refuses a dependency cycle, so the arrows cannot be made to
point both ways at once. It does not refuse an arrow pointing the wrong way: a
line added to `server/crates/reporting/Cargo.toml` making the reporting engine
depend on the workflow engine compiles, and this note is the only thing that
says it may not. #116 is where that becomes a test.

Two issues would move items from the second list to the first. #116 turns the
architecture rules into tests, which is where the dependency direction and the
single write path belong, because both are properties of the compiled graph
rather than of the text. #113 enforces the invariants that are greppable, which
is the cheaper half and covers the shapes that a search can refuse.

Until those land, this note is an explanation of the rules and not the rules
themselves, and a change that violates one of them will be caught by a reader or
not at all.
