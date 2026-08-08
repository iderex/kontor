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

The second is the evidence that the first bites, in two groups because the scan
carries two rules over one register of reaches. On a test target: an unmarked
test that opens an outbound socket is refused, the same test marked for an
outbound network is accepted, a marking naming something outside the four is
refused, and a test that reaches for nothing passes. On a test inside a crate's
`src/`, where no marking is possible: one that opens a socket is refused, the
same one with the reach taken out passes, the same reach in library code beside
a clean test module passes, and a test module declared as a file rather than
written inline is followed into that file and refused there.

A third pair is about the harness rather than the rule. Every leg runs the
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

Two bounds on that half, stated because neither is visible from a green run.
Ordinary library code under `src/` is not judged by either half: a crate that
opens a socket outside a test earns nothing here, and the pattern over the tree
that would refuse one is #113's. And the register of reaches is a floor rather
than a guarantee, in both halves, since it holds the shapes somebody could
plausibly write today and not one nobody has written yet.

How much that half found to judge is printed rather than written here, in the
second clause of the scan's own output, because a count in a document drifts
against the tree the moment somebody writes the first such test:

    ./test-scan

While that clause says the half judged nothing, the only evidence it refuses
anything is `./prove-headless` and its fixtures. That is the order the gap was
closed in deliberately: before the first in-crate test arrives rather than after
somebody has written one against no gate.

Both are the server's. The client workspace has no tests and no runner, and #63
is where its half belongs; a client leg that ran nothing and reported green
would be worse than this sentence.

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

The documentation is read by three rules of its own, two about what a document
points at and one about how a decision record is shaped:

    ./docs-scan
    ./docs-scan links
    ./docs-scan documents
    ./docs-scan records
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

`./prove-docs-scan` is the evidence, against repositories it builds rather than
against this tree. Every leg that expects a refusal is followed by its
one-change neighbour, and one leg renames a field in the fixture exemplar and
requires the rename to propagate, which is what says the check holds no copy of
the header block rather than a well-timed one.

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
tree for what it already is. There is no word list. And no check says whether a
command shown in a document is executed anywhere or marked as illustrative,
which needs a suite that can run one.

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
    build
    client
    connector-guide-scan
    coverage
    docs-scan
    docs
    format
    licence-apply
    licence-scan
    licences-allowed
    lint
    pr-body-scan
    prove-connector-guides
    prove-determinism
    prove-docs-scan
    prove-headless
    prove-licences
    prove-lock-files
    prove-pr-body
    prove-quality
    prove-sealed
    prove-workflow-scan
    regenerate
    rust-toolchain.toml
    server
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
default branch is refused, including for the maintainer, and the refusal is a
property of the repository rather than a habit:

    gh api repos/iderex/kontor/rulesets --jq '.[] | select(.name == "gate") | .id'
    20486686
    gh api repos/iderex/kontor/rulesets/20486686 \
      --jq '{enforcement, bypass: .bypass_actors, required: [.rules[].type]}'
    {"bypass":[],"enforcement":"active","required":["deletion","non_fast_forward","pull_request"]}

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

## Every test runs headless and unelevated

The default test suite runs with no display server, no administrative rights and
no reachable network, and a test that needs a display, a live network, a
simulator or a database is marked for which of those it needs and is excluded
from the default run by configuration rather than by a flag anybody has to
remember.

What is marked, what each marked target needs, why it is marked, and how much of
the suite is left running with nothing outside the machine reached:

    ./test --marked

Four things carry that rule now, and they carry different halves of it.
`./test-scan` refuses an unmarked test target whose source reaches for one of
the four, before anything is compiled, and refuses a test inside a crate's
`src/` that reaches for one at all, since that one can carry no marking.
`./test --marked` refuses a marked target that does not say why it is marked, so
the register a reader checks the marking against has no entry they cannot check.
`./test-sealed` runs the suite where none of the three the environment can
supply is there to reach. `./prove-headless` and
`./prove-sealed` are what say each of those still refuses what it names. The
section above describes all five and is where the detail is; this is the rule in
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
