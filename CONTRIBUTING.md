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

Adding a binary file means declaring it in `.gitattributes` with git's `binary`
macro. Nothing is exempt from the encoding rule by detection, which is
deliberate: a detector reading a mangled file as binary would exempt exactly the
file the rule exists for.

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

The separation is made by cargo rather than by anybody's memory. A test target
that needs something declares `required-features` in its crate's manifest, and
cargo does not build a target whose required features are off, so the default
run leaves it out with no argument to remember. #7 widens that marking to the
four things a test here can need, and makes an unmarked test that reaches for
one of them fail at collection rather than at an assertion.

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
    coverage
    docs
    format
    lint
    prove-determinism
    prove-quality
    regenerate
    rust-toolchain.toml
    server
    test
    test-needs-a-database
    text-scan

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

The two answers are not the same set and are not meant to be. A workflow that
triggers only on a pull request contributes no check run to a commit reached any
other way, so the second command prints fewer names than the first on a commit
that arrived on the default branch. Read the second one against your own head
commit once the pull request exists.

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

Nothing enforces that today and no command lists the marked tests, because
neither the suite nor the marking exists. #7 defines the marking and owes the
command that prints it, and #5 builds the suite it would print from. The rule is
stated here so that the first test written is written under it, not so that a
reader can check it.

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
