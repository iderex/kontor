# 0002. Starting from an empty tree rather than from an existing CRM core

Status: accepted
Issue: #13
Supersedes: nothing
Superseded by: nothing

## The decision

This project starts from an empty tree. It does not fork or extend an existing
open CRM. The first release is deliberately narrow, and the boring three quarters
of a CRM is paid for by not building most of it yet rather than by inheriting it.

This is the decision every other one inherits, and it is recorded before a line
of the data model exists because reversing it later means throwing the data model
away rather than editing it.

## The candidates, and what each would have given

Five were considered. Every licence claim below is one command and its output,
run against the public API rather than recalled.

SuiteCRM. A complete CRM in PHP with two decades of accumulated correction behind
it: list views, custom modules, permissions, import, export, a settled interface.
Taking it would have removed most of M2, M6 and a good part of M12.

    gh api repos/salesagility/SuiteCRM --jq '.full_name, .language, .license.spdx_id'
    SuiteCRM/SuiteCRM
    PHP
    AGPL-3.0

The first line is a correction to the reasoning this record was asked to carry.
The path `salesagility/SuiteCRM` still resolves, but it resolves by redirect, and
the repository now answers as `SuiteCRM/SuiteCRM`. Nothing in the decision turns
on it. It is recorded because a redirect is the kind of thing that quietly
invalidates a pinned dependency later.

EspoCRM. Also PHP, and the closest of the five to the shape this project wants,
in that its metadata layer already treats objects and fields as run time data
rather than as code. That is the single hardest thing in M2, and it would have
been inherited working.

    gh api repos/espocrm/espocrm --jq '.language, .license.spdx_id'
    PHP
    AGPL-3.0

CiviCRM. The most complete answer for organisations rather than for sales teams,
with the deepest handling of the contact model and of the obligations around it.
It would have given the parts of M10 that are usually underestimated.

    gh api repos/civicrm/civicrm-core --jq '.language, .license.spdx_id'
    PHP
    AGPL-3.0

Frappe CRM. The newest of the four in the same licence family, built on a
framework that already provides the metadata layer, the permission model and the
list interface, so the CRM itself is a thin layer over machinery somebody else
maintains.

    gh api repos/frappe/crm --jq '.language, .license.spdx_id'
    Vue
    AGPL-3.0

Twenty. The most modern candidate by a distance and the one with the largest
attention, in the same language this project has chosen for its client.

    gh api repos/twentyhq/twenty --jq '.language, .license.spdx_id'
    TypeScript
    NOASSERTION

`NOASSERTION` is where the identifier stops being useful, so the file itself was
read:

    gh api repos/twentyhq/twenty/contents/LICENSE --jq .content | base64 -d | head -20

It is mostly AGPLv3 with an additional permission granted under section 7. Some
packages are under MIT instead, named in the file and marked by the `license`
field of their own `package.json`. And individual files are placed under a
separate commercial licence by a comment at the top of the file, the literal
being an `@license Enterprise` marker in a comment. A fork inherits the
obligation to track that marking file by file, forever, and a file that gains the
marker upstream after the fork is a licence change arriving through a routine
merge.

## Why none of them was taken

Three arguments, in the order of how much they decide.

The first is upstream motion. Releases arrive at roughly weekly cadence:

    gh api repos/twentyhq/twenty/releases --jq '.[0:3][] | "\(.tag_name) \(.published_at)"'
    twenty/v2.27.0 2026-08-04T11:29:52Z
    sdk/v2.27.0 2026-08-04T11:30:39Z
    twenty/v2.26.0 2026-07-31T12:05:32Z

A fork that changes the write path of the core, which is exactly what #14 and #15
require, either rebases against that motion continuously or stops taking
upstream. The second is a fork in name only, and it is the outcome that actually
happens, because the first is a standing cost nobody funds.

The second is that the two differentiators are not features that sit on top of a
CRM core. Forecasting the way this project means it is a set of questions about
the past state of a record, answered from history rather than from the current
row. Workflow automation the way this project means it is a durable engine
reading a change feed. Both are demands on the storage layer, and #15 makes the
change log the single source that both read. Adding that to somebody else's
storage layer means changing the part of it that everything else depends on,
which is the part a fork can least afford to change.

The third is the licence, and it is the weakest of the three rather than the
strongest. The reasoning this record was asked to carry places it first, on the
grounds that every serious candidate constrains a licence question this plan
leaves open. That is no longer the situation. The licence here is settled and
landed:

    gh api repos/iderex/kontor --jq .license.spdx_id
    AGPL-3.0

which is the same identifier four of the five candidates return, so on the plain
reading it constrains nothing. What survives of the argument is the fifth
candidate: a per file commercial marking is a real inheritance and a real ongoing
obligation, and it would have been an argument against that one candidate rather
than against forking as such. Recorded this way round because the original
ordering would have led a later reader to believe the licence did work it did
not do.

## What this costs

The cost is real and is not minimised here. This project will be worse than the
incumbents at ordinary CRM work for a long time, and the only honest response is
a first release that does a small set of things completely.

Specifically, the first release carries people, organisations, deals, activities
and the pipeline a deal moves through, and nothing else structural. That is
stated in #16, which adds that everything else an operator wants is a custom
object rather than a shipped one.

So the first release will not have, and an operator arriving from an incumbent
will notice the absence of: a product catalogue and price books; quotes, orders
and invoices; case or ticket management, meaning the service half of a CRM
entirely; marketing campaigns and any form of bulk campaign sending; territory
and hierarchy management; document generation; commission and lead scoring; and
an application marketplace. Nothing on that list is on the board:

    gh issue list --repo iderex/kontor --state all --limit 200 \
      --search '"price book" in:body' --json number
    []

The absence is deliberate rather than an oversight, and it is the price of the
decision recorded here. It should not be read as a promise that these arrive
later; it is a statement that they are not being built now and are not planned in
the milestones that exist.

## What this project still takes from the existing field

Not code, and one thing that matters more than any single feature: a route in.

An operator's existing data is the reason this decision has a debt attached at
all, and #123 is where that debt is paid. It requires a documented migration path
from at least one widely used open CRM and from the export format of at least one
commercial one, built on the import machinery from #23 rather than as a parallel
route, with the mapping written as a document an operator can read and adjust,
and with a report of what could not be mapped as the deliverable rather than as
an afterthought.

The second thing taken is the model itself, in the ordinary sense that the
objects in #16 are the objects the field settled on decades ago, and departing
from them without a reason would be a cost with no buyer.

## What would reverse this

Reverse it if the milestones that are not the two engines, meaning M2, M6 and
M12, turn out to consume the project while M4 and M5 stay unstarted. That is the
observable form of the argument against this decision, and it is checkable rather
than a matter of taste: compare the closed issue count in those milestones
against the closed count in M4 and M5.

Reverse it also if a candidate appears whose storage layer already writes a single
append only change log that both a reporting path and a workflow engine read,
because at that point the demand that forces this decision is already met
upstream and the argument for an empty tree has gone with it.

Reversal is expensive after the data model lands, which is the reason this record
is written before it does rather than after.

If this decision had gone the other way, #1 would be reopened, because the means
follows the core.
