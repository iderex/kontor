# The core objects, and what each one means

Five objects exist before an operator configures anything: the person, the
organisation, the deal, the activity, and the pipeline the deal moves through.
Everything else is a custom object, which is a real table under
`docs/decisions/0003-custom-fields.md`.

This note is written for somebody who sells for a living to disagree with. Where
a definition here is wrong, it is wrong now, in a paragraph, rather than in the
forecast of M4 where the same mistake is a number nobody can trace.

None of this is enforced. There is no schema, no metadata layer and no check
that reads a definition, and #17 is where these become the built in definitions
the metadata layer carries.

## The person

A person is a human being the operator deals with. Not a role, not a mailbox, not
a job title.

Required: a name, or an email address, or a telephone number. At least one of
those three, because a person nobody can reach is a note rather than a record,
and requiring all three loses the business card somebody typed in from a
conference.

A person may belong to an organisation, and may belong to none. A person who
changes employer is one person whose organisation changed, and the change is in
the log of #15. Making them two records loses the history that made them worth
knowing.

Identity, which is the rule the import of #23 and any deduplication depend on:
two records are the same person when they carry the same email address,
lowercased. Nothing else is identity. A tagged address is a different address,
because it is a different mailbox to the system that issued it, and treating
`a+sales@example.com` as `a@example.com` merges two people who are sometimes two
people.

Where neither record has an email address, they are the same person when the
same normalised name sits in the same organisation and carries the same
telephone number in E.164 form. Where neither has an email address and one has
no organisation, there is no identity rule, and the import creates a second
record rather than guessing. A wrong merge is not recoverable by the person who
notices it; a duplicate is.

## The organisation

An organisation is a legal entity the operator sells to. A department is not one.
A brand is not one. A group of companies is not one, and the parent relationship
below is how that is expressed.

Required: a name.

An organisation may have a parent organisation, forming a tree. The tree is for
reporting the way a customer is actually structured, and it may not contain a
cycle. Nothing else about it is structural.

Identity: two records are the same organisation when they carry the same
registered identifier, meaning the number the operator's jurisdiction issues to a
company, where the operator records one. Where they do not, it is the same
primary web domain, compared without the leading `www.` and lowercased. Where
neither exists, there is no identity rule and the import creates.

The name is deliberately not identity. Two organisations with the same name are
common, one organisation writes its name six ways, and every product that has
matched on names has produced a customer list nobody trusts.

## The deal

A deal is one opportunity to be paid by one organisation. It is the object every
number in M4 is computed from, so it is the one to argue about hardest.

Required: a name, an organisation, a pipeline, a stage, and a close date. Not an
amount, for the reason two sections below.

A deal belongs to exactly one organisation and to exactly one pipeline at a time.
It may name any number of people as the ones involved, and one of them is the
one who decides, or none is and that is a fact worth reporting on.

Identity: a deal has no natural identity. Two deals with the same name against
the same organisation are two deals, because selling the same thing twice to the
same customer is the ordinary case rather than a mistake. On import, identity is
the identifier the source system carries and nothing else; without one, the
import creates. This is the object where a clever identity rule does the most
damage, because merging two deals silently halves a pipeline total.

### What the amount means

The amount is the total the operator expects to be paid over the committed term
of what is being sold, in the currency of the deal, excluding tax.

Every clause there is doing work. Total rather than annual, because a three year
contract and a one year contract at the same annual rate are not the same
opportunity. Committed term rather than expected lifetime, because a renewal that
has not been agreed is a different deal. Excluding tax, because tax is a property
of where the customer is and not of the sale, and a pipeline that mixes the two
compares deals in different jurisdictions wrongly.

How money and currency are represented is #22, which writes it down as a record
of its own, including the rule that a total across currencies carries the rate
and the moment the rate was taken. This note does not anticipate that answer and
does not restate it.

### When the amount is trustworthy

It is trustworthy from the first stage whose entry condition requires that a
price has been given to the customer, and not before.

Before that stage the amount is an estimate somebody typed, and it is reported as
one. Every report that sums amounts states how much of the total came from stages
where the amount is not yet trustworthy, because a pipeline dominated by early
stage guesses is a different thing from one dominated by quoted deals, and the
sum on its own cannot tell the two apart.

Which stage that is, is a property of the stage rather than of this note, so an
operator whose process differs is not lying to the system. It is stated as
`amount_is_quoted` on the stage.

### A deal with no amount

A deal may have no amount, and what that does to every aggregate is stated here
once rather than decided per report.

It is counted in counts. It is excluded from sums and from averages. It is never
treated as zero.

Every aggregate that excluded one says how many it excluded, next to the number,
in the same response and in the same view. A pipeline total of 400,000 over
twelve deals when four more have no amount is a different fact from a total of
400,000 over sixteen, and a reader who is not told cannot tell the two apart.

Treating a missing amount as zero is the specific defect this paragraph exists
against. It makes a forecast look complete, it makes an average wrong in the
direction of optimism, and it is invisible in the output.

## The activity

An activity is something that happened between the operator and a person: a
note, a call, a meeting, or a task that is either still to be done or was done.

Required: a kind, a moment, and one record it is about. Every activity points at
a person, an organisation or a deal, and may point at more than one.

A task is the one kind that is about the future, and it carries a due moment and
a completion state. A task nobody completed is not deleted when it is overdue.

Identity: the identifier of the thing the activity came from, meaning the message
identifier of a mail, the identifier of a calendar entry, or the call identifier
of a telephone system. An activity somebody typed by hand has no identity rule,
and importing the same file twice creates two of them. That is stated here so
that #83 and #85 do not each invent an answer.

## The pipeline, and the stages in it

A pipeline is an ordered list of stages. A deal is in exactly one stage of one
pipeline at any moment.

An operator may have several pipelines, because selling a product and selling a
service are different processes, and forcing both through one list is what makes
a conversion rate meaningless.

Movement between stages is recorded. Every move writes to the change log of #15
with the stage before, the stage after, the moment, and the actor. Movement
backwards is recorded identically to movement forwards, and neither overwrites
the other.

That last sentence is the one forecasting depends on. A deal that went to
negotiation and came back to discovery is the most informative deal in the
system, and a model that keeps only the current stage cannot see it happened.
Time in stage under #38, conversion under #38, and the change waterfall of #42
are all computed from those entries and never from the deal's current state.

### What a stage means

A stage carries an entry condition, written as something somebody can check, and
it is required and may not be empty.

The condition says what must be true for a deal to be in the stage, not what
somebody intends to do next. "The customer has told us the budget exists and who
signs" is a condition. "Qualified" is a name.

The point is that two people in the same team put the same deal in the same
stage. Where a stage means something different to two people, every conversion
rate computed from it is an average of two unrelated numbers, and no test catches
it because the data is well formed.

Each stage also carries whether reaching it means the amount has been quoted,
which is `amount_is_quoted` above, and whether it is an end state, meaning won or
lost. A pipeline has at least one won stage and at least one lost stage, and a
deal in an end state does not move on its own.

The default pipeline a new instance is seeded with belongs to #122, and its
stages are examples rather than part of this definition. What this note fixes is
that every stage in every pipeline carries a condition, whoever wrote it.

## What an operator may change

All five objects may be renamed, in the sense that their display label is the
operator's. An operator who calls an organisation an account or a deal an
opportunity is using the same object with their own word for it, and nothing in
the product depends on the English word in this note.

All five may be extended with custom fields, under
`docs/decisions/0003-custom-fields.md` and within the bound that record states.

None of the five may be removed, and none may have a required field made
optional. They are structural: the change log, the reporting engine, the workflow
engine and the sync protocol all name them, so removing one is not a
configuration change but a different product.

Hiding is the one that needs care, and it is allowed for four of the five. An
operator who does not sell to companies may hide the organisation object from
navigation, and it goes on existing behind the deals that reference it. The deal
may not be hidden, because the whole of M4 reads it and a reporting engine with
its subject hidden is a broken instance rather than a configured one.

A custom object is the opposite of all of this. It may be created, renamed,
hidden and removed by the operator, and removing one removes its table under the
two step deletion the custom field record describes.

## What this note does not decide

The tables, the column types and the keys. Those follow from here and are #17.

Permissions. Who may see and change which of these records is #21, and nothing
in this note grants or denies anything.

Deletion and restore of a record, as opposed to of a field, which is #20.
