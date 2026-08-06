# 0007. What a forecast actually computes

Status: accepted
Issue: #35
Supersedes: nothing
Superseded by: nothing

## The decision

A forecast is a statement about a period, and it is made of four pieces that are
usually conflated into one figure. They are reported separately, always, and the
single figure is available only as their sum with every piece still readable
beside it.

The open field is not short of charts. It is short of a definition of the numbers
on them, and a number whose definition is not written down cannot be argued with,
corrected, or trusted by anyone who did not build it.

## The four pieces

Closed and won to date. The sum of the amount field of every deal in the period
whose stage is the terminal won stage, taken as of the moment the number is
computed. It is arithmetic and it has no uncertainty. Its only subtlety is the
one #16 makes the deal object state: what the amount means and at which point in
a deal's life it is expected to be trustworthy.

The categorised pipeline. The sum of the amount of every open deal in the period,
grouped by the category a person has assigned to it, which is committed, best
case, or neither. This is a human judgement. The engine records it, timestamps
it through the change log, and never changes it. A category that the system
alters on the person's behalf is not a judgement any more, and the number stops
meaning what its name says. The categories and how they roll up are #40.

The fitted expectation. For every open deal in the period, a probability derived
from what actually happened to comparable deals, multiplied by the deal's amount,
and summed. This is the piece the engine computes rather than collects, and it is
defined precisely below.

The waterfall. The difference between the same period's figure at two moments,
decomposed into the movements that produced it: deals created, advanced, slipped
out of the period, pulled into it, grown, shrunk, won and lost. This is the
question a sales manager actually asks, and it is impossible without knowing what
the world looked like at the earlier moment, which is what the append only log of
`docs/decisions/0004-change-log.md` exists for. #42 builds it and #37 builds the
as-of query it stands on.

## The fitted probability, precisely

For an open deal, the comparable set is every deal that satisfies all four of
these: it is in the same pipeline; it reached the stage the open deal currently
occupies; its age in that stage, at the moment it was in it, falls in the same
quartile of the age distribution for that stage as the open deal's current age in
stage; and it reached a terminal stage, won or lost, inside the look back window.

The probability is the won count of that set divided by its size.

The look back window is twelve months by default, measured on the date a deal
reached its terminal stage. Twelve months rather than a longer window because a
sales motion changes and old deals stop being comparable; twelve rather than
shorter because a shorter window cuts a seasonal cycle in half and makes the
fourth quarter look like the first. It is configurable per pipeline. Whatever
value was used is printed with the number, so a reader never has to know the
default to interpret a figure.

The minimum comparable set size is thirty. That number comes from the width of
the interval it implies rather than from being round. The standard error of a
proportion is the square root of p times one minus p over n, which is largest at
p of one half:

    sqrt(0.25 / 30)  = 0.0913
    sqrt(0.25 / 100) = 0.0500
    sqrt(0.25 / 10)  = 0.1581

At thirty the estimate is worth roughly plus or minus nine points, which is
already close to the distance between adjacent stages in a typical pipeline. At
ten it is sixteen points, which is wider than that distance and therefore carries
no information about which stage the deal is in. Thirty is where the estimate
starts saying something the stage did not already say.

Below thirty the deal is not fitted. It is excluded from the fitted expectation
and reported in the unfittable set, which carries its count and its total amount
and is part of the answer rather than a footnote to it. No default is
substituted, no stage percentage is used, and no neighbouring quartile is
borrowed from. A forecast that says forty of these two hundred deals have too
little history to fit is more useful than one that quietly assumes they behave
like the average, and the second one is indistinguishable from the first until
somebody acts on it.

## The interval, and what it does not claim

Treat each fitted deal as an independent outcome that lands at its amount with
its probability and at zero otherwise. The expectation is the sum of probability
times amount. The variance is the sum of probability times one minus probability
times amount squared. The reported interval is the expectation plus and minus
1.96 times the square root of that variance.

What it claims: this is the dispersion implied by the fitted probabilities, if
those probabilities are right and if the deals are independent of one another.

What it does not claim, stated at length because an interval is the most easily
over-read object in a report.

It is not a confidence interval on a true value. It is a spread implied by a
model, and the model's inputs are estimates with their own error, which this
interval does not include.

It does not account for correlation between deals. Deals are not independent: a
lost quarter loses many of them together, for one reason. Correlation makes the
real spread wider, always, never narrower. So the reported interval is a floor on
the uncertainty rather than a bound on it, and it should be read as saying at
least this uncertain.

It says nothing about the unfittable set, which is outside it entirely, and
nothing about the categorised pipeline, which is a judgement rather than a
distribution.

It is not a prediction about any individual deal. A probability of 0.4 on one
deal is a statement about a population of comparable deals and not about that
one.

## The three rules, and where each is enforced

Every number states the definition it came from, the moment it was computed as
of, and the record set it covered. The definition comes from one place by
construction, which is #36. The as-of moment is what #37 makes answerable. The
record set is the permission filtered set for the caller, which is the same
filtering as a record read under #27 rather than a second implementation of it.

Every number can print the query that produced it. That is #44, and it is what
makes the first rule checkable rather than asserted.

The engine never fills a gap with a guess. Where it lacks the data, it reports
the gap in the same shape as it would report a number, which is why the
unfittable set has a count and an amount rather than a warning string. #45 proves
the whole path is deterministic, which is what stops a gap being filled
differently on two runs.

None of the three is enforced by anything in this repository today. There is no
code for a check to read, and the three issues named above are where each becomes
a mechanism. Until they land, the rules are held by a reader.

## The compute decision

Every number is a SQL query generated from a metric definition and run against
the same PostgreSQL instance that holds the records. There is no separate
analytical store at first release.

What that buys: one database for an operator to back up and restore, which is
#95; no synchronisation lag, so a report cannot disagree with the record page it
was opened from; and permission filtering that is the record read's filtering
rather than a second implementation of it, which is the copy most likely to be
subtly wrong and least likely to be noticed.

What it costs is a ceiling. The waterfall reads the change log, and
`docs/decisions/0004-change-log.md` puts that at roughly twenty six million
entries on the instance it describes. A grouped scan over that is fine. A
waterfall over a long range on an instance an order of magnitude larger is the
case that will not be.

## The alternative, and the condition that reopens this

The successor considered is a columnar copy of the change log, maintained beside
the records and read only by the reporting path. Not a different product and not
a hosted warehouse, because both of those give away the two things bought above.

The condition is measured rather than argued. Reopen this when the twelve month
change waterfall for one pipeline exceeds two seconds at the ninety fifth
percentile on the machine #126 states, at the row counts #126 reports, with the
query and both numbers quoted. #46 is where that budget is set and held, so the
measurement exists before the argument does.

Anything short of that is a query to fix rather than an architecture to change,
and the first response to a slow report is the query plan, not a second store.

## What this engine does not do

The readme claims advanced reporting and forecasting, and the claim needs a
boundary, so here it is.

It does not summarise anything in prose, and it does not generate text. The
engine is statistical, which is a choice recorded here rather than a limitation
being apologised for.

It makes no causal claim. It reports that deals in a stage closed at a rate. It
does not report that the stage caused it, and no view in the product is allowed
to phrase it that way.

It does not score a person. Nothing here derives a probability from an attribute
of a human being, and the comparable set is defined on deals and stages only.

It does not compare an operator's instance to anyone else's. There is no data
from another instance in this product and no route by which any could arrive.

It does not reconstruct history it does not have. A period before the retention
horizon is refused rather than estimated, which is the rule
`docs/decisions/0004-change-log.md` states.

It does not predict an individual outcome. Every number here is about a
population, and the interface is required to say so where it shows one.
