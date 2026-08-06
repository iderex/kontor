# 0005. Money is an integer, a close date is a date, a log entry is a moment

Status: accepted
Issue: #22
Supersedes: nothing
Superseded by: nothing

## The decision

Money is an integer count of minor units with a currency attached, and the two
travel together. There is no amount without a currency and no arithmetic that
crosses one.

A close date is a calendar date with no time and no zone. A change log entry is
a moment on the clock. They are different types, they are stored in different
column types, and nothing converts between them silently.

One zone decides period boundaries for moments, it is instance configuration,
and dates do not use it because they do not need one.

`server/crates/money` is where these are types rather than conventions. What
follows is the argument; the crate is the enforcement, and
`server/crates/money/tests/properties.rs` is the evidence that it enforces.

## Money

An amount is `i64` minor units plus a currency. 92 EUR 30 is 9230 with EUR, and
930 JPY is 930 with JPY, because the yen has no minor unit.

No floating point anywhere, in the type, in a constructor, or in an accessor. A
binary float cannot hold 0.1, so a total assembled from floats depends on the
order the rows came back in, and two runs of one report disagree in the last
place for no reason the reader can see. That is not a rounding preference. It is
a report that cannot be reconciled with itself.

`i64` minor units reaches roughly 92 quadrillion of a two decimal currency,
which is not a bound anybody meets. Every operation that could leave the range
is checked and returns an error rather than wrapping.

The currency carries its own exponent, which is how many decimal places its
minor unit sits at. Two is the common answer and it is not universal: the yen
and the Icelandic krona have none. A system that assumes two prints amounts with
a fraction that cannot exist, accepts one that cannot be entered, and is wrong
by a factor of a hundred wherever a conversion touches it.

The set of currencies is small and is refused rather than defaulted. A currency
the instance does not know is an error at the edge, not two decimal places
assumed.

## Adding two currencies is not possible

There is no `Add` implementation for an amount. The only addition available
refuses two currencies, and the only way to reach a mixed total is through a
conversion that records how it was made.

That is the whole of the type level enforcement and it is deliberate that it is
an absence rather than a check. A check can be skipped by the one code path that
did not call it. A missing implementation cannot be skipped, because the code
that would need it does not compile.

What nothing here proves is the absence itself. No test can assert that an
implementation does not exist, and a later change adding one would compile,
pass every test in the crate and remove the guarantee this section is about.
That is a real gap and it is left open rather than papered over: catching it
needs a check over the source, which is #113.

## Conversion, and what a converted total carries

A conversion needs a rate, and a rate is a number with the moment it was taken.
There is no constructor for a rate without that moment.

The rate itself is an integer, being minor units of the target per million minor
units of the source, because a rate held as a float puts the defect the first
section refuses back in through the conversion path.

Where a rate comes from is the operator's table. Rates are entered or imported,
and nothing in this product fetches one from a service on its own. A background
fetch is a route out of the host, and every route out of the host is declared
under #97 and #88 rather than added quietly for convenience.

A total that used a conversion keeps every rate it used. A total that converted
nothing has an empty rate list, and that is how a reader tells the two apart. A
report showing a converted total shows those rates and their moments beside it,
under the rule of #44 that a report shows what produced it.

The rule this exists for is the one a reader cannot check on their own: a report
may not convert last year's deal at today's rate unless it says so. Both
behaviours are legitimate. Valuing a historical pipeline at the rate of the day
each deal closed and valuing it at today's rate answer different questions.
Silently doing either is what makes a number nobody can reproduce, so the rate
and its moment travel with the total and the report is required to show them.

## A date is not a moment

A close date is the day a deal closes. It is the same day for everybody looking
at it, and it lands in one quarter for everybody. Held as a `date` in the
database and as a civil date in the crate, with no zone anywhere near it.

This is what removes the ambiguity the issue behind this record describes. A
deal closing on the last day of a quarter cannot land in two quarters for two
people, because there is no moment to interpret and therefore nothing to
interpret it in.

A change log entry is a moment. It happened at one point in time, and two people
in different places agree on that point while disagreeing about what their
clocks read. Held as `timestamptz`, which PostgreSQL stores as an instant rather
than as a local reading, and as a timestamp in the crate.

The two are separate types in the crate for one reason: a conversion between
them is where the defect lives. Turning a date into a moment requires a zone and
turning a moment into a date requires a zone, and code that does either without
saying which zone is code that picked one.

## Which zone decides a period boundary

For a date, none. A close date is placed in its quarter by the calendar.

For a moment, the instance reporting zone decides. It is one setting on the
instance, part of the validated configuration of #90, and it fails closed: an
instance with no zone configured does not start rather than assuming UTC, since
an assumed zone produces reports that are quietly wrong at every boundary.

An operator changes the zone as they would any other configuration, and what
happens to existing data is nothing at all. No stored value changes. Instants
stay instants and dates stay dates, so there is no migration and no rewrite.

What does change is which period some moments fall into, and that has to be said
plainly rather than presented as a free setting. An entry made at 00:30 local
time on the first of a month, in a zone moved by two hours, is in the previous
month afterwards. A report run before the change and the same report run after
it can differ, at the edges, by the entries within the offset of a boundary.

That is why the change is recorded in the audit trail of #96 with who made it
and when. A number that moved and a number that was wrong look identical without
that record.

## What a daylight saving change does to a period

It makes it shorter or longer, and the crate computes the length rather than
assuming it.

In a zone that moves its clocks, the first quarter of a year is 23 hours short
of ninety days times twenty four, and the fourth is an hour long. Anything that
divides by the length of a period, which is every rate per day and every average
in M4, is wrong by that hour if it multiplies days by twenty four.

The first moment of a day is also not always midnight. Where a zone moves its
clocks forward at midnight, the local time 00:00 does not exist that day, and
the crate resolves the start of the day to the first moment that does rather
than to a reading nobody's clock showed.

## What is checked, and what is not

`server/crates/money/tests/properties.rs` checks the properties over every value
of a bounded domain rather than over a sample: every pair of currencies for the
addition rule, every day and every quarter of a four year window for the period
rules, in a zone that moves its clocks and two that do not.

The window contains two leap days and four transitions in each direction, so the
awkward cases are covered by the domain rather than by somebody remembering to
write them.

Three things are not covered and are named rather than left to be discovered.

The absence of an addition implementation, for the reason two sections above.

Rounding on conversion, which truncates toward zero today. A half up rule, a
banker's rule and truncation differ by a minor unit per conversion, and which
one an operator's accountant expects is a question this record does not answer.
It matters at the point money leaves the product, which is nowhere yet.

Whether any route runs these tests. Nothing does. `./build` compiles both layers
and runs no suite, and no workflow runs `cargo test`. #5 is the harness and #6
is what puts it on a pull request, and until both land these tests are run by
whoever remembers to type the command.
