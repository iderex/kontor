# 0008. The trigger, action and execution model

Status: accepted
Issue: #49
Supersedes: nothing
Superseded by: nothing

## The decision

Triggers read the change log and nothing else. Actions are a closed set. Every
run is durable, delivered at least once, and inspectable.

The gap this closes is not the number of actions available. It is whether a run
survives a restart, whether anyone can find out what a run did, and whether the
thing that fires a workflow sees the same world the reports see.

## The trigger model

A workflow fires because an entry appeared in the change log. Not because a hook
was called somewhere in the request path, and not because a piece of code
remembered to publish an event.

What that buys is that the set of things a workflow can react to and the set of
things a report can see are the same set, by construction rather than by
discipline. A change made by an import, by the API, by the browser or by another
workflow triggers identically, because all four write through the one path that
writes the log, which is what `docs/decisions/0004-change-log.md` refuses any
alternative to. #51 builds this.

The alternative is a hook in the write path, which every system that grew its
automation later has. It is faster to add and it is synchronous, which is the
thing people actually want from it. It costs the failure mode this model does not
have available to it: the code path somebody forgot to instrument. That path is
never the one anybody tests, because the person who forgot to instrument it also
did not think to test it, and the symptom is a workflow that works for a year and
then silently does not for records that arrived by a route added last month.

What this model costs, and it is a real cost rather than a rounding error. A
trigger is not synchronous with the write. A workflow sees a change after it has
committed, so it cannot refuse one.

## The boundary: a workflow reacts, it does not veto

Stated plainly because it will be asked for, repeatedly, and because discovering
it late is expensive.

A workflow cannot prevent a change. It cannot reject a value, cannot block a
stage transition, and cannot roll back the write that triggered it. What it can
do is write a correction, which is a new change with the workflow as its actor,
visible as such in the history.

Refusal belongs to the metadata layer, where validation lives, and that is #17. A
rule that must refuse is a validation rule and is configured there. A rule that
must react is a workflow. Anyone asking for a workflow that vetoes is asking for
a validation rule, and the answer is to build it in the layer that can hold it
rather than to make triggers synchronous.

The reason not to bend on this is that a synchronous trigger puts arbitrary
operator configured work inside the write transaction, where it holds locks and
where its failure fails the user's save. That is the same escalation the closed
action set below exists to prevent, arriving through a different door.

## The action model

Actions are a closed set. Each one is a defined operation with declared inputs,
declared effects and declared failure behaviour, registered in the one place #54
assembles them.

There is no action that runs arbitrary code at first release.

The reason is the deployment this product is for. An operator who can create a
workflow would otherwise be an operator who can execute code on the host, and in
a self hosted product the administrator is often one busy person holding every
role at once. Handing out code execution through a form is a privilege escalation
surface with a friendly interface on it, and it is the kind that is never
reviewed because it does not look like a deployment.

The expression language of #55 is the bounded exception, and it is bounded in the
one direction that matters: it computes over the record and the trigger entry and
it may not call out. That is the whole of its power and #55 is where the boundary
is enforced rather than described.

## Whose permissions a workflow's writes carry

A workflow runs as a named identity that the operator chooses when they publish
it. Not as the person whose change triggered it, and not as an unrestricted
system account.

Not as the triggering person, because then the same workflow does different
things depending on who typed. It would succeed when a manager moved a deal and
fail when an intern did, which is a workflow whose behaviour nobody can predict
or test, and which leaks the permission model into an automation's results.

Not as an unrestricted account, because then anybody who can cause a change can
cause writes they could not have made themselves. Every trigger becomes a route
to every permission, and the route is invisible from the permission model.

So it is an explicit identity, holding ordinary permissions from the model #21
defines, evaluated exactly as they are for a person. An operator can answer what
this workflow is allowed to do by reading that identity's permissions, which is
the same question and the same answer as for anyone else.

Publishing a workflow requires the publisher to hold every permission they give
it. Otherwise delegation is escalation: a person with limited rights publishes a
workflow under a wider identity and triggers it themselves.

Every write a run makes carries that identity as the actor in the change log,
with the cause recorded as a workflow and the cause reference pointing at the
run. So the history answers who changed this and why with a run somebody can then
open, which is #59.

## The execution model

State is written before an effect is attempted. A run resumed after a restart
reads what it recorded and continues from there, so it knows what it already did
rather than inferring it.

Delivery is at least once, with an idempotency key on every outbound effect.
Exactly once across a process boundary is not available, and a system that claims
it is a system that has not yet had the crash that proves otherwise.

In words a user of the product would understand, which is how this promise has to
be written because it is the one they will meet:

Every step of a workflow runs at least once. If the server stops in the middle of
a run, the run picks up from the last step it finished. A step that had started
but had not recorded its result may run a second time.

What a duplicate looks like when one happens: a second copy of the same message
to the same address, or a second delivery of the same outbound event, both
carrying the same key as the first. A receiving system that honours the key
recognises the repeat and does nothing; one that does not will show the thing
twice. A duplicate is not a lost run and it is not a wrong value, and the choice
to allow it is deliberate, because the alternative failure is a run that stops
halfway and does nothing more, which is worse and much harder to notice.

Inside this product, a duplicate never doubles a record change, because a write
carries the run and step as its idempotency key and the second attempt is
recognised. The duplicate is visible at the boundary, where an outside system is
what decides.

The clock and the outside world are both substitutable, which is what makes any
of this testable without waiting, and #60 is where that is built.

## What would reopen the scripting runtime

The alternative considered is embedding a scripting runtime, which every mature
automation product eventually does, and which makes the hard cases easy.

It costs a sandbox that has to hold against an adversary who is also the person
configuring it, a resource budget enforced inside the runtime rather than by
convention, and a supply chain for whatever the scripts import.

The condition to revisit is that the closed action set has been demonstrably
insufficient for named cases, with those cases written down as issues at the time
they were hit rather than recalled afterwards. A general sense that scripting
would be useful is not the condition.

If it is revisited, what it would have to satisfy, all of it rather than most:

No host filesystem and no network except through the declared actions, so the
runtime adds no route out of the host that #97 does not already know about.

A CPU, memory and wall clock budget enforced by the runtime itself, refusing at
the limit rather than logging that it passed one, which is the shape #58 requires
of a workflow that would run away.

A clock that #60 can substitute, so a script is testable under the same simulated
clock as everything else.

A declared and reviewable supply chain for anything a script imports, held to the
same standard as #114 holds the rest of the tree.

A separate opt in surface with its own permission, not an action sitting beside
the others in the same list. An operator enabling it is told plainly, in the
interface and not only in a document, that it is code execution on their host.

Until all five exist, the closed set is the answer, and its insufficiency is
recorded case by case rather than assumed.
