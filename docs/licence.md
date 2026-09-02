# The licence, and what follows from it

This note is for somebody deciding whether to run this software, build on it, or
offer it to other people. It says what the licence requires in plain language
and it names the commands that show the tree agreeing with itself. It is not
legal advice and nobody here is qualified to give any: where a decision turns on
a term below, the term is in [LICENSE](../LICENSE) and a lawyer is who reads it.

## Which licence, read rather than remembered

    gh api repos/iderex/kontor --jq .license.spdx_id
    AGPL-3.0

That is the GNU Affero General Public License, version 3, and the tree declares
the same identifier in the two places a package ecosystem asks for one:

    git grep -n 'AGPL-3.0-only' -- server/Cargo.toml client/package.json

`licence-scan` refuses a disagreement between those two lines before it judges
anything else, because a header rule comparing against one of two answers passes
a tree that has already lost the argument.

## What it asks of somebody running it

Nothing, as long as the software stays inside the organisation running it.
Running a program is not distributing it, an operator installing this for their
own staff has no obligation to publish anything, and modifications they keep to
themselves stay theirs.

The version number in that sentence matters. Version 3 is why version 2 of the
Apache licence can be in this dependency tree at all, and it is why the
paragraph below exists.

## What it asks of somebody offering it to other people

This is the term that makes this licence different from the one most open
projects use, and it is the reason it was chosen.

If you modify this software and let people use the modified version over a
network, those people have to be offered the source of the version they are
actually talking to. Not this repository, and not the version it was forked
from. The one running.

Offering the unmodified thing, from this repository, adds no obligation you do
not already have. Running a modified copy for your own staff adds none either,
under the paragraph above. What the term reaches is the case this project was
written against: somebody takes this, improves it, sells access to the
improvement, and returns nothing.

## What it asks of somebody building against it

Writing something that speaks to a running instance across a network is not
copying it, and nothing here reaches that.

Taking source out of this tree and putting it in another program is copying, and
the combined program is under this licence. That is what the header on every
source file is there to say. A file travels: copied into another tree it is read
on its own, and the root of this one does not go with it.

    licence-scan headers

judges every tracked source file for that header, and `licence-apply` writes it,
so the spelling is one spelling rather than one per contributor.

## What the dependencies may be under

A dependency arriving under a licence this project cannot be distributed with is
a defect that surfaces at a release, which is the most expensive moment for it
to surface.

    licence-scan dependencies

reads what every package in both toolchains declares about itself and compares
it against [licences-allowed](../licences-allowed), which carries a reason for
every entry and refuses one that carries none. Adding a licence to that file is
a decision somebody wrote down.

Two limits, stated rather than left to be discovered. It reads a declaration,
not a licence text, so a package declaring something it is not passes. And the
compatibility judgement behind each reason is a judgement: what is checkable is
that the question was asked and the answer is written where the next person can
disagree with it.

## What a contributor grants, and how it is recorded

Every commit carries a `Signed-off-by:` trailer whose name and address match its
author. That trailer is an assertion of the Developer Certificate of Origin,
version 1.1, whose text is in [DCO](../DCO): that the contributor wrote the
contribution or has the right to submit it, and that it may be redistributed
under this project's licence. The check runs on every pull request and fails
closed, and CONTRIBUTING.md is where the commands are.

So contribution is a certification rather than an assignment. A contributor
keeps the copyright in what they wrote, and it is licensed inbound under the
same licence this project ships under.

That is what is in force. Whether it stays the mechanism is an open question I
have not answered, and it is entry 1 of #129 rather than something this note
decides. What follows from the current answer is worth stating plainly because
it is the part people miss: with no assignment, changing this project's licence
later would need the agreement of everybody who has contributed by then, and
that gets harder with every merge.

## What this note does not cover

Whether the mobile clients can be distributed through the mainstream application
stores under this licence, and whether an exception is offered so that a client
speaking the sync protocol need not inherit it, are open questions rather than
settled positions. Both are in #129 with the options and what each one costs.
Nothing in this note assumes an answer to either.
