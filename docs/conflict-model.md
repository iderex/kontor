# The conflict model

Two people editing one deal while one of them is offline is the normal working
day of a sales team rather than an edge case, and the rule that decides what
happens is the single thing that makes this product trustworthy or not. This
document states that rule, before there is any code to describe.

## The promise, in one paragraph

A change somebody made on this system is never thrown away without a person
deciding to throw it away. Where two people changed the same field and one of
them was offline, the instance does not pick a winner behind their backs: it
keeps both values, shows the person who was offline what they did not see, and
asks them. Where a change can be applied without ambiguity it is applied, and
what it replaced stays readable in the change log with the name of whoever put
it there. There is one case where an edit is dropped and not recorded, and it is
the case where recording it would defeat somebody's right to erasure; that case
is named in this document rather than left to be discovered.

That paragraph is what an operator may quote to their team. Everything below is
how it is kept.

## What this document is

The rule the sync protocol says it does not decide:

    git show origin/main:docs/sync-protocol.md | sed -n '651,654p'
    ## What this document does not decide

    The conflict resolution rule. #73 holds it, `outcome` carries its result, and what
    is fixed here is only that a discard is not expressible.

So the edges are already fixed and this document works inside them.
`docs/sync-protocol.md` fixes four outcome states, fixes that a `conflicted`
outcome leaves the change in the client's queue, which is R15, and fixes that no
message shape lets a server report a queued change was discarded, which is R30.
Nothing here adds a message shape, and that is a constraint this document was
written under rather than a coincidence: a resolution rule that needed new wire
shapes would be re-opening a specification that is already the thing #74 writes a
conformance suite against.

The log this document writes to is the one log of
`docs/decisions/0004-change-log.md`, with the entry shape that record states. The
objects the rule is applied to are the ones `docs/model.md` defines. The two-step
deletion the rule reads on is the one `docs/decisions/0003-custom-fields.md`
describes for a field and #20 decides for a record.

## What this document is not

It is not an implementation and nothing in this tree implements it. There is no
sync module and there is no client, so every statement below is a design that a
later change is measured against rather than a description of behaviour anybody
can run.

It is not proved. The property this model exists to guarantee, that random
interleavings of offline edits across two clients converge and that every
submitted change is either applied or discoverable, needs two clients and a
server to interleave. That is the sixth condition of #73 and it is unmet, which
is why this document lands without closing that issue.

## The unit of a conflict is one field of one record

A `change` on the wire is one field of one record moving, and the log writes one
entry per changed field rather than one per changed record. The conflict rule is
stated at the same grain, and the consequence is worth stating plainly because it
is the behaviour people notice first: two people editing one deal do not conflict
unless they edited the same field of it. One changing the amount while the other
changes the stage is two changes that both apply, and neither person is asked
anything.

That is a deliberate trade rather than a free win. A record can end up in a state
neither person would have chosen, because the amount one of them set was
reasonable only under the stage the other one moved away from. This model does
not try to detect that: a rule that guessed at which fields are meaningful
together would be wrong about custom fields it has never seen, and the failure of
guessing wrongly is that people are asked to resolve conflicts that are not
conflicts, which is how a resolution prompt becomes something everybody clicks
through. The change log is what makes the combined state readable afterwards, and
the person who cares is the one who reads it.

## The version a change is judged against

`change` carries `base`, the cursor the client held when it made the change. A
cursor is a position in the one log, and the log's identifier is monotonic and
totally ordered, which `docs/decisions/0004-change-log.md` chose so that an
as-of question has an order to read. So the version is assigned by the server,
never by the client, and never by a clock.

The question the server asks of every pushed change is exactly one question.

    has this object, this record, this field been written since `base`?

It is answerable from the log alone, at the field grain, with no extra column on
the record and no version counter to keep in step with anything. `made_at` on a
change is never part of that answer. It is what a person reads, and two clients
disagree about what time it is, which is why the protocol says of that field that
it never decides an order.

## The rule for a scalar field

Four answers, and they are the four outcome states the protocol already carries.

**The field has not moved since `base`.** The change is applied. The person made
it knowing what the field held, and what it held is written to `old_value` of the
log entry the write appends, so the replaced value keeps its author and its
moment. This is the ordinary case and it is the great majority of every push.

**The field has moved since `base`, and the pushed value is what the server now
holds.** The outcome is `superseded`. Nothing is written, because writing it
would append a log entry recording a change from a value to itself, and nothing
is lost, because the record already says what the person wanted it to say.

**The field has moved since `base`, and the pushed value differs from what the
server holds.** The outcome is `conflicted`. The server does not write the value
and does not discard it. It creates a conflict record, described below, which
carries the offered value and its author, and it answers with `detail` naming
what it holds and the log position that moved it. The change stays in the
client's queue, which is R15, and it leaves that queue when a person has decided
about it.

**The change cannot be applied at all.** The outcome is `refused`, which is the
schema having moved under the change, the field not existing, the value not being
one the field admits, or the caller not being allowed to write it. A refusal is
not a discard either: the client keeps the change and shows the person the
reason, which is what R16 already requires for the schema case and what this
model extends to the rest.

The word that is not in that list is silent. There is no answer in which the
server prefers one person's value over another's without saying so, and there is
no message shape that could carry one if there were.

### Worked example: the ordinary case

Two people, one deal, one field, and nobody is asked anything.

    the log holds        c:18422  deal r:41ab stage -> "qualification"  by Ida
    on a phone, offline  Ravi sets stage to "negotiation", base c:18422

    push    { "key": "k:8ac1", "object": "deal", "record": "r:41ab",
              "field": "stage", "value": "negotiation",
              "made_at": "2026-08-09T14:02:11Z", "base": "c:18422" }

    receipt { "key": "k:8ac1", "state": "applied", "detail": null }

    the log now holds    c:18630  deal r:41ab stage
                                  old_value "qualification"  new_value "negotiation"
                                  actor Ravi  cause user action

Ida's value is not gone. It is `old_value` on the entry that replaced it, with
Ravi named as the actor who replaced it and the commit moment of the transaction
that did.

### Worked example: the same value from both sides

    the log holds        c:18500  deal r:41ab stage -> "negotiation"  by Ida
    on a phone, offline  Ravi sets stage to "negotiation", base c:18422

    receipt { "key": "k:8ac1", "state": "superseded",
              "detail": { "held": "negotiation", "moved_at": "c:18500" } }

No entry is written. Two people agreeing is not a conflict and it is not a
change, and a log that recorded it would report an edit that changed nothing.

### Worked example: the real conflict

    the log holds        c:18500  deal r:41ab stage -> "closed-won"  by Ida
    on a phone, offline  Ravi sets stage to "negotiation", base c:18422

    receipt { "key": "k:8ac1", "state": "conflicted",
              "detail": { "held": "closed-won", "moved_at": "c:18500" } }

The record still says `closed-won`. Ravi's change is still in Ravi's queue. And
a conflict record now exists on the instance holding `negotiation`, Ravi's name
and the moment Ravi made it, so the edit survives the phone being lost, which is
the half a queue on a device cannot give.

Ravi sees both values and decides. Taking the held value is one field change on
the conflict record and nothing else; the offered value stays on the conflict
record and stays in the log, so it is recoverable by anybody who later asks what
happened. Taking his own value is that same field change plus a fresh `change`
for the deal, made with a current `base`, which then applies by the first rule
above and writes `closed-won` into `old_value` with Ravi as the actor.

Nothing about this is automatic and that is the decision. The person who was
offline is the only one who can say whether a stage they chose an hour ago still
applies now that they can see somebody closed the deal.

## Appended things never conflict

A note, an activity and an attachment are appended rather than replaced, so two
people adding one while both are offline produce two of them and no question is
asked of anybody. `docs/model.md` gives an activity its own identity rule, which
is the identifier of the thing it came from, so two clients that both sync the
same mail produce one activity rather than two; two people who each typed a note
by hand produce two, because a note somebody typed has no identity rule and
guessing at one would merge two people's words.

The rule that makes this hold is that an append is never expressed as a change to
a field holding a list. A list-valued field would conflict on every concurrent
append, and it would conflict in the worst possible way, by making one person's
note the `old_value` of the entry that recorded the other's.

### Worked example: two notes, no conflict

    on a phone, offline    Ravi writes a note on deal r:41ab
    in the web client      Ida writes a different note on deal r:41ab

    both appear, both keep their author, nobody is asked anything, and the
    order they are read in is the order of the log rather than of either clock.

Editing a note somebody already wrote is not an append and is not covered by this
section. It is a scalar field of the activity record and it takes the scalar rule
above, conflict record and all.

## A record deleted on one side and edited on the other

The deletion this rule reads is the two-step one: the first step hides the
record and is reversible for a stated window, and the second step is the
permanent erasure that is a separate act somebody confirms.
`docs/decisions/0003-custom-fields.md` describes that shape for a field and says
the record case is #20 and that where the two differ #20 is right.

**The record was hidden on the server and edited on the client.** The outcome is
`conflicted`, not `applied` and not `refused`. A conflict record is created
holding the offered value, and `detail` names the hide and who made it. The
person is shown that somebody deleted the record and what they themselves had
written, and they may restore the record and apply their edit, or leave it
hidden. The edit is not applied silently to a record on its way out of the
instance, and it is not dropped because somebody else got there first. Neither
side wins by default, which is what the fourth condition of #73 asks for.

**The record was erased permanently and edited on the client.** The outcome is
`refused`. No conflict record is created and the offered value is not written
anywhere on the instance, and this is the one place in this model where an edit
is dropped without leaving a trace on the server. The reason is that a conflict
record holding the value would put the erased person's data back on the instance
under a different name, which is the obligation of #101 defeated by the
mechanism this document exists to build. The client tells the person that the
record was erased and that their edit cannot be applied, and the person clears it
themselves. A person knowingly discarding their own edit is not a silent loss;
the instance holding a copy of it would be.

**The record was hidden on the client and edited on the server.** The rule is
that a hide made offline is judged like any other change, against what the record
looks like now: where somebody has written the record since the hider's `base`,
the outcome is `conflicted` and the person who deleted it is asked, because
somebody working on a record is the strongest available evidence that deleting it
was wrong. Where nothing has moved, the hide applies.

That third case cannot be expressed on the wire today, and this is the finding
rather than an implementation detail. `change` carries an object, a record, a
field and a value, and there is no shape a client can send that means the record
itself is hidden or restored: the only deletion in the protocol is `gone`, which
travels from the server to the client. So the rule above is stated and is
unreachable until the protocol carries the operation, which is #72 to add and #20
to define. Until then a client that has no way to send a deletion has no way to
lose one either, so the promise is not weakened by the gap; what is missing is
the feature, not the guarantee.

## Anything the rules do not cover

A conflict record, shown to a person, asked rather than decided. It is the
default in the true sense: where this document does not say what happens, what
happens is that somebody is asked.

A conflict record is an ordinary record rather than a shape of its own, and that
choice is what keeps this model inside the protocol. It syncs to a client in
`records` on an ordinary `page`, it is written inside the transaction that
answers the push, so it is in the log by the transaction rule of
`docs/decisions/0004-change-log.md`, it is permissioned by whatever #21 decides
for the record it is about, and it is resolved by an ordinary `change` on one of
its own fields. No new message, no new outcome state, and nothing a client has to
learn beyond a record type.

What it carries: which object, which record and which field the conflict is
about; the value the server held and the log position that put it there; the
value that was offered, its author and when they made it; and, once somebody has
decided, which value they kept and when. Enough that a person reading it a month
later can see both values and who chose.

Two people resolving one conflict is not a special case. The resolution is a
scalar field of the conflict record, so the second person to resolve it meets the
scalar rule: the same decision is `superseded`, a different one is `conflicted`,
and the argument two people are having about a deal becomes an argument they can
see they are having.

### Worked example: a case no rule covers

A change arrives for a field whose type changed from a picklist to a text field
while the client was offline, and the offered value is not one of the values the
picklist held.

The schema half is a refusal and the protocol already names it: `schema-moved`,
per R17, with the object, the field and the current version. The part this model
adds is what happens to the change afterwards, which is that the client keeps it,
per R16, and that where the operator later makes the value admissible the person
can push it again. Where the value can never be admissible the person is told so
and clears it, and the instance records nothing, because nothing about the value
ever reached it.

## What a client may not do

Three behaviours would break the promise without breaking any rule above, so they
are stated as refusals rather than left to be inferred.

A client may not drop a change from its queue on a `conflicted` outcome. R15
already says the change stays; what this adds is that only a person's decision
removes it, never a retry, never a restart, and never a sync that ran while
nobody was looking.

A client may not resolve a conflict by re-pushing the same change with a fresh
`base`. That is the automatic overwrite this document refuses, wearing the
clothes of the ordinary case, and it is the shortest path from this model to
losing an edit. A conflict is resolved by somebody choosing, and the fresh
`change` is what that choice produces rather than what stands in for it.

A client may not merge two values into a third on anybody's behalf. Concatenating
two notes, summing two amounts and taking the later of two dates are all things
a person may do and no client may do for them.

## What this model does not decide

Which conflicts are worth showing at all, meaning whether a conflict record on a
field nobody looks at should raise anything in the interface. That is a client
question and it belongs with the record page of #65 and the field workflow of
#77.

The permissions on a conflict record. It carries a value from the record it is
about, so it can reveal a field the reader may not read, and #21 is where that is
evaluated rather than here.

How long a conflict record is kept. It is a record, so it is subject to whatever
retention #102 sets and to the erasure of #101 like any other, and this document
adds no horizon of its own.

The wire shape for a deletion made offline, which is the gap named above and
belongs to #72 and #20.

## What this document does not prove

Nothing here is refused by any check in this tree. `docs-scan` judges that the
links and the document references resolve, that the words are the words the
register names and that the file is UTF-8 with no carriage return in it. None of
that reads a sentence, and no reading of this tree decides whether a rule stated
here is the right rule or whether anything later obeys it.

The convergence property is unmet and unproved, and it is the sixth condition of
#73. Two clients and a server module are what a property test over random
interleavings needs, and the tree holds neither.
