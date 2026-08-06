# 0006. The API shape and how it is allowed to change

Status: accepted
Issue: #25
Supersedes: nothing
Superseded by: nothing

## The decision

A resource oriented HTTP API, with a machine readable description generated from
the server's own metadata layer and served by the instance itself, plus a small
fixed core that never changes shape.

Filtering, sorting and aggregation get a stated query grammar rather than an open
expression language.

This is the most public thing the project produces. The web client, the mobile
clients, the outbound actions of the workflow engine and every integrator speak
to it, so its shape and its compatibility promise are decided together and before
an endpoint exists.

## What makes this harder than the usual case

Objects and fields are defined by the operator while the instance is running,
which is the decision `docs/decisions/0003-custom-fields.md` records under #14.
A contract fully known at compile time therefore cannot describe an instance, and
a client generated against one instance is not automatically valid against
another.

Every part of what follows exists to answer that. The fixed core is what a client
can rely on before it has read anything. The description is how it learns the
rest. The versioning rule covers both, and they are versioned differently because
they change for different reasons.

## The small fixed core

The core is the part that is the same on every instance running a given major
version, and it is what a client depends on before discovery. It contains, and
is intended to stay at, five things.

The instance document, which names the major versions this instance serves, the
address of the description document, the current description version, and the
deprecation date of any version being retired.

The description document itself, described below.

Authentication, meaning the token exchange of #26. A client that cannot
authenticate cannot read the description, so this cannot live behind discovery.

The error document shape, described below, including the stable code vocabulary
for the errors the core itself can return.

The capability document, which states the limits in force on this instance: the
maximum page size, the maximum filter depth, the maximum aggregation group
count, and the request size and rate limits of #32. A client reads its limits
rather than discovering them by being refused.

Health and readiness under #94 are adjacent to the core and are deliberately not
in it, because they are for the operator rather than for a client and they answer
before authentication.

Everything else, meaning every object, every field and every endpoint that
touches a record, is generated from the metadata layer and is therefore
instance shaped.

## How a client discovers the instance

The client fetches the description document. It is generated from the metadata
layer, so it describes the objects and fields this instance actually has,
including the operator's custom ones, and it carries for each field the type, the
validation rules, and three flags that the query grammar below depends on:
whether the field may be filtered, whether it may be sorted, and whether it may
be used as an aggregation group key.

The description carries a version. It is an opaque identifier and it changes
whenever the metadata layer changes, which is whenever an operator adds, removes
or retypes a field.

## What happens when the schema moves under a client

A client sends the description version it holds with every request. The server
compares it to the current one and behaves differently for the two kinds of
difference, which is the whole point of comparing rather than ignoring.

Where the difference is additive, meaning the current description contains
everything the client's version contained, the request is served. The response
carries the current version, so the client learns it is behind on the next
successful call rather than on a failure.

Where the difference removes or narrows something the request names, meaning a
field the request filters on, sorts by, writes to or asks for, the request is
refused. The error names the field, the change, and the current version. The
server does not guess, does not silently drop the clause, and does not fall back
to a wider reading, because all three produce a response that looks successful
and answers a different question than the one asked.

A request that names nothing outside the fixed core is unaffected by either case.
That is what the fixed core buys, and it is why a client written against the core
alone keeps working through operator schema changes it knows nothing about.

A client that sends no description version is treated as holding the current one.
That is a convenience for a person with a command line, and it is stated here so
that nobody reads the absence of the header as a stronger promise than it is.

## The versioning rule

The fixed core carries a major version in the path. It changes only for a
breaking change to the core itself, which is expected to be rare and is expected
to be argued in a decision record of its own rather than in a release note.

The generated part is versioned by the description version, which is per instance
and is not a number anyone releases. An operator adding a field changes it, and
that is not a breaking change to anything, because the compatibility promise is
about what this project ships and not about what an operator does to their own
instance.

A breaking change to the core is any of: removing an endpoint or a response
field; narrowing a type or a value range; making an optional request field
required; adding a new required request field; changing what an error code means;
changing the default ordering or the pagination semantics; and tightening a
published limit. Adding a value to an enumeration is also breaking, unless the
enumeration was declared open in the description, which is why every enumeration
that is expected to grow is declared open from the first release and clients are
required to handle a value they do not know.

Not breaking: adding an endpoint, adding an optional request field, adding a
response field, relaxing a limit, adding a value to an enumeration already
declared open, and any change to the human readable text of an error.

A major version is supported for at least twenty four months after its successor
becomes the default on a release of this project. The instance document names the
deprecation date of a version being retired, so the promise is readable from the
instance rather than from a document somebody has to find.

## The error model

An error is an error at the transport level. It carries an HTTP status code in
the 4xx or 5xx range, and it is never a 200 response containing a failure.

This is stated as a rule rather than as a preference because the alternative
below gets it wrong by construction, and because every piece of monitoring an
operator already owns understands a status code and none of it understands a
convention invented here.

The body is one document with a stable machine readable `code`, a human readable
message that is not part of the compatibility promise, and where the failure is
per field, a list naming the object, the field and the rule that refused it. The
code vocabulary is part of the fixed core, and adding a code is not a breaking
change while changing what one means is.

Bulk operations are the case that tempts an exception, and #31 refuses it there
too: a bulk request that half succeeded does not return 200 with the failures
inside. It has its own status and its own document naming exactly what landed and
what did not.

## The query grammar and its limits

The grammar is closed. Everything it accepts is listed in the description, and a
client can tell in advance what will be refused instead of discovering it by
being refused.

Filtering is allowed on any field the description marks filterable, with the
operators equal, not equal, less than, greater than, the two inclusive forms, one
of a set, is null, and prefix match on text. Clauses combine with and and or to a
stated maximum depth, published in the capability document.

Sorting is allowed on any field the description marks sortable, with at most two
keys, and the record identifier is always appended as a final key so that the
order is total and pagination cannot skip or repeat a row.

Aggregation is count, sum, minimum, maximum and mean over a numeric field,
grouped by at most two fields the description marks groupable, which are the
fields of bounded cardinality.

What is refused, and the reason each one is refused rather than merely
unimplemented: substring and regular expression matching on unindexed text,
because it is a sequential scan an unauthenticated cost model cannot bound; joins
beyond one declared relationship hop, for the same reason and because the
permission model of #27 must be evaluable on the result; grouping by a field of
unbounded cardinality, because the result set is the table; and any expression a
caller composes themselves, which is the case that closes the grammar in the
first place.

Every refusal names the rule that refused it and the field it refused, so the
message tells the caller what to change.

The reason the grammar is closed rather than open is that an open expression
language over a live database is a denial of service surface and a permission
bypass surface at the same time. It is one surface, not two, because the same
expression that is expensive is usually also the one that reaches a row the
caller may not see, and a fix for either half tends to reopen the other.

## The alternative considered

A single graph endpoint where the client asks for exactly the fields it wants. It
suits a run time schema better than anything else does, and it is what the most
modern open CRM chose.

What it costs. A query cost problem that has to be solved on day one rather than
deferred, because the caller composes the shape and the server has to bound it
before running it. A harder story for caching, since one endpoint and one method
defeats every layer between the client and the server that would otherwise help.
A harder story for rate limiting, since request count stops correlating with
work done. And an error model that reports failure inside a successful response,
which every monitoring tool an operator already runs then has to be taught about,
one tool at a time.

What the decision gives up by not taking it is real: round trips on a client that
needs a little of a lot of records, and a client that must ask for a whole
resource to read three fields of it.

That is mitigated rather than solved, and the mitigation is named here so it is
not mistaken for the reversal condition being met. A request may name a subset of
the declared fields of a resource, checked against the description like every
other field reference. That is bounded field selection over a closed vocabulary,
which costs nothing the grammar above does not already cost, and it is not an
open query language.

## What would reverse this

Reverse it if the mobile clients turn out to need field selection badly enough
that the resource shape forces round trips they cannot afford, measured against
the budget #81 sets rather than argued from preference. Bounded field selection
is the cheaper answer and is expected to be tried first; the reversal is what is
left if it is not enough.

Reverse it before the sync protocol of #72 is specified rather than after, since
the protocol and the API are the two things a client depends on and moving one
after the other has landed is the expensive order.
