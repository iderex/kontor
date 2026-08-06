# 0001. The means: language, runtime and toolchain

Status: accepted
Issue: #1
Supersedes: nothing
Superseded by: nothing

## The decision

Three layers are in play and they do not share a language.

The server, meaning the record store, the API, the reporting engine and the
workflow engine, is written in Rust. Edition 2024. The floor is the 1.97 stable
series, and the exact version is pinned by a file in the tree rather than by this
document; #2 adds that file and this record does not restate the number it will
carry. The toolchain is the one that ships with the release: `cargo` for build
and test, `rustfmt` for formatting, `clippy` for lint. No third party build
system sits in front of it.

Storage is PostgreSQL, with a floor of major version 17.

The web client is TypeScript running in a browser, built and tested under Node.
The floor is the 24 line. The package manager is npm, and its lock file is
committed.

The mobile clients are not decided here. What binds them is the sync protocol,
not the toolkit, and their means is recorded separately in
`docs/decisions/0010-mobile.md` under #71.

There is no second server language. A shell script, a Python invocation or a
generator that produces Rust or TypeScript is not a second language for this
purpose, but each one is a dependency and is argued where it is added.

## The versions, read rather than remembered

The Rust floor tracks the current stable series:

    gh api repos/rust-lang/rust/releases --jq '.[0:2][] | "\(.tag_name) \(.published_at)"'
    1.97.1 2026-07-16T12:29:15Z
    1.97.0 2026-07-09T12:25:16Z

The edition is the one the toolchain itself produces for a new package, so it is
not a preference expressed against the tool:

    cargo new --bin probe --vcs none && grep edition probe/Cargo.toml
    edition = "2024"

The Node floor is the current long term support line, and the line has a stated
end date the operator and the contributor can both read:

    gh api repos/nodejs/Release/contents/schedule.json --jq .content | base64 -d \
      | python -c "import json,sys; d=json.load(sys.stdin); print('v24', d['v24'])"
    v24 {'start': '2025-05-06', 'lts': '2025-10-28', 'maintenance': '2026-10-20', 'end': '2028-04-30', 'codename': 'Krypton'}

The PostgreSQL floor is one major behind the newest released one. The mirror's
tags show which majors are cut:

    gh api "repos/postgres/postgres/tags?per_page=100" \
      --jq '[.[].name | select(startswith("REL_"))] | .[0:12] | join(" ")'
    REL_19_BETA2 REL_19_BETA1 REL_18_4 REL_18_3 REL_18_2 REL_18_1 REL_18_0 REL_18_RC1 REL_18_BETA3 REL_18_BETA2 REL_18_BETA1 REL_17_10

Standing on 17 rather than 18 buys the operator whose distribution has not moved
yet and costs nothing this plan has so far named. No claim is made here that any
particular feature arrived in a particular major. Where a later record needs one,
it raises the floor in its own text and says which feature forced it.

Every floor above is a floor and not a ceiling. Raising one is a change to this
record, not a change made quietly in a workflow file.

## Can the means carry the three rules

A property a machine can refuse. Rust refuses at compile time the class this
project most wants refused, which is a record in a state that should not exist.
An enumerated type with data on its variants makes "a deal that is closed and
also has no close date" unrepresentable rather than merely tested for, and the
same shape carries the permission decision and the workflow step state. On the
client side TypeScript refuses a smaller set, which is why the contract the
client depends on is generated from the server rather than written twice; see
#25 for the surface and `docs/decisions/0009-web-client.md` under #62 for the
client half.

A proof that runs. `cargo test` is in the toolchain rather than beside it, so the
suite has no separate installation step to skip, and a test that must not be
skipped cannot be turned off by omitting a plugin. The client half runs under
Node with no display, which is what #7 turns into a gate.

A claim that cites the command behind it. Both toolchains print machine readable
output that a workflow can quote, `cargo metadata`, `cargo test --format json`
and `npm ls --json` among them, so a number in a document can carry the command
that produced it rather than a screenshot of it.

## Is anything outside this repository forcing this

The browser forces TypeScript, or something that compiles to what a browser
runs, and this project has no argument with that. It is the only forced means in
the set.

Nothing forces Rust. Nothing forces PostgreSQL. An operator's existing database
estate is an argument that will be made, and it is answered in the storage
record rather than here.

## What this adds that the tree does not already carry

The tree today carries no application language at all:

    git ls-tree --name-only HEAD
    .github
    LICENSE
    NOTICE.md
    README.md

It does already carry three things that are easy to overlook when counting.
Workflow YAML. Shell, in the `run:` blocks of those workflows. And Python, which
`uvx` fetches to run the workflow auditor:

    git grep -l 'uvx' -- .github/workflows/
    .github/workflows/zizmor.yml

So the decision adds two languages, Rust and TypeScript, and two runtimes, the
Rust toolchain and Node, on top of a tree that already carries three smaller
ones. The cost is named rather than minimised. Two toolchains means two lock
files, two formatters, two lint gates, two supply chains to review and two ways
for a contributor's local verdict to differ from the workflow's, which is why #2
pins both exactly and why #3 and #5 gate both from the first day rather than
bolting the second one on later. A contributor who can review a change to the
reporting engine is not automatically one who can review a change to the list
view, and the pool of people who can do both is small.

## Is the result testable by the suite that will exist

Yes for the server, and this is the strongest single argument in the set. The
two differentiators are a query path and a durable engine, both of which are
pure functions of stored history plus a clock, and both toolchains here can stub
a clock and a database without a parallel apparatus. The unit suite in #5 runs
with the outside world stubbed and needs no database; the integration suite is a
separate command whose name says what it needs.

The client is the weaker half and it is weak in a known way. A real browser is
not stubbed by the default suite, so the browser tests live behind the marking
#7 defines and are excluded from the default run by configuration. That is a
parallel apparatus, and calling it one is more useful than pretending the
marking makes it disappear.

## The alternative considered and not taken

TypeScript on both sides. One language, one package manager, one test runner,
one review pool, and the fastest route to a working CRM. For a product whose
visible surface is mostly lists and forms this is a real argument and not a
straw one.

What it costs is a runtime on the operator's host, which contradicts the single
artefact this project wants an operator to run beside a database; a dependency
tree an order of magnitude larger to review, against a licence and supply chain
posture the repository already gates on; and the compile time refusals that the
reporting and workflow engines are the specific reason for wanting.

## What would reverse this

Reverse it if the server spends most of its lines shaping requests rather than
in the two engines, which is measurable rather than a matter of taste: count the
lines under the API module against the lines under the reporting and workflow
modules once all three exist, and if the first dominates, the argument for the
second language has gone.

Reverse it also if review capacity on the server side is what actually limits
the project, meaning changes sit unreviewed for want of somebody who can read
them, rather than for want of somebody with time.

This record is downstream of #13. If that decision goes the other way and this
project extends a CRM that already exists, the means follows the core and #1 is
reopened.

## The numbering and naming every later decision record follows

Every decision record in this repository is a file at
`docs/decisions/NNNN-slug.md`, where `NNNN` is four digits with leading zeros and
`slug` is lower case words joined by hyphens, naming the subject rather than the
verdict.

A number is allocated by the issue that asks for the record, and it is written
into that issue's body before the file exists. It is not the next free number at
the time of writing, so the sequence has gaps while the issues that reserved them
are still open, and two records are never written under one number. What is
reserved is printed rather than listed here:

    gh issue list --repo iderex/kontor --state all --limit 200 \
      --search '"docs/decisions/" in:body' --json number,title

A number is never reused and a record is never renumbered, because the number is
how other records and issues refer to it. A record is not edited into a different
decision either. It is superseded: the new record names what it supersedes in its
header, the old one names what superseded it, and the old text stays readable so
that a reader can see what was believed and why it changed.

Each record carries the header block this one carries, then the decision, then
the reasoning, then the alternative and what it costs, then the condition that
would reverse it. Records answer the four questions above only where the record
is about a means. A record that decides something other than a means, for example
the shape of a log entry, answers what it is about.
