# 0004. One append-only change log, two readers

Status: accepted
Issue: #15
Supersedes: nothing
Superseded by: nothing

## The decision

Every change to a record is appended to one log. The log is never updated and
never deleted in place. Nothing writes a record without writing its history in
the same transaction.

Two things read that log and neither writes it. The reporting engine reads it to
answer a question about a past moment, which is what makes the change waterfall
of #42 possible at all. The workflow engine reads it as its trigger source, which
is #51.

That single source is the point. A workflow can never fire on a change reporting
cannot see, and reporting can never show a change no workflow could have seen.
Those two staying consistent is worth more than either feature on its own, and it
costs nothing extra only because the log is one thing rather than two.

## The alternative, and why the single log wins

The usual arrangement is two mechanisms: an audit table written by database
triggers, and an event stream written by application hooks. It is easier to add
to a system that already exists, which is exactly why it is what most systems
have.

It guarantees the two disagree. They are written at different points in the
transaction, they are maintained by different people for different reasons, and
one of them is eventually skipped by a code path somebody forgot. The failure is
silent in both directions: a workflow that never fired on a change the audit
table shows, and a report that shows a change no trigger recorded. Neither looks
like a bug from the outside. Both look like the feature being wrong.

The single log has no such failure mode available to it, because there is no
second thing to disagree with.

## The shape of a log entry

One entry per changed field, not one per changed record. A change that touches
three fields writes three entries sharing one transaction identifier.

    id            monotonic identifier, orders entries within and across transactions
    txn           groups the entries written by one transaction
    occurred_at   the commit time of that transaction
    object        which object type the record belongs to
    record        which record
    field         which field
    old_value     the value before, as jsonb
    new_value     the value after, as jsonb
    actor         who: a person, a service identity, or the system
    actor_kind    which of those three the actor is
    cause         user action, import, api call, workflow, or migration
    cause_ref     the import, workflow run or request that the cause points at

May never be null: `id`, `txn`, `occurred_at`, `object`, `record`, `field`,
`actor`, `actor_kind`, `cause`. An entry that cannot name who made a change or
what caused it is not written; the write fails instead.

`cause_ref` is null only where `cause` is a user action taken directly on the
record, since there is nothing further to point at.

The two value columns are the subtle part. They are `jsonb` rather than text
specifically so that two different things can be told apart. SQL `NULL` in
`old_value` means the field did not exist before this entry, which is what a
creation writes. The JSON value `null` means the field existed and was empty.
Collapsing those two is the error that makes a history of an optional field
unreadable, and it cannot be repaired afterwards because the information was
never written.

The identifier is monotonic rather than a timestamp because two transactions can
commit inside the same clock tick, and an as-of question needs a total order.
`occurred_at` is what a human reads; `id` is what the engines order by.

## The transaction rule

A record write and its log entries commit together or neither commits.

This is not a convention about calling order. It is the reason the log lives in
the same database as the records rather than in a queue, a broker or a second
store: a transaction is the only mechanism that makes the pair atomic without a
protocol on top. A design that ships the log somewhere else has to solve exactly
once delivery to keep the same promise, and it will not.

Two consequences follow and both are accepted. The log cannot be written
asynchronously to make a write faster. And an outage of the log is an outage of
writing, rather than a period during which writes succeed and history is lost.
The second is the behaviour this project wants: a refused write is visible and a
missing history is not.

## Retention, and what an as-of question does at the edge

The default is that the log is retained in full. It is not trimmed on a timer, it
is not rolled up, and nothing removes an entry as routine maintenance.

An operator may configure a retention horizon under #102, and if they do, entries
older than the horizon are removed. That is the only routine removal, and it is
an act the operator configures and can defend rather than a default they inherit.

An as-of question for a moment before the horizon does not return a value. It
returns an error naming the earliest moment this instance can answer and the
horizon that put it there. It does not reconstruct an answer from whatever
entries survive, because a reconstruction from a partial log is a wrong answer
that looks exactly like a right one, and a forecast built on it is wrong in a way
no test catches.

A report whose range crosses the horizon fails as a whole rather than returning a
series whose early points are silently different in kind from its later ones.
That is the expensive behaviour and it is chosen deliberately. The cost is real:
an operator who sets a one year horizon cannot run a two year waterfall, and
#102 is where that trade is put in front of them rather than hidden in a default.

One removal is not routine and is not retention. An erasure obligation under #101
removes the entries about a data subject inside the retained window. Where it
does, it leaves a tombstone entry in place of what it removed, so that a gap in
the history is visible as a gap rather than as a period in which nothing
happened. An as-of question that crosses a tombstone says so in its answer.

## What the log costs in rows and in storage

Arithmetic on stated assumptions, not a measurement. Every input below is an
assumption and is labelled as one. #126 is where the footprint is measured on a
stated machine, and the numbers here are what that measurement is expected to
check rather than a substitute for it.

Assume an instance holding 500,000 records across the five core objects. Assume
a record has 12 fields set when it is created, and is changed 20 times over its
life with a mean of 2 fields changed per change. Both assumptions are about a
small to medium business rather than about an enterprise instance.

    creation entries   500,000 records x 12 fields              =  6,000,000
    edit entries       500,000 records x 20 changes x 2 fields  = 20,000,000
    total                                                         26,000,000

So the rows per record change is exactly the number of fields that change, which
is the design and not an overhead: 2 on the assumption above, 12 for a creation.

For storage, assume a mean of 200 bytes per entry. That is the fixed columns at
roughly 60 bytes, the row header and item pointer at roughly 28, and the two
`jsonb` values at roughly 40 each for the short strings, numbers and identifiers
that make up most CRM fields, plus alignment. A long text field costs more and a
history of long text fields costs much more, which is the case that moves this
number most.

    heap        26,000,000 entries x 200 bytes                  =  5.2 GB
    indexes     26,000,000 x 2 indexes x 48 bytes               =  2.5 GB
    total                                                          7.7 GB

Two indexes because two readers ask two different questions: the reporting engine
asks for one record's entries in order, and the workflow engine asks for
everything after a position it holds.

Against that, assume the records themselves at 1 KB each, which is 0.5 GB. The
log is therefore expected to be roughly fifteen times the size of the data it
describes, and it never shrinks on its own. That ratio is the honest headline of
this decision and the reason retention is a first class concern in #102 rather
than an afterthought.

## What is refused

Any write path that reaches the record tables without going through the one path
that writes the log.

That is the whole rule, and it is worth naming the four routes that will try it,
because each of them has a reason that sounds good at the time. A bulk import
that inserts directly because per row writes are slow. A migration under #18 that
corrects data while it is changing shape. A workflow action that holds a database
handle for its own reasons. And an operator or a contributor with a SQL console,
who is the one route no code can refuse.

The failure is silent, which is what makes it worth this much text. A write that
skips the log leaves a record that looks entirely correct and a history that is
wrong about it forever, and the wrongness surfaces months later as a forecast
nobody can reconcile.

Nothing enforces this today. No check in this repository reads a module boundary
or a write path, and there is no code for one to read. #116 is where this rule
becomes a test over the compiled graph, which is the only route that can refuse
the first three of the four. The fourth is not refusable by anything in this tree
and is answered by the audit trail of #96 and by the operator's own controls.
