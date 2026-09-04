# Contributing

This guide is written to be run rather than admired. Every rule below names the
command that shows whether the rule is met, and where no such command exists yet
the guide says so instead of implying one.

## Before you push

    ./build

That builds both layers from a fresh clone and is the same script the build
workflow runs, so a verdict here and a verdict there come from one procedure
rather than two.

It needs three things installed first, and it refuses rather than working around
any of them.

`rustup`, not a Rust toolchain installed some other way. The exact compiler is
pinned by `rust-toolchain.toml`, rustup is what applies that file, and a `cargo`
that did not come from rustup builds with whatever version it happens to be.

Node, at the version `.nvmrc` pins. Nothing applies that file on its own, so the
script compares it against the `node` on your path and stops on a mismatch.
`nvm use` in this directory is the usual way to satisfy it. Editing `.nvmrc` to
match your machine is not, because the pin is the point.

A POSIX shell. On Windows that is the one that ships with git.

The client type gate is its own command, because a check that only runs as a
side effect of building cannot be run without producing artefacts:

    npm run typecheck --prefix client

It emits nothing and it is the same command the client type workflow runs.
Building already type checks, since `tsc` cannot emit without doing so; this is
the way to ask the question on its own.

An escape from the type system is allowed in client source and an unjustified
one is not. Write the reason on the same line as the escape, as `reason:`
followed by the reason, so that moving the escape moves its justification with
it. The check is a script rather than a block inside a workflow, so you can run
it before pushing and see the same verdict:

    client/escape-scan

It refuses a line that escapes and carries no reason, and it refuses an empty
`reason:` as if it were absent.

Both of those gates are only worth what their evidence is worth, so the evidence
is a command too:

    client/prove-gates

Every leg runs a shipped command against a fixture and reads the exit code. A
type error is refused, the same fixture passes with that one setting switched
off, its corrected neighbour passes, an unreasoned escape is refused, an empty
reason is refused as an absent one, and a reasoned escape is accepted. Loosen a
setting where it lives and the leg that depended on it goes red, which is the
whole reason the legs do not restate any rule of their own.

Line endings, encoding and the generated files have three commands of their own,
and they read what git holds rather than what is in your working tree, because a
clone with `core.autocrlf` set has a working tree that differs from the index by
line ending and only one of the two is what anybody else will check out:

    ./text-scan
    ./regenerate --check
    ./prove-determinism

The first refuses a carriage return in tracked text and anything that is not
UTF-8 without a byte order mark. The second remakes every generated file that is
committed and refuses a difference, so a hand edit to one fails rather than
surviving; the register of which files those are is the comment at the top of
`regenerate`, and `./regenerate` on its own remakes them without judging. The
third is the evidence that all three refusals work, against fixtures it builds
rather than against this tree.

The licence is in the tree rather than only at its root, and four more commands
are about keeping it there:

    ./licence-scan headers
    ./licence-scan dependencies
    ./licence-apply
    ./prove-licences

The first refuses a tracked source file that does not carry this project's SPDX
identifier on its first line, or under its shebang where it has one. The
identifier is read out of `server/Cargo.toml` and `client/package.json` rather
than written into the check, and a disagreement between those two is refused
before any file is judged, because a check comparing against one of two answers
passes a tree that has already lost the argument. The second reads what every
package in both toolchains declares about itself and refuses one whose licence
is not covered by `licences-allowed`, a file carrying a reason for every entry
and refusing an entry that carries none.

`./licence-apply` is the tool that writes a missing header, so the header has
one spelling rather than one per contributor, and it is idempotent. It never
rewrites a header naming a different licence: that is somebody's statement
rather than a typo, and `licence-scan` names those files for a person to decide
about. `./prove-licences` is the evidence that all of it refuses what it names,
against fixture repositories it builds rather than against this tree.

What the dependency rule cannot do is read a licence text or judge whether a
declaration is true, and `docs/licence.md` says so in the same place it states
what the licence asks of an operator.

Adding a binary file means declaring it in `.gitattributes` with git's `binary`
macro. Nothing is exempt from the encoding rule by detection, which is
deliberate: a detector reading a mangled file as binary would exempt exactly the
file the rule exists for.

What may not be in the tree at all is a credential, and two commands are about
that:

    ./secret-scan
    ./prove-secret-scan

The first refuses a tracked file carrying a shape a credential usually has: a
private key block, credentials inside a URL, a quoted literal assigned to a name
that means a credential, and the token formats some services issue in a shape
nothing else has. Which shapes those are and how many the tree carries are
printed by the command rather than written here. A value written into a file and
committed is in every clone and in every fork, and it stays in the history after
somebody takes it out of the file, so rotating it at the service that issued it
is the only repair and the cheap moment to catch one is before it is pushed.

`secrets-allowed` is where a value that has that shape and is not a credential
is declared, one line per value with the reason on the line above it, in the
shape `licences-allowed` already uses. It fails closed in both directions: an
entry with no reason is refused, and so is an entry no tracked file answers, so
taking the last use of a value out of the tree reddens the entry that declared
it and the repair is to take the entry out in the same change.

The refusal names the file, the line and the shape, and never the value. A check
that quoted what it found would put the credential into the log of every run
that refused it, which is the failure it exists against arriving through the
door marked evidence. `./prove-secret-scan` is the evidence, against
repositories it builds rather than against this tree, and one of its legs reads
what a refusal did not say rather than only the code it exited with.

Three things it does not reach, said plainly because a green run says nothing
about any of them. A secret that looks like nothing passes: the shapes are a
floor holding what a credential usually looks like rather than a statement that
the tree carries none. Whether a declared value is live is somebody's judgement
and no reading of the tree checks it, which is what the reason beside each entry
is for. And it reads the tree rather than the history, so it refuses the commit
that adds a credential and says nothing about one that already landed.

A connector carries two things, and only one of them is code. Two more commands
are about the other one:

    ./connector-guide-scan
    ./connector-guide-scan --write
    ./prove-connector-guides

`docs/decisions/0011-connectors.md` decided that a connector declares what
crosses the boundary in its own manifest, and the default suite refuses one that
has not. That is what a reviewer reads. An operator deciding whether to connect
their mailbox reads prose instead, and #89 is where that is asked for: one guide
per connector, at `docs/connectors/<id>.md`.

The first refuses a connector crate with no guide, a guide naming no connector,
a guide missing a heading, and a guide whose boundary block is not what the
declaration says. The headings are read out of
`docs/connector-guide-template.md` rather than written into the check, so a
heading added there is required from that commit. The boundary block is
rendered from what cargo resolved out of the manifest, so the guide cannot say
something the code does not, and `--write` is what puts it there: a hand edit is
refused rather than kept. The third is the evidence, against fixture trees it
builds rather than against this tree, which is the whole evidence today because
no crate here is named as a connector and the scan prints that it judged
nothing.

What neither can judge is whether a guide is true. That a heading is present is a
fact about the bytes; that what is written under it describes this connector is
not, and the declaration carries the same bound in its own record. A reader is
the whole mechanism for both halves.

Formatting and lint are two more commands, and they are two rather than one
because they answer different questions. A formatter makes the tree consistent;
a lint gate refuses shapes that are consistent and still wrong.

    ./format
    ./format --check
    ./lint
    ./prove-quality

The first formats both layers. The second says whether they are formatted and
writes nothing, which is what the workflow runs, and either of them takes
`server` or `client` to do one layer. `./lint` runs every leg and takes
`server`, `client` or `suppressions` to run one. The last is the evidence that
all four gates refuse what they say they refuse, against the fixtures under
`server/fixtures/` and others it builds as it goes.

Do not argue with a formatter default. Both toolchains have one that is
defensible and it is taken, so that the argument does not have to be won. Where
a default is not taken the deviation lives in that layer's configuration file
with the reason on the line above it, which is `server/rustfmt.toml` and
`client/.prettierrc.yml`, and a deviation with no reason above it is the thing
to send back in review.

The lint gate runs at a level where a warning fails. That level is set once per
layer, in `server/Cargo.toml` under `[workspace.lints]` and in
`client/.oxlintrc.json`, and never on a command line, so your terminal and the
workflow reach the same verdict.

A suppression is allowed and an anonymous one is not. Name the rule and give the
reason in the same place as the suppression, so that moving it moves its
justification with it:

    #[expect(dead_code, reason = "the trait is implemented for #29 and used there")]

    // oxlint-disable-next-line no-await-in-loop -- the calls must be ordered

On the server that is refused by clippy, through `allow_attributes` and
`allow_attributes_without_reason`. On the client it is refused by a script,
`client/suppression-scan`, because the linter this workspace can install has no
rule for it and honours a blanket suppression silently. The comment at the top
of that script carries the command showing why the linter is the one it is, and
`./prove-quality` has a leg that demonstrates the silence rather than asserting
it.

Where the server's modules may depend on each other is a rule too, and it has
two commands of its own:

    ./layout-scan
    ./prove-layout

`docs/layout.md` states a stack and says the arrows all point one way. Cargo
refuses a cycle, so the arrows cannot point both ways at once, and it refuses
nothing about a single arrow pointing the wrong way. `./layout-scan` reads the
manifests under `server/crates/` and refuses an edge that points up a level,
one that points sideways between two modules at the same level, one into the
binary that composes the server, one into the connector module, one out of the
money module, one to a crate that is not there, and a crate the note does not
place at all. It reads the manifests rather than a built graph, so it answers
before anything is compiled and it can be pointed at a directory of manifests
given as its argument.

That last refusal is the one worth knowing about before you add a module. A new
crate under `server/crates/` is refused until `layout-scan` says where it sits,
which is deliberate: a module whose level nobody chose is a module whose edges
nothing can judge.

`./prove-layout` is the evidence. Every leg builds a fixture graph, runs the
shipped scanner against it, and reads both the exit code and the reason, so a
leg named for one refusal cannot be satisfied by a different one. Each refusal
leg is followed by the same graph with that one edge removed, which has to pass.

The suites are two commands, and the second one is named for what it needs:

    ./test
    ./test-needs-a-database
    ./coverage

`./test` is the unit suite. Every test in it runs with no display server, no
administrative rights and no reachable network. A test that needs a real
database is not one of these and belongs to the second command, which takes
`KONTOR_TEST_DATABASE_URL` and refuses rather than guessing:

    KONTOR_TEST_DATABASE_URL=postgres://kontor:kontor@127.0.0.1:5432/kontor \
      ./test-needs-a-database

A test in the default suite may need nothing outside the machine, and there are
four things it can need instead: a display server, an outbound network, a
simulator or a database. A test that needs one is marked with the feature named
for it, and `./test-scan` refuses an unmarked test that reaches for one before
anything is compiled.

The separation is made by cargo rather than by anybody's memory. A test target
that needs something declares `required-features` in its crate's manifest, and
cargo does not build a target whose required features are off, so the default
run leaves it out with no argument to remember. What is marked is printed rather
than listed here, because a list in a document drifts against the manifests:

    ./test --marked

That prints each marked target, what it needs, and why it is marked, and it
refuses a marked target that says nothing about the why. The reason goes on the
line directly above `required-features` in the crate manifest, as `reason:`
after the comment mark, which is the same rule as for a lint suppression and for
an escape from the client type system and it is there for the same reason:
moving the marking moves its justification with it. An empty reason is refused
as an absent one, and a comment above the marking that is not a reason is
refused too, because a target kept out of the default suite with no reason
recorded is one nobody reading the register can judge.

It also prints how many test targets run with nothing outside the machine
reached, as a count and as a proportion, so a later decline shows up as a
number rather than as a feeling. Two bounds are printed with it rather than
left here. It is counted in targets and not in tests, because cargo says how
many targets exist without building one and says nothing about how many tests
are inside them. And it is the server workspace, which is the only one with a
suite. Nothing refuses a decline in that number: over a handful of targets a
floor moves by a whole target at a time, so it would refuse the next marked
target rather than a trend, and #117 is where the audit of that number belongs.

`./test` runs the scan itself before it compiles anything, so the refusal
arrives with the cause rather than as a connection error in the middle of a run,
which reads like a flake and passes on the next machine.

    ./test-scan
    ./prove-headless

The second is the evidence that the first bites, in three groups because the
scan carries three rules over one register of reaches. On a test target: an
unmarked test that opens an outbound socket is refused, the same test marked for
an outbound network is accepted, a marking naming something outside the four is
refused, and a test that reaches for nothing passes. On a test inside a crate's
`src/`, where no marking is possible: one that opens a socket is refused, the
same one with the reach taken out passes, a display reached in library code
beside a clean test module passes, and a test module declared as a file rather
than written inline is followed into that file and refused there.

The third group is not about a test at all, and the paragraph below the next one
is where it is described. Library code that opens an outbound socket is refused,
the same crate with the reach taken out passes, the same reach in a crate named
as a connector passes, the same crate renamed off that prefix is refused again,
and a module naming every one of those shapes in a doc comment and reaching for
none of them passes.

A fourth pair is about the harness rather than the rule. Every leg runs the
shipped scan as a path, the way `./test` and the workflow run it, and the pair
shows why: the same file with its executable bit off is refused when run that
way and accepted when passed to `sh`, so a proof written the second way is green
on a tree whose scripts the workflow cannot execute. This project has lost jobs
to that twice. The pair needs a filesystem that can take the bit off a file,
which Windows cannot, and there it prints that it was skipped and that nothing
in the run covers the mode.

Those two judge the source. What judges the environment is a third command and
the evidence that it is doing anything:

    ./test-sealed
    ./prove-sealed

`./test-sealed` runs the same suite inside a network namespace of its own, where
the only interface is a loopback that is down, there is no route out of it, and
the user cannot become root while the seal holds. Root builds the box and the
suite runs inside it with strictly less than the run that started it: the user
and group are dropped back to the caller's, the supplementary groups are
emptied, and `no_new_privs` is set so the setuid binary that root was reached
through refuses to elevate. All of that is read back out of `/proc` before the
suite starts, and the script refuses rather than reporting on a seal it did not
get, so a green run is a statement about the environment and not only about the
tests.

It compiles outside the seal and runs inside it with `--offline`, because the
claim is that the suite reaches for nothing rather than that a cold clone builds
without a registry. It is Linux only, it needs a password-free route to root to
build the namespace, and it says so and stops rather than passing where either
is missing.

A display server is the one of the four a network namespace leaves alone, since
one reached through a path on the filesystem is still there inside. `./test-sealed`
refuses to start when `DISPLAY` or `WAYLAND_DISPLAY` names one, rather than
letting a green run be read as a suite that had none.

`./prove-sealed` is the evidence, and every leg of it is a pair: one command run
on each side of the seal, with both exit codes read. A seal that quietly stopped
sealing reddens the inside half of every pair while the outside half goes on
passing, which is the case a single-sided check cannot see. The last pair is a
cargo test target that opens a socket, which passes outside the seal and is red
inside it.

A unit test inside a crate's `src/`, in a `#[cfg(test)]` module, is part of the
library target and cannot carry `required-features` at all. This passage said no
command checked one and that a reader was the whole mechanism, which was true
until `./test-scan` grew a second half for them. There is no marking to check
such a test against, so the rule is the stricter one that it may not reach for
any of the four, and the message it earns names the move to a target under
`tests/` rather than a feature to add. A module written as `#[cfg(test)] mod
name;` is followed into the file it names, and so is anything that file declares
in turn.

One bound on that half, stated because it is not visible from a green run: the
register of reaches is a floor rather than a guarantee, since it holds the
shapes somebody could plausibly write today and not one nobody has written yet.
That bound is the same in every half below.

How much that half found to judge is printed rather than written here, in the
second clause of the scan's own output, because a count in a document drifts
against the tree the moment somebody writes the first such test:

    ./test-scan

While that clause says the half judged nothing, the only evidence it refuses
anything is `./prove-headless` and its fixtures. That is the order the gap was
closed in deliberately: before the first in-crate test arrives rather than after
somebody has written one against no gate.

The same file carries one rule that is not about a test. Ordinary library code
under `src/` may not open an outbound connection unless the crate holding it is
a connector, which is the boundary
[`docs/decisions/0011-connectors.md`](docs/decisions/0011-connectors.md) states
in terms a review can refuse a proposal with, and which #113 asked for a pattern
over. This passage said no command judged such code and named that issue as
where the pattern belonged, and it now judges it.

Only the outbound row of the register is read there. The other three are not
boundaries: the store module is meant to speak to a database, and a crate that
draws is a question about a client rather than about this stack. What makes a
crate a connector is its name, which is the answer `connector-guide-scan` and
the connector module already give, so the exempt set is decided by something a
crate carries before it has declared anything. The workspace holds none today,
which the third clause of the same output says, and the evidence that any of it
refuses anything is `./prove-headless` again.

That rule reads the source with its comments and string literals blanked out,
and the two halves above read the bytes as written. The difference is deliberate
and it is a bound in both directions. A test is short and a socket address
written as a string is exactly the case worth catching there. Library code in
this tree is mostly prose, and a module documenting the boundary names every
shape a reach is written with, so a rule reading the comments would refuse the
file that explains the rule; a reach hidden in a string literal in library code
is what that costs.

All three are the server's. The client half of the rule is two commands of its
own, and the second is the evidence:

    client/test-scan
    client/prove-test-scan

The first carries two rules over the client workspace. A tracked file named as a
test that the runner is not given is refused, because the runner is given one
pattern and a test outside it never runs, and a test that never runs passes
forever while being counted. And a test that reaches an address or drives a real
browser is refused, because this suite runs inside the test process. It reads
the bytes as written, comments and string literals included, which is the trade
`./test-scan` makes over a test target and it is made here for the same reason.

What the client suite is, in full: the runner node ships, given the pattern in
the `test` script of `client/package.json`. No dependency and no compiler, since
node strips the types itself, and nothing was added to the workspace but that
line. What that buys is a suite with no apparatus to maintain; what it costs is
that JSX is not stripped, so a component test cannot be written as `.tsx` until
somebody argues for a means, and the first rule refuses one rather than letting
it sit unrun.

Three things it is not, said plainly because a green run says nothing about any
of them. There is no rendering environment, which a component test needs and
which is a dependency nobody has argued for yet. There is no command for a test
that needs a real browser, which is why such a test is refused rather than sent
somewhere. And the workspace holds no test at all today, which is what the
scan's own count says: the runner exits green when it is given nothing, so a
client leg reporting only an exit code would say the same thing on a workspace
with tests and on one without. #63 holds all three.

`./prove-test-scan` under `client/` is what says any of it refuses anything,
against throwaway repositories it builds rather than against this tree. Its last
group is not about the scan: it runs the shipped `test` script against a fixture
the scan has already judged, which is what binds the two statements of the
pattern, and it carries the runner's own bound by proving that a run given
nothing reports zero and passes.

`./coverage` runs the same suite under instrumentation and refuses below a
floor. THE FLOOR IS A MEASUREMENT AND NOT A TARGET. It is 80% of lines and 75%
of regions, which is what the tree reached, truncated to a whole percent:

    cargo llvm-cov --manifest-path server/Cargo.toml --workspace \
      --exclude kontor-lint-fixture --locked --summary-only

whose TOTAL row reads 287 regions at 75.96% and 233 lines at 80.69%. That row is
described rather than pasted here because it is wider than this page.

Truncating rather than taking the number itself is the smallest slack that stops
a refactor moving one line from reddening the gate. Raising the floor is a
change to `./coverage`, argued in the issue that raises it; lowering it is the
same change and the one a reviewer should be slowest to agree with.

That command needs `cargo-llvm-cov`, which is the one tool here that does not
ship with the toolchain, and `./coverage` refuses with the install line rather
than working around it. The `llvm-tools` component it reads is named in
`rust-toolchain.toml`, so rustup installs that part inside this tree.

Both installs above read a lock file and refuse where it does not match the
manifest beside it, which is the whole of what stops a dependency being resolved
fresh on the machine that builds a release. `cargo build --locked` carries it on
the server and `npm ci` carries it on the client, and one more command is the
evidence that both still refuse:

    ./prove-lock-files

Every leg builds a fixture package and runs the shipped flag against it. A
dependency added to a manifest alone is refused, an absent lock file is refused
rather than written, the same drifted fixture passes with the flag off, and the
corrected neighbour passes, so a leg cannot be satisfied by a toolchain that
refused for another reason or by one that refuses everything. Each fixture
depends on a directory beside it rather than on a registry, so no leg reaches a
network.

What that command does not read is `build`. The flags are written there, the
legs run them against fixtures of their own, and nothing joins the two, so a
`build` that dropped `--locked` or reached for `npm install` would leave every
leg green. #114 is where the rest of the supply chain belongs, and it names what
is not covered yet.

The comments in `.github/workflows/` are where every gate is argued, and one
command refuses a comment that argues from a file this repository does not hold:

    ./workflow-scan
    ./prove-workflow-scan

The first reads the part of each line after its first `#`, takes every token
ending in an extension it knows, and refuses one that resolves to no tracked
file. A bare `something.yml` resolves under `.github/workflows/` as well as
under the root, because that is how these files refer to each other. The second
is the evidence, against repositories it builds rather than against this tree.

What it does not judge is a path on a `run:` line, which is executed and fails
the run rather than misleading a reader, and an issue number, which would have
to be resolved against the tracker rather than against the checkout. #156 is
where both bounds are recorded.

The same directory is read a second time, for what its steps run rather than for
what its comments claim:

    ./action-pin-scan
    ./prove-action-pins

The first refuses a step naming an action by anything other than a full commit
hash. A tag and a branch are names their owner can repoint at any time, and the
action runs inside this repository's job, with the token that job holds and the
checkout it has already made, so repointing one is a route into every gate here.
A reference beginning with `./` names an action inside this checkout and carries
nothing anybody outside can move, so it is accepted, and the tree holds none
today. The second is the evidence, against repositories it builds rather than
against this tree, and every leg expecting a refusal is followed by its
one-change neighbour.

Three things it does not reach, said plainly because a green run says nothing
about any of them. Whether a hash is the commit the version comment beside it
claims is not judged, since resolving that reaches the network. Whether the
comment is there at all is not judged either, which is a rule about how a pin is
written rather than about whether it is one. And a hash is immutable rather than
trustworthy: it says the bytes will not change under the job, not that the bytes
were ever good, which is what the dependency review gate is for and what #114
carries the rest of.

Two version floors are decided in a record and answered in files elsewhere, and
one command refuses an answer that has left its floor behind:

    ./floor-scan
    ./prove-floors

`docs/decisions/0001-means.md` states a floor for PostgreSQL and a floor for
Node, and closes that passage by saying that raising one is a change to that
record rather than one made quietly in a workflow file. The first command is
what makes the sentence refusable. It reads both floors out of the record at
every run and carries no version number of its own, so raising one there is red
until each pin follows it, and lowering one where a pin lives is not available
without editing the record that argues it. A pin is not a floor: a pin says what
this repository builds and tests against today, and a floor says what an
operator is told they may run, and when the two disagree the first person to
find out is the operator on the older version.

It fails closed in both directions. A floor the record does not state is
refused, because a check with no authority to read must not pass as one that
read an authority and found nothing. A floor the record states and no tracked
file answers is refused too, since a floor nothing is pinned to is a floor
nothing follows.

The second is the evidence, against repositories it builds rather than against
this tree. Every leg that expects a refusal is followed by its one-change
neighbour, and the numbers in every fixture are fixture numbers, because a leg
written against the real record would prove what that record said on the day it
ran. The near miss worth the most is a pin whose number merely starts with the
floor, which is what says the comparison is on the whole number rather than on a
prefix of it.

Three things it does not reach, said plainly because a green run says nothing
about any of them. The Rust floor is stated in the same record and answered in
two more places, and it is not read: #131 names PostgreSQL and Node, and whether
the third belongs beside them is open there rather than settled by a script.
What the record states for Node is a line rather than a minimum, so what is
judged is the major, and two pins on that line are indistinguishable here. And
nothing runs anything against a floor version, which is the rest of #131: the
pins agreeing with the record says the numbers agree, not that either suite was
ever exercised against the oldest version an operator is told they may run.

The lint gate refuses shapes that are wrong in one file. A scanner that follows
a value from where it enters the program to where it is used answers a different
question, and code scanning is where that one is asked. It runs in the workflow
rather than here, because the analysis needs a database built over the whole
source and that is minutes rather than seconds, but the part of it that refuses
is a command like every other one:

    ./finding-scan.js sarif/rust.sarif
    ./prove-finding-scan

One job per language, three of them, and each names its language in the
configuration rather than leaving the scanner to work out what the repository is
written in. The third is the workflow files, which are code holding a token, and
they are read as source here rather than left to the audit that reads them for
another question. The query set is stated in `.github/codeql/config.yml`,
nothing is excluded from it, and the file says where an exclusion would go and
what it would have to carry.

An analysis uploads what it found and exits successfully whether it found
anything or not, so on its own it reports rather than refuses.
`./finding-scan.js` is the refusal. It reads the report the analysis wrote and fails the run on a
result whose rule carries a security severity at or above the line that file
states, which is 4.0, the bottom of the band the surface calls medium. A rule
carrying no such number, which is what a maintainability query carries, is
uploaded and counted and does not fail anything. Moving the line is a change to
that file, argued in the issue that moves it.

It fails closed on five shapes rather than reading any of them as a clean tree:
a report that is not there, bytes that are not JSON, a report holding no run at
all, a result naming a rule the report does not describe, and a severity that is
not a number. The third is the one worth knowing about, since an analysis that
read nothing writes exactly that.

`./prove-finding-scan` is the evidence, against reports it writes rather than
against anything a scanner produced here, every leg expecting a refusal followed
by its one-change neighbour. The near miss worth the most is a critical finding,
whose severity sorts before the line as text and after it as a number, so a
check written the easy way would pass exactly the findings it exists for.

Three things it does not reach, said plainly because a green run says nothing
about any of them. Whether the scanner found everything is not decided by any of
it: a report with no results is a statement about the queries that ran rather
than about the tree, and no fixture in this repository proves that a given query
still catches what it is named for. The server extraction is partial, because
the extractor reads the source without building it, which is the only mode that
language offers, so a file whose dependencies or macros it cannot resolve is
analysed with less than it needed; each analysis prints how many of its files
that was, and nothing refuses a rise in that number. And there is no mobile
toolchain here, so none of it is scanned; #75 is where that core arrives and is
the issue holding the gap.

The documentation is read by four rules of its own, two about what a document
points at, one about how a decision record is shaped and one about the words a
document is written in:

    ./docs-scan
    ./docs-scan links
    ./docs-scan documents
    ./docs-scan records
    ./docs-scan terminology
    ./prove-docs-scan

The first takes the target of every markdown link in a tracked document and
refuses one that resolves to no tracked file, because a link an author wrote is
a promise that it leads somewhere. A target naming a scheme is left alone, since
resolving one reaches the network and every rule here answers from the checkout.

The second reads the prose of every tracked document and refuses a token ending
in `.md` that carries a directory and names no tracked file. Three shapes are
not that, and each says so in its own way. A token carrying a placeholder, in
angle brackets or written in upper case where the naming rule asks for lower,
stands for a name rather than being one. A token followed by `under #<n>` names
a document an issue owes, which is the numbering rule working rather than a dead
reference; the issue is not resolved, because that reaches the tracker. And a
token inside an indented block or a fence belongs to a command, which fails for
the reader who runs it rather than misleading the reader who does not, which is
the bound `workflow-scan` states over `run:` lines for the same reason.

Only a token ending in `.md` is judged, and the narrowing is deliberate. The
tokens in these documents that carry a slash and are not paths in this tree
include a repository name, a check name with spaces in it, an absolute path on
the machine and an argument to a command that reaches a service, so a rule
guessing at which of those was meant to resolve would refuse the sentence
explaining it. A document is the case worth holding, because a document is
followed rather than executed.

The third refuses a decision record that is not named `NNNN-slug.md`, one whose
title number disagrees with the number it is filed under, two records written
under one number, and a header block whose fields are missing, empty, out of
order, or not the ones a record carries. Which fields those are is read out of
`docs/decisions/0001-means.md`, the record that states the rule, rather than
written into the check, so a field added there is required from that commit.
That record is judged against itself and passes by construction, and it is the
one file here whose header block nothing checks.

The fourth reads the words a document is written in against `documentation-words`,
which is the register saying which spelling this project uses and carrying a
reason for every entry. An entry is the spelling that is refused, then `->`,
then the spelling to write instead, and the line immediately above it is why.
The check reads that file rather than holding a copy, so a word joins the rule
by being written down there, and it refuses an entry that says nothing about
itself, an entry whose answer it would itself refuse, and an entry answering
with a word no tracked document uses. That last one is the register failing
closed in the other direction: it holds the vocabulary the documentation already
establishes rather than a list of words somebody would like.

Three things are outside it. A word inside backticks, in an indented block or in
a fence is somebody else's and is passed over, which is how a check name on
another repository and a field in another project's metadata stay readable.
General spelling is not judged at all: deciding whether an unknown word is
misspelled needs a dictionary, which is a dependency this tree does not carry
and could not reach from a suite that reaches nothing, so what is judged is the
words the register names and nothing else. And one word this project does spell
its own way is deliberately not in the register, with the reason written where a
reader who goes to add it will meet it first.

`./prove-docs-scan` is the evidence, against repositories it builds rather than
against this tree. Every leg that expects a refusal is followed by its
one-change neighbour, and one leg renames a field in the fixture exemplar and
requires the rename to propagate, which is what says the check holds no copy of
the header block rather than a well-timed one. The terminology legs carry the
same pair: one of them changes the word an entry answers with and requires the
message to follow it. The vocabulary in those legs is a fixture vocabulary and
none of its words is one this project uses, because a leg written against the
real register would prove what the register held on the day it ran.

What none of it judges is whether a document is true. That a heading is present
is a fact about the bytes; that what is written under it still describes what
this tree does is not, and no reading of the tree decides it. A reader is the
whole mechanism for that half.

What #115 asks for and is not here, said plainly because a green run says
nothing about any of it. A fragment on a link target is not judged, since the
spelling of an anchor belongs to the renderer and nothing tracked here writes
one. Nothing checks that a record carries its alternative and the condition that
would reverse the decision: three of the records in this tree carry neither
under any heading a pattern could find, so that rule as stated would refuse the
tree for what it already is. The word list judges the words in it and not the
spelling of every other one, which is the half a dictionary would be needed for.
And no check says whether a command shown in a document is executed anywhere or
marked as illustrative, which needs a suite that can run one.

One document is read a fifth way, because it is the one that gets copied from
rather than read. `docs/sync-protocol.md` is what a client is written against,
including a client somebody outside this repository writes, and every message
shape in it carries a table of its fields and one example:

    ./sync-example-scan.js docs/sync-protocol.md
    ./prove-sync-examples

The first refuses an example carrying a field its table does not declare, one
omitting a field its table requires, one whose `type` is not the shape it sits
under, a shape with no example or with two, an example that is not JSON, a table
row that does not declare four cells or answers requiredness with a sentence, and
a requirement list with a hole in it. It fails closed on a document that is not
there, one holding neither shape heading, one holding a heading and no shape, one
with no requirement list, and a fenced example that is opened and never closed,
because a scanner reading an empty document as a clean one turns a deleted body
into a green tick, and everything below an unclosed fence is read as part of the
example above it.

Only the top level keys of an example are judged. A field whose value has a shape
of its own is judged where that shape has a section, which is why the shapes that
travel inside a message have sections rather than being described in the prose of
the messages carrying them.

What it cannot judge is whether a table describes what an instance would really
send. Nothing in this tree speaks the protocol yet, so a green run says the
document agrees with itself and nothing more; #74 is where a client is driven
through the whole of it against a stubbed server. `./prove-sync-examples` is the
evidence that each refusal above bites, against documents it writes rather than
against this project's own, and every leg expecting a refusal is followed by the
one-change neighbour that has to pass.

The tree it builds:

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
    action-pin-scan
    build
    client
    connector-guide-scan
    coverage
    docs-scan
    docs
    documentation-words
    finding-scan.js
    floor-scan
    format
    layout-scan
    licence-apply
    licence-scan
    licences-allowed
    lint
    pr-body-scan
    prove-action-pins
    prove-connector-guides
    prove-determinism
    prove-docs-scan
    prove-finding-scan
    prove-floors
    prove-headless
    prove-layout
    prove-licences
    prove-lock-files
    prove-pr-body
    prove-quality
    prove-sealed
    prove-secret-scan
    prove-sync-examples
    prove-workflow-scan
    regenerate
    rust-toolchain.toml
    secret-scan
    secrets-allowed
    server
    sync-example-scan.js
    test
    test-needs-a-database
    test-scan
    test-sealed
    text-scan
    workflow-scan

## What runs on a pull request

This guide does not list the checks, because a list in a document drifts against
the thing it describes and the drift is invisible until somebody trusts the
list. Two commands print them instead.

The workflows that exist:

    gh workflow list --repo iderex/kontor

The names a particular commit was actually judged under, which is the set a
required check would be matched against:

    gh api "repos/iderex/kontor/commits/$(git rev-parse HEAD)/check-runs" \
      --jq '.check_runs[].name'

And the names written into the workflow files, which is the set the two commands
above are read against:

    git grep -h '^    name: ' -- .github/workflows/ | sed 's/^    name: //' | sort

Every job here carries a name written out in the file rather than derived from a
job id or a matrix value, because a required check is matched by that string and
a rename silently stops requiring it.

The three answers are not the same set and are not meant to be. A workflow that
triggers only on a pull request contributes no check run to a commit reached any
other way, so the second command prints fewer names than the first on a commit
that arrived on the default branch. Read the second one against your own head
commit once the pull request exists.

Every gate here is judged once per name on a given commit. A gate triggers on
the pull request event and, where it also watches the default branch, on a push
to that branch alone, so a commit that is both pushed to a working branch and in
a pull request is not judged twice under one name. `docs/required-checks.md` is
where that matters, because a name carried by two runs of two events cannot be
required of a merge without somebody first reading how the platform resolves the
duplicate.

## No work without an issue

Every change starts as an issue and lands as a pull request. Pushing to the
default branch is refused, including for me, and the refusal is a property of
the repository rather than a habit:

    gh api repos/iderex/kontor/rulesets --jq '.[] | select(.name == "gate") | .id'
    20486686
    gh api repos/iderex/kontor/rulesets/20486686 \
      --jq '{enforcement, bypass: .bypass_actors, required: [.rules[].type]}'
    {"bypass":[],"enforcement":"active","required":["deletion","non_fast_forward","pull_request","required_signatures"]}

That output ended at `pull_request` until this commit, and the live ruleset
returns a fourth entry. It was found by re-running the command rather than
reading the line pasted beneath it, which is this document's own rule about
evidence applied to this document. Nothing here would have caught it:
`docs-scan` judges that a reference resolves and that the words are the ones
`documentation-words` names, and no check re-executes a command a document
quotes to compare what it printed. What the fourth entry asks of a contributor
is under `## Sign off your work` below, and what the ruleset requires is
recorded in `docs/required-checks.md`.

An issue says three things.

What is wrong. Not what to build, but what is broken, missing or undecided, so
that a reader can disagree with the problem before arguing about the fix.

What the evidence is. Where the evidence is a number, the issue carries the
command that produced it, run against the reference the reader will have rather
than against a working tree. A number without its command cannot be checked and
cannot be seen to have moved.

What done means, in terms somebody other than the author can verify. An issue
whose done condition is a matter of taste has not stated one.

## Sign off your work

Every commit carries a `Signed-off-by:` trailer, and the name and address in it
match the commit author exactly. The trailer is an assertion of the Developer
Certificate of Origin, version 1.1, whose text is in [DCO](DCO). Read it once
before you sign anything, because signing is what it means rather than a
formality the tooling wants.

As you commit:

    git commit -s

On work already committed, where `<base>` is the commit your branch left the
default branch at:

    git rebase --signoff <base>

The check runs on every pull request and fails closed. One commit anywhere in
the branch without a trailer matching its author reds it, and the failure names
the commit and the exact line it wanted. Confirm it ran against your own head:

    gh api "repos/iderex/kontor/commits/$(git rev-parse HEAD)/check-runs" \
      --jq '.check_runs[] | select(.name | test("DCO")) | "\(.name) \(.conclusion)"'

The trailer is not a signature, and the ruleset asks for both. A rule on the
default branch requires a verified signature, and it has no bypass actors, so it
is required of everybody including me:

    gh api repos/iderex/kontor/rulesets/20486686 \
      --jq '{bypass: .bypass_actors, required: [.rules[].type]}'
    {"bypass":[],"required":["deletion","non_fast_forward","pull_request","required_signatures"]}

    gh api "repos/iderex/kontor/commits?per_page=6" \
      --jq '.[] | "\(.sha[0:9])\t\(.commit.verification.verified)\t\(.commit.verification.reason)"'
    0ac6a7fbc	true	valid
    99e7445ae	true	valid
    d580df6ea	true	valid
    21be86ddf	true	valid
    5b3d5ccaf	true	valid
    c9d3295da	true	valid

The two mechanisms answer different questions and neither stands in for the
other: a signature says a key held the commit, and the trailer says the author
asserts the right to contribute it. Exactly which commits the platform walks
when it applies that rule is the platform's answer rather than this tree's, and
it is not restated here; what is checkable from here is that six of six commits
on the default branch carry a valid signature and that the rule is active with
nobody exempt.

`docs/required-checks.md` said this rule was not requested, for the reason that
the trailer had been chosen instead. The rule arrived, that document now records
what was read back, and no decision record in this tree argues the setting, which
is what #187 leaves open rather than settles.

A signing failure is a stop rather than an obstacle to route around. The way past
it is one word, in either spelling:

    git commit --no-gpg-sign
    git -c commit.gpgsign=false commit

Neither is refused before the merge. Nothing in this tree reads a signature, so a
bypassed commit builds, tests and reviews exactly like a good one, and the only
thing that says otherwise is the merge at the end of the line. Fix the signing
instead.

## Every test runs headless and unelevated

The default test suite runs with no display server, no administrative rights and
no reachable network, and a test that needs a display, a live network, a
simulator or a database is marked for which of those it needs and is excluded
from the default run by configuration rather than by a flag anybody has to
remember.

What is marked, what each marked target needs, why it is marked, and how much of
the suite is left running with nothing outside the machine reached:

    ./test --marked

Several things carry that rule now, and they carry different halves of it.
`./test-scan` refuses an unmarked test target whose source reaches for one of
the four, before anything is compiled, and refuses a test inside a crate's
`src/` that reaches for one at all, since that one can carry no marking.
`./test --marked` refuses a marked target that does not say why it is marked, so
the register a reader checks the marking against has no entry they cannot check.
`client/test-scan` carries the same rule on the other layer, where there is no
marking to carry an exception and so no reach is allowed at all.
`./test-sealed` runs the suite where none of the three the environment can
supply is there to reach, and since `./test` runs both layers, the client half
runs inside that seal too. `./prove-headless`, `./prove-sealed` and
`client/prove-test-scan` are what say each of those still refuses what it names.
The section above describes them and is where the detail is; this is the rule in
one sentence and the command that prints it.

This passage said the opposite until this commit: that nothing enforced the rule
and no command listed the marked tests, because neither the suite nor the
marking existed. Both existed by then. It was found by reading this document
against the commands the tree actually holds, which is the reading the
enumeration rule elsewhere in it asks for, and the correction is here rather
than only in the section that already knew.

## Commit messages

State what changed and what failure it prevents. Where the change is a
correction, say what was wrong and how it was found, because the second is what
stops the same defect arriving again by a different route.

One topic per commit and one per pull request. A commit carrying two unrelated
changes gets a message describing one of them, and the other one lands
undescribed.

## English, and no attribution to a tool

Everything tracked in this repository is written in English. Nothing tracked
carries a tool name, a generated-by marker or an attribution for how the work
was produced. What a change says about itself is what it changed and why.
