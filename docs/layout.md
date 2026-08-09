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
    coverage
    docs
    format
    lint
    prove-determinism
    prove-headless
    prove-quality
    prove-sealed
    regenerate
    rust-toolchain.toml
    server
    test
    test-needs-a-database
    test-scan
    test-sealed
    text-scan

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
below. No TypeScript lives here. `server/fixtures/` holds the files a gate has
to refuse and their one-change neighbours, which is why it sits beside
`server/crates/` rather than inside it: a fixture is not a module. The lint
fixture there is a workspace member even so, because inheriting
`[workspace.lints]` is the whole point of it, and every arm that must be refused
is behind a feature nothing but `./prove-quality` asks for.

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

The same division puts each formatter's and each linter's configuration at the
root of the workspace it judges, so `server/rustfmt.toml`, `client/.prettierrc.yml`,
`client/.prettierignore` and `client/.oxlintrc.json` are all one level down.
Both formatters would find a file at the repository root by walking up, so this
is a statement about which layer a setting belongs to rather than about what the
tool can locate. A setting that ever applied to both layers at once would be the
case for moving one up, and there is none.

`build` at the root is the one command that takes a fresh clone to a compiled
tree, and the build workflow runs that script rather than restating its steps.
It is POSIX shell, which `docs/decisions/0001-means.md` allows as a dependency
that is argued where it is added rather than as a second language, and the
argument is that a build driver cannot be written in the toolchain it exists to
invoke. It runs the toolchains' own verbs in order and adds no build rules of
its own, so it is not the third party build system that record refuses. It also
refuses a toolchain that does not match the pins, which is the only thing
standing between `.nvmrc` and a build that quietly used a different Node.

`format`, `lint` and `prove-quality` sit beside it at the root, because each one
covers both layers and a script covering both belongs above either. The first
runs each toolchain's own formatter, writing or only checking, and holds no
formatting opinion of its own: a rule stated there would be one neither
formatter reads. The second runs each toolchain's own linter at a level where a
warning fails, and holds no rule of its own for the same reason, with one
exception it does not state either. That exception is `client/suppression-scan`,
one layer down, which the lint script calls. The third is what says all four
gates still refuse what they say they refuse.

`test`, `test-needs-a-database`, `test-scan`, `prove-headless` and `coverage`
sit beside them, and the second name is the rule: a suite is named for what it needs rather than for what it
covers, because a name like "integration" tells nobody reading a red run what to
install. The split between them is made by cargo and not by a convention. A test
target declares `required-features` in its crate's manifest and cargo does not
build a target whose required features are off, so `./test` leaves the marked
targets out without an argument anybody has to remember. `test-scan` is the
other half of that: it refuses an unmarked test target whose source reaches for
one of the four things a test here can need, and `./test` runs it before
anything is compiled so that the refusal arrives with the cause rather than as a
connection error in the middle of a run. `prove-headless` builds a throwaway
workspace per case and says the scan still bites. `coverage` runs the first
suite under instrumentation and refuses below a floor that is a measurement
rather than a target, with the run that produced it in a comment beside the
number.

`test-sealed` and `prove-sealed` sit beside them and are the other direction.
Everything above judges the source of a test; these two judge the environment it
runs in. The first puts the suite in a network namespace of its own, where the
only interface is a loopback that is down and there is no route out of it, built
with privilege and entered with less than the run that started it, and it reads
`/proc` before it starts rather than trusting the commands that were meant to
make it that way. It compiles outside the seal and runs inside it offline, because the
registry is on the far side and the claim is about the suite rather than about a
cold clone. The second is a set of pairs: each leg runs one command on each side
of the seal and reads both exit codes, so a seal that stopped sealing reddens
the inside half instead of leaving a green run that means nothing. Both are
Linux only, and both say so and stop rather than passing where the mechanism
they depend on does not exist.

`text-scan`, `regenerate` and `prove-determinism` sit beside it at the root, for
the reason the top of this note gives: a contributor runs them, and a rule
stated inside a `run:` block is a rule nobody can run before pushing. The first
refuses a carriage return and anything that is not UTF-8 in tracked text,
reading the index rather than the working tree. The second remakes every
generated file that is committed, and its comment header is the register of
which files those are; `regenerate --check` remakes them and refuses a
difference, which is what stops a hand edit to one from surviving. The third
proves all three rules bite, against fixtures it builds rather than against the
tree it lives in.

`client/escape-scan`, `client/suppression-scan` and `client/prove-gates` are the
same kind of thing one layer down, and they sit beside the workspace they judge
rather than at the root. The first refuses an escape from the client type system
that carries no reason. The second refuses a lint suppression that names no rule
or gives no reason, and it is a script rather than a setting because the linter
this workspace can install has no rule for either half; the comment at its top
carries the command that says why that linter is the one it is.
The third is what says both client type gates still bite, by running the shipped
commands against fixtures and reading the exit codes, so a setting loosened
where it lives reddens the leg that depended on it rather than passing quietly.
The fixtures it judges sit in `client/packages/app/fixtures/`, which is outside
the `src` that `tsconfig.json` includes, so the file that is meant to be red
cannot redden the application's own build.

## The direction dependencies run inside the server

The server modules form a stack, and the whole point of the stack is that the
arrows all point one way.

Beside the stack rather than in it is the money module, which depends on nothing
inside the server and which every layer above may use. It holds an amount, a
currency, a conversion rate with the moment it was taken, a calendar date and a
moment in time, as the types `docs/decisions/0005-money-and-time.md` argues for
under #22. It is not a layer of the stack because it has no state and talks to
nothing; it is a vocabulary, and putting it in the stack would mean choosing a
level for something every level needs.

Beside it, and also not in the stack, is the connector module. It depends on
nothing and nothing depends on it, because what it holds today is the declaration
every connector carries and the refusal that reads it, which
`docs/decisions/0011-connectors.md` argues for under #82. A connector itself is a
crate of its own, named so that the refusal can ask the question of one that has
declared nothing, and where those crates sit in the stack is decided by #83
rather than here: an edge added now to anticipate the first connector would be a
placement chosen by this note instead of by the work.

At the bottom of the stack itself is the store, which owns the tables and is the
only thing that talks to the database. Above it the metadata layer, which holds objects, fields,
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

Nothing in the stack depends on the API layer. It is the top, so a change to a
request shape can never reach the record module.

This sentence said the API layer was the only module with no dependents inside
the server, and something does depend on it. `kontor`, the binary, depends on
`kontor-api`, which is what a composition root is for. The stack is the library
modules; the binary sits outside it, nothing may depend on the binary, and it
reaches the stack through the top and nowhere else, so composing the server
cannot become the second route into the record module that this section exists
to refuse. The narrowing was made when the rule stopped being a sentence and
became a command, because a check written from the old wording refuses an edge
that is correct, and it was found by writing that check.

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

One of those directions is refused by a command rather than by a reader:

    ./layout-scan
    ./prove-layout

The first reads the manifests under `server/crates/` and refuses an edge this
section does not allow: one pointing up a level, one pointing sideways between
reporting and workflow, one into the composition root, one into the connector
module, one out of the money module, an edge to a crate that is not there, and a
crate this section does not place at all. The last of those is what makes the
gate fail closed on a module somebody adds without deciding where it sits. The
second is the evidence that each of those refusals bites, against fixture graphs
it builds rather than against this tree, because a crate added here pointing the
wrong way would either be refused by the gate it is meant to prove or would need
an exclusion, and an exclusion list is the thing that quietly grows.

What that pair does not reach is worth stating rather than leaving to be found.
Its subject is the crate directories under `server/crates/`, so a workspace
member declared from some other path is outside it, and `server/Cargo.toml`
holds one today in `fixtures/lint`, which is the lint gate's subject rather than
a module of the stack. Closing that would mean deciding where a module may live,
which is a wider decision than adding the check. The other three directions in
this section are assertions about calls rather than about edges between crates,
and no command reads any of them.

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

Also enforced, since this note first said none of it was: the direction the
dependencies run between the crates under `server/crates/`, by `layout-scan`,
whose refusals are the ones the section on the stack lists and whose evidence is
`prove-layout`. That covers two of the three refused directions there, since
nothing depending on the API layer and reporting and workflow not depending on
each other are both statements about an edge. The separation of reporting from
workflow was in the list below until this commit and belongs above it now.

Written down only, meaning a person is the whole mechanism: the refusal of any
write path that reaches the record tables without writing the log; the rule that
the API layer holds no business rule; the rule that the client depends on the
generated contract and not on server internals; and every statement above about
which directory a kind of file belongs in. Each of those is a rule about a call
or about a placement rather than about an edge between two crates, which is why
the check above does not reach them.

What the tree does check is a different set, and it is worth naming so that the
two are not confused. Which workflows exist is printed rather than pasted here,
for the reason the section above gives about lists in documents: this one was
pasted, and it was two files out of date before the file that made it three was
added.

    ls .github/workflows/

Those judge sign off, dependency advisories, supply chain hygiene, line endings
and encoding in tracked text, dangerous Unicode, the workflow files themselves,
whether both layers compile against the pinned toolchains, whether the client
type checks under strict mode, whether an escape from the client type system
carries a reason, whether both layers are formatted the way their formatters
want them, whether either linter has anything to say at a level where a warning
fails, whether a lint suppression names its rule and gives its reason, whether
both suites are green, whether every test that reaches for a display, a network,
a simulator or a database is marked for it, and whether the unit suite still
reaches as much of the server as the floor in `coverage` says. One of them reads
a module boundary, and it is the one added with `layout-scan`.

Cargo is still the wrong thing to lean on here, and the distance is worth
keeping stated rather than blurred now that something else covers it. Cargo
refuses a dependency cycle, so the arrows cannot be made to point both ways at
once. It does not refuse an arrow pointing the wrong way: a line added to
`server/crates/reporting/Cargo.toml` making the reporting engine depend on the
workflow engine compiles. What refuses it is `layout-scan`, and `prove-layout`
has that exact line as one of its legs.

Two issues would move what is left of the second list to the first. #116 turns
the architecture rules into tests and holds the three that remain, which are the
single write path, the one module that evaluates a permission, and the client
depending only on the generated contract; each of those is about a call rather
than about an edge, and each waits on a module that is empty today. #113
enforces the invariants that are greppable, which is the cheaper half and covers
the shapes that a search can refuse.

Until those land, the rest of this note is an explanation of the rules and not
the rules themselves, and a change that violates one of them will be caught by a
reader or not at all.
