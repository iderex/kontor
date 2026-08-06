# Contributing

This guide is written to be run rather than admired. Every rule below names the
command that shows whether the rule is met, and where no such command exists yet
the guide says so instead of implying one.

## Before you push

There is no single command in this clone that runs the gates. That is a gap and
not an omission: the tree carries no application toolchain yet, so there is
nothing local for a formatter, a lint gate or a test runner to be pointed at.

    git ls-tree --name-only HEAD
    .gitattributes
    .github
    CONTRIBUTING.md
    DCO
    LICENSE
    NOTICE.md
    README.md
    SECURITY.md
    docs

Until that changes, the pull request is the first place any verdict appears, and
the gates that exist there run on a server rather than on your machine. #2 adds
the workspaces and pins both toolchains, #3 adds the formatting and lint gate,
#5 adds the test harness and the coverage floor, and #6 puts them on the pull
request under stable names. When those land, the command belongs at the top of
this section and this paragraph comes out.

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
