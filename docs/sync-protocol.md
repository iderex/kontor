# The sync protocol

This is what a client and an instance say to each other. It is written before any
client exists, so that a client is something written against a document rather
than reverse engineered from whichever one was built first.

`docs/decisions/0010-mobile.md` puts the whole weight of its shell decision here:
the shell is reversible only if the protocol is the agreed part, and a shell
built before the protocol would have fixed the protocol by accident at whatever
shape that first client happened to need. So this document is the permanent
artefact and a client is the replaceable one.

Nothing here is new where something else already decided it.
`docs/decisions/0006-api-shape.md` decides the transport, the error model, the
description document and the versioning rule, and this protocol is a use of that
core rather than a second one beside it. `docs/decisions/0004-change-log.md`
decides the log every cursor here points into. `docs/decisions/0010-mobile.md`
decides what a client may do offline and what the synced set is. Where one of
those and this document disagree, the record is right and this is the defect.

## What this document is

A specification of eight messages, the order they are sent in, and the exact
meaning of every field in each one. Every message shape below carries a table of
its fields and one example, and the examples are read by a machine rather than by
a reader alone: `sync-example-scan.js` refuses an example that has a field its
shape does not declare, omits one its shape requires, or names a type that is not
the shape it sits under. `prove-sync-examples` is the evidence that each of those
refusals bites.

## What this document is not

It is not an implementation and it does not describe one. Nothing in this tree
speaks any of it today:

    git grep -l 'sync' -- server/crates ; echo "exit=$?"
    exit=1

It is not the conformance suite either. A specification is prose until something
refuses a client that does not follow it, and #74 is where that refusal is built.
The requirements at the end of this document are numbered so that suite can map a
case to each one and refuse a requirement with no case.

It does not decide the conflict resolution rule. #73 does, and this protocol
carries the outcome of that rule across the wire without choosing it. What is
already fixed, by `docs/decisions/0010-mobile.md`, is that nothing in a client's
queue is discarded to resolve a conflict, and every shape below is built so that
discarding one is not expressible.

## Vocabulary

**Instance.** One deployment, holding one database. An instance has an opaque
identifier that never changes and that every message quotes, so a client that has
been pointed at a second instance finds out on the first message rather than by
merging two databases.

**Client.** Anything that speaks this protocol. The reference core of #75 is one.
A client somebody else writes is another, and this document exists so that the
second one is possible without asking anyone.

**Synced set.** The records a client holds, defined in
`docs/decisions/0010-mobile.md`: the records the user owns or follows, plus the
records they touched inside a recency window, under a cap on rows and bytes. A
phone does not hold a company's database.

**Subset request.** What a client asks its synced set to be. The instance answers
with what it will serve, which may be less.

**Cursor.** A point in the change log. It is an opaque string to a client and it
is derived rather than invented: `docs/decisions/0004-change-log.md` gives every
entry a monotonic identifier precisely so that a total order exists, and a cursor
names the last entry a client has applied.

**Session.** One sync, from `hello` to the last message either side sends. A
session identifier is quoted in every later message so that a server log and an
audit entry can be joined without either holding a record identifier.

**Change.** One field of one record moving from one value to another, made on the
client while it had no connection. The granularity is the log's, not the record's,
for the reason that record decides: a change touching three fields is three
entries sharing one transaction.

## The transport and the encoding

HTTPS, and nothing else. A client that cannot verify the certificate does not
sync, and there is no setting that turns that off, because the one deployment
where somebody would want it is the one where the traffic is a copy of a customer
database crossing a network the operator does not own.

One message per request body, one message per response body, both UTF-8 JSON. The
core's major version is in the path, which is `docs/decisions/0006-api-shape.md`'s
rule rather than this document's.

A refusal is a refusal at the transport level, carrying a 4xx or 5xx status and
the core's own error document. That is the same record's rule and this protocol
takes no exception to it: there is no 200 response here carrying a failure
inside. The `refusal` shape below is that error document with the code vocabulary
this protocol adds, not a second error mechanism.

Numbers are JSON numbers and money is not one of them. An amount crosses as the
pair `docs/decisions/0005-money-and-time.md` fixes, an integer count of minor
units with its currency, because a decimal on the wire is a currency exponent
decided twice.

The JSON value `null` and an absent field are different, and the difference is
the same one the change log draws: absent means the field was not part of this
message, and `null` means the field is present and empty. A client that collapses
the two writes a history of an optional field that cannot be read back.

## Version negotiation

A client offers every protocol version it speaks, most preferred first. The
server answers with the one it chose, and that version governs the whole session.
A server that speaks none of the offered versions refuses with `protocol-unknown`
and names what it does speak, so the client's next move is a fact rather than a
guess.

The protocol version is not the description version and the two never move
together. The protocol version is what this project releases. The description
version is what an operator's own schema changes produce, which is
`docs/decisions/0006-api-shape.md`'s rule, and an operator adding a field is not
a protocol change.

A value a client does not know is not an error unless this document says it is.
Every enumeration here is open except `type`, and a client that meets an unknown
value in an open enumeration keeps the record, marks it as carrying something it
cannot render, and does not drop it. Dropping is the failure the whole plan is
written against and it is stated here as well as in the record.

## Message shapes

Eight, and the set is closed. Each carries `type`, which is the one enumeration
here that is not open: a client meeting a `type` it does not know refuses the
message rather than guessing at it.

### `hello`

Sent by the client to open a session. It is the only message that carries no
session identifier, because it is what produces one.

| field | type | required | meaning |
| --- | --- | --- | --- |
| `type` | string | yes | The literal `hello`. |
| `protocol` | array | yes | The protocol versions this client speaks, most preferred first, as strings. |
| `client` | object | yes | What the client is: `name` and `build`, both strings, for the connection list an operator reads. |
| `schema` | string | no | The description version the client holds. Absent on a client that holds none, which is a client that has never synced. |
| `cursor` | string | no | The point in the change log this client has applied. Absent asks for a full sync. |
| `subset` | object | yes | The subset request, whose shape is below. |

```json
{
  "type": "hello",
  "protocol": ["1"],
  "client": { "name": "reference-core", "build": "0.0.0" },
  "schema": "d41f2c",
  "cursor": "c:18422",
  "subset": { "owned": true, "followed": true, "touched_within_days": 90 }
}
```

### `welcome`

Sent by the server in answer to `hello`. It is where a client learns whether it
is being served incrementally or from the beginning, and it says so before any
record crosses, so a client can decide what to do with what it already holds
rather than discovering it halfway through.

| field | type | required | meaning |
| --- | --- | --- | --- |
| `type` | string | yes | The literal `welcome`. |
| `session` | string | yes | Identifies this sync in every later message and in the audit trail. |
| `protocol` | string | yes | The version chosen out of what the client offered. |
| `instance` | string | yes | The opaque identifier of this instance. A client that holds a different one stops. |
| `schema` | string | yes | The current description version. |
| `address` | string | yes | Where the description document is fetched, which is the fixed core's route rather than anything this protocol serves. |
| `serves` | string | yes | `incremental` or `full`. Open enumeration. |
| `reason` | string | no | Why `serves` is `full` where the client asked for incremental. Absent where it is not. |
| `bounds` | object | yes | The limits in force for this session, whose shape is below. |

```json
{
  "type": "welcome",
  "session": "s:9c1e",
  "protocol": "1",
  "instance": "i:7f0a",
  "schema": "d41f2c",
  "address": "/v1/description",
  "serves": "incremental",
  "bounds": { "page": 500, "queued_changes": 5000, "subset_records": 20000 }
}
```

### `pull`

Sent by the client to ask for the next page. The same message serves a full sync
and an incremental one, and which of the two is happening was settled by
`welcome` rather than by this message.

| field | type | required | meaning |
| --- | --- | --- | --- |
| `type` | string | yes | The literal `pull`. |
| `session` | string | yes | The session from `welcome`. |
| `cursor` | string | no | The cursor from the last `page`. Absent asks for the first page. |
| `max` | number | yes | How many records the client wants, which the server may lower to its own bound and never raises. |

```json
{
  "type": "pull",
  "session": "s:9c1e",
  "cursor": "c:18422",
  "max": 500
}
```

### `page`

Sent by the server in answer to `pull`. It carries records, the records that have
left the synced set, and the cursor to send back next.

| field | type | required | meaning |
| --- | --- | --- | --- |
| `type` | string | yes | The literal `page`. |
| `session` | string | yes | The session from `welcome`. |
| `schema` | string | yes | The description version these records were read at. |
| `covers` | string | yes | `snapshot` where this page is part of a full sync, `changes` where it is part of an incremental one. Open enumeration. |
| `cursor` | string | yes | The point to send on the next `pull`. Stored by the client before it applies anything, which is what makes a resume possible. |
| `more` | boolean | yes | Whether another page follows. |
| `records` | array | yes | The records, each one whole rather than as a set of field changes. |
| `gone` | array | yes | The identifiers of records that have left the synced set or been deleted, with which of the two for each. Empty rather than absent where none has. |

```json
{
  "type": "page",
  "session": "s:9c1e",
  "schema": "d41f2c",
  "covers": "changes",
  "cursor": "c:18630",
  "more": true,
  "records": [{ "object": "deal", "id": "r:41ab", "fields": { "stage": "qualification" } }],
  "gone": [{ "object": "deal", "id": "r:2200", "why": "left-the-subset" }]
}
```

### `push`

Sent by the client to send what it did while it had no connection. The changes
are in the order they were made, and the server applies them in that order, which
is what stops a field being set and then unset arriving the other way round.

| field | type | required | meaning |
| --- | --- | --- | --- |
| `type` | string | yes | The literal `push`. |
| `session` | string | yes | The session from `welcome`. |
| `schema` | string | yes | The description version the client made these changes against. |
| `changes` | array | yes | The queued changes, oldest first, each one shaped as `change` below. |

```json
{
  "type": "push",
  "session": "s:9c1e",
  "schema": "d41f2c",
  "changes": [
    {
      "key": "k:8ac1",
      "object": "deal",
      "record": "r:41ab",
      "field": "stage",
      "value": "negotiation",
      "made_at": "2026-08-09T14:02:11Z",
      "base": "c:18422"
    }
  ]
}
```

### `receipt`

Sent by the server in answer to `push`, with one outcome per change, in the order
the changes arrived. A change with no outcome is not expressible, because a
client that could not tell whether a change landed would have to guess, and the
guess is where a change is lost.

| field | type | required | meaning |
| --- | --- | --- | --- |
| `type` | string | yes | The literal `receipt`. |
| `session` | string | yes | The session from `welcome`. |
| `cursor` | string | yes | The point in the log after these changes were applied. |
| `outcomes` | array | yes | One `outcome` per change, in the order the changes were sent. |

```json
{
  "type": "receipt",
  "session": "s:9c1e",
  "cursor": "c:18700",
  "outcomes": [{ "key": "k:8ac1", "state": "applied", "detail": null }]
}
```

### `subset`

Sent by the client to change what it holds without starting again. The server
answers with a `page` whose `covers` is `snapshot` for what has newly entered the
set, and whose `gone` names what has left it.

| field | type | required | meaning |
| --- | --- | --- | --- |
| `type` | string | yes | The literal `subset`. |
| `session` | string | yes | The session from `welcome`. |
| `request` | object | yes | The new subset request, shaped as `subset request` below. |

```json
{
  "type": "subset",
  "session": "s:9c1e",
  "request": { "owned": true, "followed": true, "touched_within_days": 30 }
}
```

### `refusal`

The core's error document, carried under a status in the 4xx or 5xx range. It is
listed here as a message shape because a client has to handle it in every state,
and because the code vocabulary below is what a client branches on.

| field | type | required | meaning |
| --- | --- | --- | --- |
| `type` | string | yes | The literal `refusal`. |
| `code` | string | yes | The stable code. Open enumeration, and the values this protocol adds are listed below. |
| `message` | string | yes | Human readable, and not part of the compatibility promise. |
| `session` | string | no | The session, where the refusal happened inside one. |
| `detail` | object | no | What the code needs to be acted on: the field a schema change removed, the versions a server speaks, the cursor a client should restart from. |

```json
{
  "type": "refusal",
  "code": "cursor-too-old",
  "message": "the change log no longer reaches that point",
  "session": "s:9c1e",
  "detail": { "restart": "full" }
}
```

The codes this protocol adds to the core vocabulary are `protocol-unknown`,
`instance-mismatch`, `session-unknown`, `cursor-too-old`, `cursor-unreadable`,
`schema-moved`, `subset-too-large`, `queue-too-large` and `change-unreadable`.
Each one is named in the section that produces it, and none of them is a
condition a client can retry its way out of without doing something different.

## The shapes inside a message

Four, and they carry no `type` of their own because they never travel alone. That
is a rule rather than a description: `sync-example-scan.js` refuses a shape under
this heading that declares a `type`, and one under the heading above that does
not, because the two sets are handled differently by every client and a shape that
drifted between them would be handled by neither.

### `change`

One field of one record moving, made on a client with no connection. Its shape
follows `docs/decisions/0004-change-log.md` rather than inventing a second one.

| field | type | required | meaning |
| --- | --- | --- | --- |
| `key` | string | yes | The idempotency key, chosen by the client and stable across retries, so a lost answer does not apply the change twice. |
| `object` | string | yes | Which object type the record belongs to. |
| `record` | string | yes | Which record. |
| `field` | string | yes | Which field. |
| `value` | any | yes | The value after. `null` is the field present and empty, which is not the same as the field being absent from the message. |
| `made_at` | string | yes | When the client made it, in UTC. It is what a person reads and it never decides an order, because two clients disagree about the time. |
| `base` | string | yes | The cursor the client held when it made this change, which is what lets the server say whether anything moved underneath it. |

```json
{
  "key": "k:8ac1",
  "object": "deal",
  "record": "r:41ab",
  "field": "stage",
  "value": "negotiation",
  "made_at": "2026-08-09T14:02:11Z",
  "base": "c:18422"
}
```

### `outcome`

What happened to one pushed change.

| field | type | required | meaning |
| --- | --- | --- | --- |
| `key` | string | yes | The key of the change this answers. |
| `state` | string | yes | `applied`, `conflicted`, `refused` or `superseded`. Open enumeration. |
| `detail` | object | no | For `conflicted`, what the server holds and the entry that moved it. For `refused`, the code and the field. Absent where the state needs nothing. |

```json
{
  "key": "k:8ac1",
  "state": "conflicted",
  "detail": { "held": "closed-won", "moved_at": "c:18500" }
}
```

A `conflicted` outcome is not a discard and the client does not treat it as one.
The change stays in the client's queue until #73's rule has been applied to it and
the result is either a new change or a decision a person made. That is the promise
`docs/decisions/0010-mobile.md` carries and this shape is built so that losing it
requires writing a client that ignores the state it was sent.

### `subset request`

What a client asks its synced set to be.

| field | type | required | meaning |
| --- | --- | --- | --- |
| `owned` | boolean | yes | Whether records the user owns are included. |
| `followed` | boolean | yes | Whether records the user follows are included. |
| `touched_within_days` | number | yes | The recency window for records the user touched but neither owns nor follows. Zero asks for none of them. |

```json
{
  "owned": true,
  "followed": true,
  "touched_within_days": 90
}
```

The cap on rows and bytes is not in this shape, because it is the instance's
rather than the client's. `welcome` states it in `bounds`, a request that would
exceed it is refused with `subset-too-large` rather than quietly trimmed, and
`docs/decisions/0010-mobile.md` sends the number itself to #76.

### `bounds`

The limits in force for a session, stated once rather than discovered by being
refused, which is the same rule the capability document follows in
`docs/decisions/0006-api-shape.md`.

| field | type | required | meaning |
| --- | --- | --- | --- |
| `page` | number | yes | The largest `max` a `pull` may ask for. |
| `queued_changes` | number | yes | The largest number of changes one `push` may carry. |
| `subset_records` | number | yes | The largest synced set this instance will serve. |

```json
{
  "page": 500,
  "queued_changes": 5000,
  "subset_records": 20000
}
```

## A full sync, in order

A full sync is what a client that holds nothing does, and what a client is sent
back to when an incremental sync can no longer be served.

The client sends `hello` with no `cursor`. The server answers `welcome` with
`serves` as `full`. The client then sends `pull` without a cursor, and repeats
`pull` with the cursor from the last `page` until a `page` arrives with `more`
false. Each `page` carries `covers` as `snapshot`.

A snapshot is paged rather than sent whole, and the paging is the same mechanism
an incremental sync uses. That is deliberate: a full sync of a large subset over a
poor connection is the case a client meets on its first day, and a protocol with
one resumable path and one unresumable path is a protocol whose worst case is the
unresumable one.

The cursor a `page` carries during a full sync is a point in the log, not a page
number. A client that stops halfway through a snapshot and comes back sends that
cursor and receives the rest of the snapshot, not a fresh one.

## An incremental sync, in order

The client sends `hello` with the cursor it stored. The server answers `welcome`
with `serves` as `incremental`. The client pushes before it pulls, so that a
conflict is judged against the state the server holds rather than against a state
the client is halfway through applying.

    hello    -> welcome
    push     -> receipt
    pull     -> page   (repeated while more is true)

A client with nothing queued skips the `push` and the sequence is otherwise the
same. A client whose queue exceeds the `queued_changes` bound sends it in several
`push` messages, oldest first, and waits for each `receipt` before sending the
next, because the order is what the server applies in.

## Subsetting, and changing a subset

The synced set is bounded and it is the operator's data leaving their building,
so it is stated rather than assumed. `docs/decisions/0010-mobile.md` fixes what it
holds and this protocol carries the request and the answer.

A client changes its subset with `subset`, inside a session, and does not resync.
The server answers with pages whose `covers` is `snapshot` for the records that
have newly entered the set, and whose `gone` names what has left it with `why` as
`left-the-subset`. A record that left the subset is not a record that was deleted,
and a client that treats the two the same shows a person that their data is gone.

A subset request larger than the instance will serve is refused with
`subset-too-large` and the refusal carries the bound. It is not trimmed to fit: a
client that asked for a set and received a smaller one without being told holds a
partial set it believes is complete, and every read it makes afterwards is wrong
in a way nobody can see.

## Resuming after an interruption

Every `page` carries the cursor to send next, and a client stores that cursor
before it applies the page rather than after. A client that dies between the two
re-receives a page it has already applied, which is why applying a page is
required to be idempotent; a client that stored the cursor first and died would
skip a page instead, and a skipped page is a record that is silently missing.

A `push` is resumed by sending it again. Every change carries a key the client
chose and holds across retries, so a `push` that was applied and whose `receipt`
was lost is applied once and answered a second time. A server that has seen a key
before returns the outcome it returned the first time, with `state` as
`superseded` where a later change has since moved the same field.

A session that has gone away is refused with `session-unknown`, and the client
opens a new one with `hello` and the cursor it last stored. No progress is lost by
that, which is the property the cursor exists for.

## A schema change while a client holds records

The description version travels on `hello`, on every `page` and on every `push`,
and it is the same version `docs/decisions/0006-api-shape.md` defines. It is
opaque and it moves whenever an operator adds, removes or retypes a field.

Where the change is additive, a sync is served. The `page` carries the current
version, so a client learns that it is behind on the next successful message
rather than by failing, and it fetches the description again at its own pace.

Where the change removes or narrows something a message names, that message is
refused with `schema-moved`, and the refusal names the object, the field and the
current version. The server does not guess and does not drop the clause, which is
that record's rule applied here rather than a second rule invented here.

A client holding a queued change to a field that has since been removed is the
case worth stating on its own, because it is the one that loses somebody's work
if it is left implicit. The change is refused with an `outcome` of `refused`
naming the field. It is not applied to a neighbouring field, it is not dropped
silently, and the client keeps it and shows it to the person who made it. There is
no state in which the server decides on its own that a person's typing did not
matter.

## The long offline case

A change log is retained rather than kept forever, which is
`docs/decisions/0004-change-log.md`'s rule and #102's setting. So there is a point
at which a client's cursor is older than the oldest entry the instance still
holds, and after that point an incremental sync is not a thing the server is able
to serve rather than a thing it declines to.

The server says so instead of pretending. `welcome` answers with `serves` as
`full` and a `reason`, or, where the cursor arrives on a later message, the
refusal is `cursor-too-old` with `restart` as `full` in its detail. A client that
receives either starts a full sync and keeps its queue: the queue is the part that
is not recoverable from the server, and a client that discarded it here would lose
exactly the work that being offline for a long time produced.

A cursor that is not a cursor this instance issued is a different case and gets a
different code, `cursor-unreadable`. Collapsing the two would hide a client that
has been pointed at a second instance behind a message about time.

## The requirements

Numbered so that #74 can map a case to each one and refuse a requirement with no
case. The numbering is dense and in order, which `sync-example-scan.js` refuses a
departure from, because a requirement that was removed and left a hole is a
requirement a suite cannot tell from one nobody wrote a case for.

R1. A client offers every protocol version it speaks, most preferred first, and
the server answers with the one it chose.

R2. A server that speaks none of the offered versions refuses with
`protocol-unknown` and names what it speaks.

R3. A client that receives an `instance` other than the one it holds stops and
does not merge.

R4. Every message after `hello` carries the session identifier from `welcome`.

R5. A client meeting a `type` it does not know refuses the message.

R6. A client meeting an unknown value in an open enumeration keeps the record and
does not drop it.

R7. A full sync is paged by cursor and is resumable at every page boundary.

R8. A client stores the cursor from a `page` before it applies the page.

R9. Applying a page is idempotent.

R10. A client pushes before it pulls in an incremental sync.

R11. Changes in a `push` are ordered oldest first and the server applies them in
that order.

R12. Every change carries a key that is stable across retries.

R13. A server that has seen a key before answers with the outcome it gave the
first time.

R14. A `receipt` carries exactly one outcome per change, in the order the changes
were sent.

R15. A `conflicted` outcome leaves the change in the client's queue.

R16. A change to a field the schema no longer holds is refused per change and the
client keeps it.

R17. A message naming something the schema has removed or narrowed is refused with
`schema-moved`, naming the object, the field and the current version.

R18. An additive schema change does not refuse a sync, and the current version
travels back on the next successful message.

R19. A subset request larger than the instance will serve is refused with
`subset-too-large` and is never trimmed to fit.

R20. A `subset` message changes the synced set inside a session and does not
require a full sync.

R21. A record that has left the subset is reported in `gone` with `why` as
`left-the-subset`, distinctly from one that was deleted.

R22. A cursor older than the retained log is answered with a full sync or refused
with `cursor-too-old`, and never with a partial answer.

R23. A client sent back to a full sync keeps its queue.

R24. A cursor this instance did not issue is refused with `cursor-unreadable`
rather than with `cursor-too-old`.

R25. A refusal is a 4xx or 5xx status carrying the error document, never a 200
carrying a failure.

R26. An amount crosses as an integer count of minor units with its currency.

R27. An absent field and a `null` field are distinct and a client preserves the
difference.

R28. A session that has gone away is refused with `session-unknown` and the client
resumes from its stored cursor with a new `hello`.

R29. The `bounds` in `welcome` are the limits in force, and a client does not ask
for more than they state.

R30. No message shape lets a server report that a queued change was discarded.

## What this document does not decide

The conflict resolution rule. #73 holds it, `outcome` carries its result, and what
is fixed here is only that a discard is not expressible.

The cap on the synced set as a number. `docs/decisions/0010-mobile.md` sends it to
#76, alongside the encryption question it belongs with.

The retention of the change log, which is what decides how long a client may be
away before a full sync is the only answer. #102 sets it and this document reads
whatever it says.

Which platform the first client is built for. That is an entry in #129 and
`docs/decisions/0010-mobile.md` states that nothing about the protocol depends on
it, which is the point of putting the split where that record puts it.

The transport-level authentication. #26 builds the token exchange and
`docs/decisions/0006-api-shape.md` puts it in the fixed core ahead of discovery,
so a session here begins after a client is already authenticated and this document
adds no credential of its own.
