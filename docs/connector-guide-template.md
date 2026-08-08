# The connector guide template

A connector is a hole in the sentence this project is sold on, and
`docs/decisions/0011-connectors.md` is where that is argued. The declaration in a
connector crate's manifest is what a machine reads. This is what a person reads
before they turn one on, and #89 is where it is asked for.

One guide per connector, at `docs/connectors/<id>.md`, where `<id>` is the `id`
its declaration carries. Every heading below is required, in this order, and
`connector-guide-scan` refuses a guide missing one. The headings are read out of
this file rather than written into the check, so a heading added here is required
from that commit and the check cannot drift against it.

Write under each heading in plain language and for somebody who has not read the
code. Where an answer is uncomfortable, the guide is the place it belongs: an
operator who finds it out afterwards has already sent the data.

## What it needs

What the operator has to have before this connector can be turned on, and what
they have to enter. The address is theirs to name, so say where they find it.

## What crosses the boundary

Do not write under this heading. It holds a generated block, between the two
markers below, rendered from the declaration in the connector crate's manifest so
the guide cannot say something the code does not.

    connector-guide-scan --write

rewrites it, and a hand edit is refused by the same check that refuses a missing
one. The block a guide starts life with is this:

<!-- connector boundary: generated from the declaration, and a hand edit is refused -->

| Question | What the declaration answers |
| --- | --- |
| Purpose | |
| Destination | |
| Outbound | |
| Inbound | |
| Credential | |

<!-- connector boundary: ends -->

## What is stored on the host, and what is only referenced

The declaration says what crosses. This says what is kept afterwards, which is a
different question and the one an operator is answering to somebody else. Name
what is written into this instance, what is only pointed at, and how long each
one stays.

## What the operator has to do

The obligations that are theirs rather than this project's, including where an
agreement with the external service is theirs to hold, and what they have to be
able to show about it.

## How to disconnect, and what happens to what it brought in

The steps, and then the honest part: what is removed, what stays, and what has to
be removed by hand. A guide that stops at the button has answered the easy half.

## What is not covered

What the external service does with the data once it has it, which is outside
anything this project can promise, and anything else this connector does not
reach. State it as a limit rather than as a reassurance.
