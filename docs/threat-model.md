# The threat model

A customer relationship manager reachable from the internet, holding an entire
company's customer relationships, with an automation engine that can make
outbound requests, is a specific target with a specific shape. This document
writes that shape down, so that the security work is a set of decisions somebody
can disagree with rather than a set of habits nobody wrote down.

## What this document is, and the state it was written in

It is the model, not a report on an audit and not a claim that anything is safe.
It names what is worth taking, who would take it, the routes they would take it
through, and for each route what stops them and where that is enforced.

It is written against a tree that holds almost none of the surfaces it walks.
There is no request path, no database, no authentication, no connector and no
client with behaviour, which `SECURITY.md` already states as the reason most of
what this project will eventually cover cannot be attacked today. So the honest
answer to "where is this enforced" is, for most of this document, that it is not
enforced anywhere yet, and this document says that in each place rather than once
at the top where a reader stops noticing it.

That is a real limitation and not a formality. A model written against designs
rather than against code is a model of what the designs say; the day an
implementation departs from a decision record, this document is wrong about that
surface and nothing in this tree notices. What holds it is a reader, and the
review this document is supposed to get on every release is #128, which does not
exist. That is the one condition of #105 this document cannot meet by being
written.

## The assets, ranked

Ranked by what an operator loses, worst first.

**1. The customer records and their history.** The people, the organisations,
the deals, the activities and the change log that says how each of them got to be
what it is, which `docs/model.md` and `docs/decisions/0004-change-log.md` define.
This is the asset the product exists to hold, and it is worse than a list of
contacts: the history says who was talking to whom, about what, at what value,
and when a relationship went wrong. A copy of it is a copy of the operator's
commercial position and, for the people in it, personal data they gave to
somebody else.

**2. The mailbox and connector credentials.** A mailbox credential is not a
credential for this instance, it is a credential for the operator's mail system,
and it is the one asset here whose theft gives an attacker something outside this
software entirely. `docs/decisions/0011-connectors.md` scopes one credential to
one connection so that revoking a connection under #88 revokes something rather
than everything, and #91 is where the handling is decided.

**3. The credentials and sessions of the instance's own users.** A working
session for a manager is read access to the whole first asset, and a working
session for an administrator is more than that. #26 is where callers are
authenticated.

**4. The instance's own secrets.** The database credential, the key that
encrypts stored connector credentials at rest, any signing key. Each of these is
a route to one of the three above rather than an asset in itself, which is why it
sits below them and not at the top. #91 holds them and #90 holds the
configuration they arrive in.

**5. The audit trail.** #96 keeps a record of who saw and changed what. It is
below the records it describes and above availability, because an attacker who
can edit it can make the first four losses unprovable.

**6. Availability.** A sales team that cannot reach its pipeline loses a day. It
is last deliberately: this ranking is about what cannot be undone, and an outage
is the one entry that can be.

Two things are deliberately not on that list. The source is public, so it is not
an asset to be stolen. And the operator's own infrastructure is theirs, which
`SECURITY.md` already puts out of scope.

## The actors

Five, and each one is defined by what they already hold rather than by intent,
because intent is not a thing a control can read.

**An unauthenticated attacker on the network.** Holds nothing but reachability.
Their routes are the ones that answer before authentication: the version
negotiation of the sync protocol, the inbound trigger route, the login route,
and whatever the reverse proxy passes through. Their best outcome is a session
they did not earn or an unauthenticated read.

**An authenticated user of the instance.** Holds a valid session with ordinary
permissions, and is the actor this product's permission model is mostly about. A
sales team is exactly the setting where one person must not see another's
pipeline. Their routes are every route, and their best outcome is a record, a
report or a workflow that returns what their permissions do not allow.

**An administrator.** Holds the configuration, the workflow definitions, the
connector configuration and, in most deployments of a self hosted product, the
host and the database as well. Almost nothing in this software can stop an
administrator, and `SECURITY.md` says that an actor exercising the permission
they hold is not a finding. What this model asks about them is different and is
worth asking: whether their actions are recorded where somebody else can read
them, and whether a mistake they make is recoverable.

**A compromised connector credential.** Not a person: a credential this instance
holds for somebody else's service, now in an attacker's hands, or the service at
the far end behaving badly. The routes are inward, meaning what a connector
brings in and writes, and outward, meaning what this instance sends to an
address that is no longer what the operator thought it was.

**Somebody holding a stolen phone.** Holds a device with a local store, a queue
of unsent changes and, depending on what the platform gives, a credential. This
is the actor the mobile milestone adds and the one an operator will ask about
first, because it is the only one they can picture.

Two actors are not carried here and the omission is deliberate. The operator's
hosting provider and anybody with the database console are outside what this
software can refuse, and `docs/decisions/0004-change-log.md` already names the
console as the one route no code here can close. And this project's own supply
chain is a different model on a different subject, held by the gates the
contributor guide describes rather than by this document.

## The surfaces

Seven, taken in the order an attacker meets them. Each says what the attack is,
what stops it, and where that is enforced, and where the answer is nothing, it
says nothing and names the issue that owes the control.

### The API

The route everything else is built on. The attacks are the ordinary ones and
they are ordinary because they work: an unauthenticated call that is answered, a
call authenticated as one user that returns another user's records, a filter or a
sort that reaches a field the caller may not read, a query whose cost is unbounded,
and a write replayed until it has happened four times.

What stops them, by design. `docs/decisions/0006-api-shape.md` closes the query
grammar, so a caller composes nothing: filtering is on fields the description
marks filterable, sorting on at most two keys, aggregation over fields of bounded
cardinality, and everything else is refused by name. That record states why the
grammar is closed rather than open, and the reason is exactly this model's: an
open expression language over a live database is a denial of service surface and
a permission bypass surface at the same time, and a repair for either half tends
to reopen the other. #27 requires every read path to take the caller's
permissions as an argument the route cannot forget, #33 makes every unsafe
request idempotent, #32 holds the rate and size limits, and #26 authenticates.

Where it is enforced: nowhere. `server/crates/api/src/lib.rs` holds a licence
line and a doc comment saying the module is empty and that M3 fills it. Every
sentence above is a design in a record or a done condition on an open issue, and
the issues are #26, #27, #29, #32 and #33.

### The inbound trigger route

The route an attacker reaches first among the ones that exist to be reached from
outside, because it is the one a form on a website posts to. The attacks are an
unauthenticated start, a payload shaped to break the parser, a payload large
enough to be the attack on its own, a retry storm, and a trigger the operator
disabled that goes on being accepted.

What stops them, by design. #53 gives an inbound trigger its own credential, its
own rate limit and its own payload size limit, each separately configurable with
a safe default; validates the payload against a declared shape before any step
runs and stores the rejection rather than dropping it; makes the route idempotent
by the same mechanism as the rest of the API; and makes disabling take effect
immediately with a refusal to the sender rather than a silent discard.

Where it is enforced: nowhere. #53 is open and blocked on #21, #31, #32, #33 and
#50. This surface is the one where the gap between design and code costs most,
because it is the only route in this plan that a stranger is meant to be able to
reach.

### The outbound workflow action

Treated at length in its own section below, because a workflow action that
fetches an address somebody supplied is the classic route from a public
application into an operator's internal network.

### The sync protocol

The route the mobile clients use and, by design, the route somebody else's client
may use too. `docs/sync-protocol.md` is written to be copied from. The attacks
are a session taken over or continued after it should have ended, a client fed
another instance's data, a change replayed until it has applied twice, a cursor
forged to read a window the caller may not see, a subset request large enough to
be a denial of service, and an edit lost quietly enough that nobody notices.

What stops them, by design. The protocol carries the requirements it is written
in terms of, and the ones that bear on this model are that a client which
receives an instance other than the one it holds stops and does not merge, that
every message after the handshake carries the session identifier, that a change
carries a key stable across retries and a server which has seen a key answers
with the outcome it gave the first time, that a cursor this instance did not
issue is refused as unreadable rather than as too old, that a subset request
larger than the instance will serve is refused rather than trimmed, and that no
message shape lets a server report that a queued change was discarded. The
authentication that a session begins after is #26 and is deliberately not in that
document. The rule that decides what happens when two people edited one field is
`docs/conflict-model.md`, and the promise it carries is the one an operator
quotes.

Where it is enforced: nowhere, and the distance is worth stating precisely.
`sync-example-scan.js` refuses an example in that document which disagrees with
its own table, so what is enforced today is that the specification is internally
consistent. Nothing in this tree speaks the protocol, #74 is the conformance
suite every client would have to pass, and #72 is the specification's own issue.
A green run of the example scan says the document agrees with itself and nothing
about any implementation.

### The reporting query path

The surface this project is most likely to get wrong, because it is the one where
a permission failure does not look like one. A report that aggregates records the
caller may not see is a data leak wearing a chart, and the number it returns is
believed precisely because a chart is not where anybody looks for a leak.

What stops it, by design. `docs/decisions/0007-reporting.md` states that the
record set behind every number is the permission filtered set for the caller, and
that it is the same filtering as a record read rather than a second
implementation of it. #21 requires the evaluation to happen in one place that the
API, the reporting engine and the workflow engine all call, and its own done
conditions say an aggregate over records the caller may not see either excludes
them or refuses, and that which of the two it does is a documented choice rather
than an accident of the query. #44 makes every number able to print the query
that produced it, which is what makes the claim checkable rather than asserted.

Where it is enforced: nowhere, and the reporting record says so in its own words,
that none of its three rules is enforced by anything in this repository today.
The issues are #21, #27, #36 and #44.

### The import path

The route by which an attacker's bytes become the operator's records. The attacks
are a file that breaks the parser, a file that becomes a formula when somebody
opens the export of it in a spreadsheet, a value long enough to truncate into
something else, a date read in the wrong order so that a whole pipeline shifts by
a month, and an import that half applies and leaves nobody able to say which half.

What stops them, by design. #23 makes an import two passes, a validation pass
that touches nothing and reports every row it would reject with the reason, and
an apply pass that runs only on what the first accepted; makes date and number
parsing explicit rather than guessed; makes deduplication use the identity rule
of `docs/model.md` rather than a heuristic; makes an import one unit in the change
log so that everything it created can be found and reversed; and asks for
fixtures for the hostile cases by name, including the leading character a
spreadsheet treats as a formula.

Where it is enforced: nowhere. #23 is open. One thing that supports it does exist
and is worth naming, because it is the difference between a hostile fixture that
proves something and one that does not: `.gitattributes` decides line endings for
every tracked path and `text-scan` refuses a carriage return in tracked text, so
a fixture whose point is its bytes cannot be silently normalised on its way into
git.

### The web client

The surface a user meets, and the one where a compromise is a compromise of
every user rather than of a server. The attacks are a script that reaches the
page and runs with the session, a page that loads code or a font from an address
the operator did not choose, a token stored where another script can read it, and
a link that makes an authenticated browser act.

What stops them, by design. `docs/decisions/0009-web-client.md` is where the
client means and its boundaries are decided, and #97 asks that the client load no
font, script, style or image from any address other than the instance it was
served from, asserted by a test that fails on any external request.

Where it is enforced: nowhere, and the reason is that there is nothing to
enforce it against. The workspace holds no document, no stylesheet and no font,
which is a fact about the tree rather than a control. What does exist is the
client's test boundary, which refuses a test that reaches past the test process,
and that is a rule about the suite rather than about what a page fetches. The
issues are #63, #65 and #97.

## The outbound request surface, at length

This is the surface an attacker reaches for once they have any foothold at all,
and it is the one worth the most text, because the failure is not a bug in a
parser but a feature working exactly as written against an address nobody
checked.

The shape here is narrower than the classic one and the narrowing is a decision
rather than luck. `docs/decisions/0008-workflow.md` closes the action set, so
there is no action that runs arbitrary code, and its reasoning is this model's:
an operator who can create a workflow would otherwise be an operator who can
execute code on the host, which is a privilege escalation surface with a friendly
interface on it. The expression language of #55 is the bounded exception and is
bounded in the one direction that matters, that it computes over the record and
the trigger entry and may not call out. `docs/decisions/0011-connectors.md` says a
connector reaches the service named in its configuration and nothing else, not a
second address for a lookup, an update endpoint or a diagnostic report.

So the set of places this software can be made to send a request from is
enumerable, which is the difference between a model that can be complete about
this surface and one that cannot. It is not the same as the set being small, and
it is not the same as the addresses in it being safe, because every one of them is
a value somebody typed into a configuration form.

What an attacker wants from it, in the order they would try. Reach an address on
the operator's internal network that has no authentication because it was never
meant to be reachable, which is most internal services. Reach a cloud instance
metadata address and read a credential out of it, which is the single highest
value target on most hosts and needs no authentication at all by design. Turn the
instance into a scanner, using the difference between a refusal and a timeout to
map a network from outside it. Make the instance fetch something enormous, or
something that never finishes, and hold a worker while it does. And read the
response, because an action that reports what it fetched is an oracle.

What has to be true for that surface to be safe, stated as requirements against
whoever builds it rather than as a description of anything that exists.

The address is resolved once and the connection is made to the address that
resolution returned. A check that resolves a name, approves it, and then opens a
connection by name again is the rebinding failure, and it is the one that gets
written by somebody who did check.

The resolved address is refused where it is loopback, link local, a private
range, a unique local address, or any of the ranges reserved for something other
than the public internet. Both address families, because a deployment that
refuses only one is a deployment where the other is the route.

A redirect is a new address and takes the whole check again, or redirects are not
followed at all. A chain of redirects that ends inside the network is the same
attack with one more hop in it.

The scheme is the one the action declares and nothing else. A credential in the
address is refused rather than sent.

The response is bounded in bytes and the request is bounded in time, and both
bounds are the instance's rather than the caller's.

The identity the request runs under is the workflow's own, which
`docs/decisions/0008-workflow.md` requires an operator to choose at publication
and requires the publisher to hold every permission they give it. That is what
stops the outbound surface being reachable by anybody who can cause a change.

Where it is enforced: nowhere. Not one of the seven requirements above is
implemented, and #54 is the action set, #55 the expression language, #58 the
runaway guard and #91 the credential handling. One neighbouring thing is enforced
and it is a property of the source rather than a runtime control, so it is worth
being exact about what it does and does not buy: `test-scan` refuses library code
under a crate's `src/` that opens an outbound connection where the crate is not a
connector, and `prove-headless` is what says that rule bites. That keeps the
outbound reach inside the module this model can reason about. It says nothing at
all about which address that module then connects to, which is the whole of the
attack described above.

One more property of this surface, recorded because it is the part an operator
asks about and the part this model cannot answer alone. Whether any feature may
send data to a service outside the host at all is an open question in #129, and
until it is answered a document stating that nothing does would be a sentence
somebody later has to soften. `docs/decisions/0011-connectors.md` already answers
the narrower half, that no connector sends customer data to a service this
project chose, and that half is settled.

## The assumptions this model rests on

Stated so that a reader can attack the assumptions rather than only the design.

**About the deployment.** The instance runs behind something that terminates
transport security and does not itself trust what it forwards, which is #98. The
operator holds the host, the database and the backups. There is one operator per
instance: nothing in this plan is multi-tenant, and no control here is written to
separate two customers who share a process. The operator is often one busy person
holding every role at once, which is the deployment the closed action set of
`docs/decisions/0008-workflow.md` was chosen for.

**About the database.** It is trusted, and anybody who reaches it directly is
outside everything this software refuses. That is not a comfortable assumption
and it is the honest one: `docs/decisions/0004-change-log.md` names the SQL
console as the one of four write routes that no code can refuse, and answers it
with the audit trail of #96 and the operator's own controls.

**About the records.** Everything in them is personal data about somebody who is
not the operator's employee, and much of it was given to a third party rather
than to this operator. That is what puts the first asset where it is and it is
what makes the erasure obligation of #101 a product feature rather than a
document.

**About this tree.** The designs in `docs/decisions/` are what the code will do.
Where an implementation departs from a record, this model is wrong about that
surface and no reading of this tree will say so.

**About what a green run means.** Every gate the contributor guide describes
judges the source or the specification. None of them is a runtime control, none
of them tests a request, and a fully green pull request says nothing about
whether any surface above resists anything. Which gates exist is printed by the
commands `CONTRIBUTING.md` names rather than listed here, for the reason that a
list in a document drifts against the thing it describes.

## What this model does not do

It does not rank the risks. A ranking needs a likelihood and an impact per
scenario, and a likelihood over an implementation that does not exist would be a
number invented to fill a column. The assets are ranked because their order is a
judgement about the operator's loss and does not depend on any code; the
scenarios are not, and they get an order the day there is something to measure.

It does not carry the mobile clients' platform. Which platform ships first is an
open entry in #129, `docs/decisions/0010-mobile.md` states that nothing about the
protocol depends on it, and the stolen phone actor above is written so that it
does not either.

It does not decide whether any feature may send data outside the host. That is
the third entry in #129 and it belongs to the person who owns that decision.

It is not the data protection statement. `docs/data-protection.md` under #100 is
where the operator facing claim about customer data staying on the host is made,
and this document is where the routes it would leave by are enumerated.

## What refuses a departure from this

Nothing, and there is no mechanism to name. `docs-scan` judges that this
document's links and its references to other documents resolve, that the words
are the ones the register names, and that the file is UTF-8 with no carriage
return in it. None of that reads a sentence, and no reading of this tree decides
whether a surface named here is still the surface, whether a control claimed as
absent has since arrived, or whether one claimed as present has gone.

What was supposed to hold it is a review on every release, and that is the
seventh condition of #105: the model is reviewed against the release checklist
rather than written once and forgotten, and the checklist item names it. The
checklist is #128 and it does not exist, so this document lands with that
condition unmet and #105 stays open on it.
