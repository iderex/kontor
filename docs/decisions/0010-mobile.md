# 0010. Native shells over one offline core

Status: accepted
Issue: #71
Supersedes: nothing
Superseded by: nothing

## The decision

The offline core is written once and is the permanent part. The shell is the
reversible part. One platform ships at first release rather than two badly, and
which one is not decided here.

## Why the split is where the argument is

The question is usually posed as native applications against a serious offline
capable client, and that framing hides the actual decision, because the two are
not alternatives.

A native application that cannot work offline is no better on a train than a web
page. An offline capable client that cannot reach the dialler, the calendar, the
camera and the notification system is missing exactly the things a salesperson
picked up a phone for. Both halves are required whichever way the question is
answered, so answering it decides nothing.

What the two halves do not share is difficulty. The offline core, meaning the
local store, the change tracking, the sync loop and the conflict resolution, is
the hard part and the part that is expensive to get wrong, because a conflict
rule that loses a change loses it permanently and invisibly. The shell is
ordinary application work. So the split puts the argument where the cost is: the
core is decided once and carefully, and the shell is decided in a way that can be
undone.

## What makes the shell reversible

The protocol, and only the protocol.

#72 specifies the sync protocol before any client exists, which is what makes a
client something that can be written against a document rather than reverse
engineered from the first one. #74 builds the conformance suite every client must
pass, which is what makes a second client a client that passes the suite rather
than a rewrite of the hard half.

Without both of those the shell is not reversible and this decision does not
hold, so they are the load bearing dependencies of this record rather than
neighbouring work. A shell built before the protocol is specified would fix the
protocol by accident, at whatever shape that first client happened to need.

#73 is the third piece, and it is part of the core rather than of the shell: the
conflict model, and the promise that nothing is silently lost.

## The shell decision

A shared core in a language that compiles to both platforms, with a native
interface layer on each.

The licences were read rather than remembered, since a licence that excluded a
candidate would settle the question before the merits did:

    gh api repos/JetBrains/compose-multiplatform --jq .license.spdx_id
    Apache-2.0
    gh api repos/facebook/react-native --jq .license.spdx_id
    MIT
    gh api repos/flutter/flutter --jq .license.spdx_id
    BSD-3-Clause

None of the three is excluded, so the choice is on the merits.

What it costs, stated rather than glossed. Two interface layers to build and
maintain. A shared core whose debugging story spans two toolchains, which is a
real tax on the day a bug is in neither half cleanly. And a first release that
leaves half the potential users waiting, which is the cost the decision below,
which is mine rather than the plan's, actually spends.

## The alternative considered

A single cross platform interface toolkit for both platforms, which halves the
interface work. For a product whose surface is forms and lists this is a
completely reasonable answer and not a straw one.

It costs the platform affordances at the margins, which is where they are least
well supported by such a toolkit and most visible to a user, and #79 is the issue
that spends its whole length on exactly those. And it costs a rendering layer
between the user and the system, which shows up in the interactions a salesperson
repeats fifty times a day rather than in the ones they do once.

Reverse this decision if the native interface work turns out to dominate the
milestone, meaning the interface layers consume more of it than the core and the
protocol together. Reverse it before the second platform starts rather than
after, because the second platform is where the saving would be taken and after
it is built there is nothing left to save.

## What offline means here, precisely

Stated concretely, because offline capable is a phrase that means whatever the
reader hopes.

With no connection, a client can read every record in its synced set; create a
person, an organisation, a deal or an activity; edit any record in the synced
set; log a note, a call or a meeting; move a deal between stages; and complete
or reschedule a task. All of it is recorded locally as changes and queued.

The synced set is bounded rather than the whole instance. It is the records the
user owns or follows, plus the records they touched inside a stated recency
window, subject to a stated cap on rows and bytes that #76 sets alongside the
encryption question and #81 measures against a budget. A phone does not hold a
company's database, and a design that assumes it does fails on the largest
operator first.

For how long: until the queued changes reach the bound the client states, at
which point it refuses new offline changes and says so, rather than accepting
them and dropping the oldest. Refusing is visible; dropping is the failure this
whole plan is written against.

What a client cannot do offline at all: run a report, because reporting reads the
full change log and that is not on the device and is not going to be; search
outside the synced set; see a record it has not synced; trigger the effects of a
workflow, since runs execute on the server; change a permission; import or
export; or send mail. Each of those is refused with a message naming the
connection as the reason, rather than appearing to work and failing later.

When the connection returns, the queue is sent in order and the conflict model of
#73 decides what happens where the server moved underneath. Nothing in the queue
is discarded to resolve a conflict; that is the promise #73 carries and this
record depends on.

## What this record does not decide

Which platform ships first.

That is mine to decide rather than a plan decision, because it depends on who
the intended operators are and on what I intend to do about store distribution,
and both of those are choices about the project rather than consequences of the
engineering. Store distribution in particular carries an obligation attached
to a person rather than to the project.

It is recorded as an entry in #129 and it is not answered here, in either
direction, deliberately. Nothing in this record assumes an answer: the core, the
protocol and the conformance suite are identical whichever platform is chosen,
which is the point of putting the split where it is.
