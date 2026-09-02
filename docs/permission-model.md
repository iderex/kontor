# Who may see and change which records

A sales team is the setting this model exists for. One person must not see
another's pipeline, the person who manages them both must see the whole of it,
and neither a report nor a workflow may become the route around either rule.

The promise, in the paragraph an operator can quote to their team: what you may
see and change is decided once, in one place, and every route into this instance
asks that same place the same question. A report you can run over records you
cannot open shows you no more than opening them would, and an automation you can
trigger writes no more than the identity it was published under could have
written by hand.

This document names the levels, says where a level comes from, and says how two
of them combine on one record. It is the first of the six conditions of #21, and
it is written before the module rather than after it, because the other five are
checked against this and cannot be checked against an implementation that is its
own specification.

## What a permission is

A permission is one sentence with three parts: a **principal** holds a **level**
on a **set of records**.

A principal is a person, a service identity, or a workflow identity. All three
are the same kind of thing to this model and are evaluated identically, which is
what `docs/decisions/0008-workflow.md` already requires of the third. The change
log calls the same three an actor, in `actor` and `actor_kind`, so the answer to
who may change this and the answer to who did change this are written in one
vocabulary.

A level is one of the four below. A set of records is whatever the source of the
grant describes: one record, every record of an object, every record owned by a
person, or every record owned by anybody below a role in the tree.

## The four levels

They are a ladder rather than a set of independent capabilities, and the whole
of the combination rule below depends on their being ordered.

**`none`.** The record is not there. It is absent from a list, from a count,
from a search result and from every aggregate. Asking for it by identifier
answers the same way as asking for an identifier that was never issued, because
an answer that distinguishes the two tells a caller which records exist.

**`read`.** Every field of the record may be read, except a field the field rule
below restricts further.

**`write`.** `read`, and a field may be changed. Creating a record is a `write`
on the object rather than on a record, since the record does not exist yet.

**`manage`.** `write`, and three more things: the owner may be changed, a share
may be granted or withdrawn on this record, and the record may be hidden or
restored under whatever #20 decides hiding means.

Nothing above `manage` exists. An administrator is not a fifth level; it is a
principal that holds `manage` everywhere, which is stated below with what that
costs.

## Where a level comes from

Six sources, and a principal's level on a record is whatever these give them.

**Ownership.** Every record has exactly one owner, who is a person. The owner
holds `manage`. A record whose owner has left the operator is still owned by
that person until somebody with `manage` moves it, because a record with no
owner is a record with no `manage` and so no route back.

**The role tree.** Every person holds one role, and roles form a tree in which a
person's role is the parent of the roles of the people they manage. A principal
holds, on every record, the highest level held by anybody in the subtree below
their own role. This is the manager sees the whole thing rule, and it is the
only source that is transitive: two levels up sees what one level up sees.

The tree is the operator's shape rather than this project's, and a cycle in it
is refused for the same reason the organisation tree of `docs/model.md` refuses
one.

**The object default.** Per object, per role, a level that applies to every
record of that object. This is where an operator says that everybody may read
every organisation and nobody may read a deal they are not otherwise entitled
to. The default for a new object is `none` for every role, because a default of
`read` is a decision to disclose taken by whoever wrote the installer rather
than by the operator.

**A team.** A named set of people, granting a stated level on the records owned
by the members of that team. A team is not a role: it grants sideways rather
than upwards, it has no tree, and a person may be in several. Territory selling,
where four people work one account list and none of them manages another, is
what a role tree cannot express and this can.

**A share.** A grant of a stated level on one record to one principal, made by
somebody who holds `manage` on it. This is the exception route, and it exists so
that an operator does not widen a role to solve one deal.

**Administration.** A principal marked as an administrator holds `manage` on
every record. `docs/threat-model.md` already says what this model asks about
that actor, which is not whether they can be stopped but whether what they did
is recorded where somebody else can read it.

## How two combine

**The highest level any source gives wins, and there is no source that takes a
level away.**

So a person who owns a deal, is in a team that reads it, and holds an object
default of `none` on deals, holds `manage`. Evaluation is a maximum over the
sources rather than a walk that can be ordered differently by two callers, which
is what makes the answer independent of the order the grants were written in.

The alternative is a negative grant that overrides a positive one, and it is
refused here rather than left unimplemented. It costs two things this model is
not willing to pay. The effective level stops being derivable from the sources
that mention a record and becomes derivable only from every rule in the
instance, since any of them might be the one that denies. And it makes the model
non-monotonic in the role tree: promoting somebody could reduce what they can
see, which is a sentence no operator would predict and no test would think to
write.

What that costs, stated rather than glossed: taking access away means finding
and removing every source that grants it, and this model owes the operator a
route that answers **which grant gives this principal this level on this
record**. That route is not a convenience. It is the only way a model with no
deny can be administered, and an implementation of this that cannot answer it
has implemented something else.

## Fields

A field carries the level needed to read it and the level needed to write it.
Both default to the obvious one, being `read` and `write`, and a field may raise
either.

A field that requires `manage` to read is how an operator holds a commission
rate, a personal telephone number or a contract term inside a record that a
whole team may otherwise read. The value of a field a principal may not read is
absent from the record rather than blanked, for the same reason `none` is absent
rather than refused: a blank tells the reader there is something there.

Lowering below the record's own level is not expressible. A field readable by
somebody who may not read the record is a record boundary that leaks one field
at a time, and the model that allows it cannot state what a caller can see
without enumerating every field of every object.

The conflict record of `docs/conflict-model.md` is the case this rule was
written against, and that document defers here deliberately. A conflict record
carries a value out of the record it is about, so it is read at the level the
principal holds on **that** record, and a value from a field they may not read
is absent from the conflict record too. A conflict on a field somebody cannot
see is shown to somebody who can.

## Aggregates

An aggregate over records the caller may not see **excludes them, and says how
many it excluded, next to the number**. It does not refuse, and it never counts
them.

This is the fourth condition of #21 answered in the direction `docs/model.md`
already answers the same question for a deal with no amount: every aggregate
that excluded a record says how many it excluded, in the same response and in
the same view. A pipeline total that silently means a different thing for two
people is the failure `docs/threat-model.md` calls a permission failure that
does not look like one, and the count beside the number is what makes it look
like one.

Refusing was the alternative and it is worse in the ordinary case. A sales
person whose permissions are exactly what they should be would meet a refusal on
almost every report they ran, because almost every report covers the team, and a
product that refuses correctly configured use teaches its operator to widen
permissions until the refusals stop.

The count itself is a disclosure and this document does not pretend otherwise.
It tells a caller how many records exist that they may not see, which is a
cardinality they did not have before. It is accepted deliberately: cardinality
is the smallest thing that can be disclosed while still telling the reader that
their number is partial, and no field, identifier or grouping of an excluded
record is ever in the response. Where even the cardinality is too much, the
answer is that the caller may not run the report at all, which is an object
level grant rather than a decision taken per number.

## The change log, and everything derived from records

A change log entry is read at the level its subject is read at. An entry naming
a record a principal holds `none` on is not returned, is not counted, and does
not appear in a history, a waterfall or a conversion rate; an entry whose
`field` a principal may not read is not returned either.

This is stated here rather than in `docs/decisions/0004-change-log.md` because
it is a permission rule and that record is about the shape of an entry. It
matters more than it looks: the history is where a value that has since been
changed is still readable, so a permission enforced on the record and not on its
history is a permission that expires the moment somebody edits the field.

The same holds for every surface computed from records, which is what
`docs/decisions/0007-reporting.md` states when it says the record set behind
every number is the permission filtered set for the caller, and the same
filtering as a record read rather than a second implementation of it.

## A record a caller may not read, referenced by one they may

A deal names an organisation. A person may hold `read` on the deal and `none` on
the organisation.

The reference is present and carries no field of the record it points at. Not
omitted, because omitting it says the deal has no organisation, which is false
and is a worse answer than saying there is one you may not open. What is
disclosed is that a related record exists, and nothing else: no name, no
identifier that could be asked for elsewhere, and no count of what else points
at it.

## Whose permissions a workflow's writes carry

Decided already, in `docs/decisions/0008-workflow.md`, and read from there
rather than restated: a workflow runs as a named identity the operator chooses
at publication, holding ordinary permissions from this model and evaluated
exactly as they are for a person, and publishing requires the publisher to hold
every permission they give it.

What this document adds is the one thing that record leaves to it. The identity
is a principal here in the full sense, so it owns records, sits in the role tree
where the operator puts it, and is subject to every rule above. An identity
outside the tree is the unrestricted account that record refuses, arriving under
another name.

## One place, and what that means for a query

The second condition of #21 is that evaluation happens in one place and that the
API, the reporting engine and the workflow engine call it rather than each
carrying a copy. `docs/layout.md` names the same rule as one of the three it
cannot yet turn into a test.

The part worth writing down before the module exists is what one place has to
mean for the reporting engine, because the obvious reading of it is wrong. The
evaluator cannot be a function that takes a record and answers yes or no, called
after a query returns. A post filter breaks an aggregate, since the sum was
computed before it ran; it breaks pagination, since a page of fifty becomes a
page of thirty one; and it reads rows the caller may not see in order to decide
not to return them, which is the read this model is about.

So the evaluator has two forms over one definition: the level a principal holds
on one record, and a predicate over the same sources that composes into the
query the reporting engine generates. They are one place in the sense the
condition means, which is that the sources and their combination are written
once, and a change to a rule moves both forms or compiles as neither.

`docs/decisions/0006-api-shape.md` already constrains what that predicate has to
survive: the grammar refuses a join beyond one declared relationship hop partly
so that the permission model stays evaluable on the result.

## What this model does not decide

Who a principal is, and how they prove it. That is authentication, and it is
#26. This model starts after the caller has a name.

What hiding a record means, what restoring it means and where it really goes,
which is #20. This document says only that hiding is one of the three things
`manage` adds.

Whether reading a record is recorded, and who may read that recording. That is
the audit trail of #96, and it is a different question from whether the read was
allowed.

How long anything is kept, which is #102, and what an erasure removes, which is
#101. A permission decides who sees a record while it exists.

The interface for granting any of this. A share made by somebody with `manage`,
a team's membership and the role tree are all edited somewhere, and where that
is belongs with #64 and #65.

The defaults a new instance is seeded with. Those are #122, and this document
fixes only that a new object defaults to `none` rather than to anything else.

## What this document does not prove

Nothing in this tree refuses a sentence in it. There is no permission module and
no caller for one:

    git grep -c '' origin/main -- server/crates/api/src/lib.rs server/crates/record/src/lib.rs

so nothing evaluates a level, nothing filters a query and nothing enforces a
field rule. Five of the six conditions of #21 are about that mechanism and this
document meets none of them; it is what they will be checked against.

The one thing enforced today is the boundary such a module would live behind
rather than the rule itself, and it is worth being exact about because it is the
kind of near miss that reads as coverage. `layout-scan` refuses a dependency
edge between the modules, and `docs/layout.md` says in the same place that the
single write path, the one module that evaluates a permission and the client
depending only on the generated contract are three rules no check in this tree
holds. A reader is the whole mechanism for whether anything later obeys this,
and #116 is where that changes.
