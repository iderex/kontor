# The checks requested in front of the default branch

This document is a request. Changing a repository's protection is an act
performed by the maintainer on the repository, and nothing in this tree can
perform it, so what a plan can do is name the exact contexts, say what each one
costs, and record the state that was actually read back afterwards.

Everything below is read from the repository rather than from the files in it.
Where a name appears, the command that printed it is above it.

## The state today

    gh api repos/iderex/kontor/rulesets --jq '.[] | {id, name, target, enforcement}'
    {"enforcement":"active","id":20486686,"name":"gate","target":"branch"}

    gh api repos/iderex/kontor/rulesets/20486686 \
      --jq '{enforcement, bypass: .bypass_actors, required: [.rules[].type],
             checks: [.rules[] | select(.type=="required_status_checks")
                              | .parameters.required_status_checks[]?.context]}'
    {"bypass":[],"checks":[],"enforcement":"active","required":["deletion","non_fast_forward","pull_request"]}

    gh api repos/iderex/kontor/branches/main/protection
    {"message":"Branch not protected", ... ,"status":"404"}

Three things follow from that output and each matters to what is requested
below.

There is one ruleset, it is active, and it has no bypass actors, so what it
requires it requires of everybody.

It requires no status check at all. The `checks` list is empty, which means
every check named in the next section runs, reports, and is ignored by the merge
button. A red run today blocks nothing.

The classic branch protection endpoint answers 404. That is not a gap; it is
where the mechanism is not. The ruleset is the mechanism, and a request written
against the classic endpoint would be a request to configure a thing this
repository does not use.

The pull request rule's own parameters are worth reading with the rest:

    gh api repos/iderex/kontor/rulesets/20486686 --jq '.rules[] | select(.type=="pull_request") | .parameters'
    {"allowed_merge_methods":["merge","squash","rebase"],"dismiss_stale_reviews_on_push":false,"require_code_owner_review":false,"require_last_push_approval":false,"required_approving_review_count":0,"required_review_thread_resolution":false,"required_reviewers":[]}

A pull request is required and an approval is not. That is the honest state of a
project with one person in it, and it is stated here rather than left for
somebody to discover from a merge that nobody reviewed.

## The contexts

This document names names, where `CONTRIBUTING.md` and `docs/quality-parity.md`
both deliberately refuse to, and the difference is worth stating rather than
leaving a reader to notice it as an inconsistency. Those two describe what the
repository has, and a description that lists drifts against the thing it
describes. This one is a request for a set that does not exist yet, and a
request that names nothing cannot be acted on or argued with. The list below is
still derived rather than remembered: every name is printed by the command above
it, at a named commit, and a reader who re-runs that command against a later
commit gets the set as it is then rather than as it was here.

A required check is matched by the name a run reports under, not by the name of
the workflow or the job in the file, so the set has to be read off a commit that
has actually been judged. On a commit reached through a pull request:

    gh api "repos/iderex/kontor/commits/$(git rev-parse HEAD)/check-runs" --jq '.check_runs[].name'

Read against `b4dab90ff3336ab6215af96fd8a528bcf618ed25`, which is the last
commit to arrive on the default branch through a pull request, the pull request
event produced these twelve names, shown here with the workflow that produced
each so that a reader can find it:

    Build both layers                                              build
    Type check the client workspace                                client-types
    Prove the client type gates bite                               client-types
    Reject an unreasoned escape from the type system               client-types
    Reject a carriage return in tracked text                       text-determinism
    Reject non UTF-8 and byte order marks in tracked text          text-determinism
    Reject a generated file that is not what its generator writes  text-determinism
    Prove the determinism gates bite                               text-determinism
    Reject Trojan Source Unicode                                   unicode-guard
    DCO sign-off                                                   DCO
    Review new dependencies against the advisory database          Dependency review
    Audit workflows (zizmor)                                       Workflow Security Analysis

The mapping is the second command rather than a reading of the files:

    for id in $(gh api "repos/iderex/kontor/actions/runs?head_sha=$(git rev-parse HEAD)&per_page=100" \
                  --jq '.workflow_runs[] | select(.event=="pull_request") | .id'); do
      printf '%s :: ' "$(gh api "repos/iderex/kontor/actions/runs/$id" --jq .name)"
      gh api "repos/iderex/kontor/actions/runs/$id/jobs" --jq '[.jobs[].name] | join(" | ")'
    done

Two more names appear on that commit and neither belongs in the requested set.

`zizmor` is a code scanning check run, produced by the upload of the analysis
rather than by a job, so it is not one of the twelve and its name is the tool's
rather than a sentence anybody here chose.

`Scorecard supply-chain security` does not run on a pull request at all, so it
contributes no check run to a commit that arrives by one and cannot be required
of a merge. Requiring it would leave every pull request waiting for a verdict
that never comes.

Two workflows are known to the repository and produced no check run on that
commit, so they are not in the set today. `gh workflow list --repo iderex/kontor`
prints them; they are the formatting and lint gate of #3 and the test suite of
#5, and the procedure at the end of this document is how a name joins the set
once it has run.

## What each requirement costs

The cost of requiring a check is what it blocks, and the list is short because
these checks are cheap and refuse narrow things.

`Build both layers` blocks a merge that does not compile in either language.
There is no case for merging one, and what it costs to run is the twenty odd
seconds measured below. What it costs to wait for is a different number and the
section below separates the two.

The three client type contexts block, in order, a workspace that does not type
check, a change that loosened a compiler setting the proof depends on, and an
escape from the type system with no reason written beside it. The third is the
one that will be argued with, because it blocks a merge for a missing sentence.
That is the intended cost: the sentence is what makes the escape reviewable.

The four determinism contexts block a carriage return in tracked text, a file
that is not UTF-8 or carries a byte order mark, a generated file that is not
what its generator writes, and a change that loosened any of those three where
they live. The third has the largest cost, since it needs a Node install and a
cargo metadata resolution and is therefore the slowest of the four, and it is
also the only one that catches a hand edit to a lock file.

`Reject Trojan Source Unicode` blocks bidirectional and invisible control
characters in tracked text. Its cost is close to zero and it stays that way
because it never needs a toolchain.

`DCO sign-off` blocks a commit whose trailer does not match its author. Its cost
is real and it is paid by contributors rather than by the machine: a branch that
has been written without `git commit -s` needs a rebase before it can merge, and
the check names the commit and the line it wanted.

`Review new dependencies against the advisory database` blocks a pull request
that introduces a dependency with a known advisory. Its cost is the one on this
list that falls outside the repository's control, because the advisory database
moves. A dependency that was clean when the branch was written can redden it
later, which is a true finding rather than flakiness, and it is the reason the
next section separates the two ideas carefully.

`Audit workflows (zizmor)` blocks a workflow file with a known unsafe pattern.
Its cost is that it judges the files that judge everything else, so a change to
CI is where it will be met, and that is where it is wanted.

## Which of them should be advisory rather than required

The rule for the split is not invented here. `docs/quality-parity.md` states it:
a check is a merge condition when it is deterministic, bounded and unambiguous,
and advisory when it scores rather than judges, or when it samples. What this
section does is apply that rule to the twelve names above with a measurement
rather than an impression.

Bounded is measurable and is measured below. Unambiguous is a judgement about
each check's failure message. Deterministic is the test that turns out to be
interesting, and one of the twelve does not pass it cleanly.

Bounded turns out to be two numbers rather than one, and reading only the first
of them is how this section would have got the answer wrong. What a check costs
to run is one number. How long a merge waits for its verdict is another, and
they are not close.

What the checks cost, read off the same commit as the names above, from each
check run's own start and finish rather than from its workflow run's:

    gh api "repos/iderex/kontor/commits/b4dab90ff3336ab6215af96fd8a528bcf618ed25/check-runs?per_page=100" \
      --jq '.check_runs[] | select(.status=="completed") | [.name, .started_at, .completed_at] | @tsv'

Twenty one completed check runs, because nine of the names on that commit were
produced twice and the section after this one is about that duplication. Every
one of them finished between two and twenty four seconds. The longest is the
regeneration of the generated files at twenty four, with the build and the
workflow audit next at twenty three, and those are the three that install
something before they can start. The shortest are the text scans, which install
nothing and finish in three to eight. Nothing in this set is slow to run.

That is not what a merge waits for. A workflow run's own timestamps include the
time it spends queued before a runner takes it, and the run level figure is
therefore execution plus queue rather than execution:

    gh api --paginate "repos/iderex/kontor/actions/runs?per_page=100&status=completed" \
      --jq '.workflow_runs[] | [.created_at, .updated_at, .name] | @tsv'

Read as this was written, the six longest completed runs each span between five
hours fifty six minutes and six hours seven minutes from creation to last
update, for check runs that execute in under half a minute. Twenty nine further
runs had been created and had not started at all, and one was in flight, out of
one hundred and ninety three the repository holds:

    gh api --paginate "repos/iderex/kontor/actions/runs?per_page=100" \
      --jq '.workflow_runs[].status' | sort | uniq -c
    163 completed
      1 in_progress
     29 queued

A required context does not hold a merge for the twenty four seconds it runs. It
holds the merge until its verdict arrives, and on the evidence above the arrival
time is set by the runner queue rather than by anything in this repository.

That is not an argument for making any of these advisory. An advisory check on
the same queue reports just as late; it stops mattering, which is a different
property from being fast. It is an argument for reading the queue before the
request is applied rather than after, because the twenty four seconds is the
number somebody will quote and it is not the number that will be felt. Whoever
adds the first context should run the second command above first, and the
procedure at the end of this document is where that belongs.

Flakiness is the question no measurement here can answer, and saying so is more
useful than a reassuring sentence. The same call grouped by conclusion returns
no failed run of any workflow, one hundred and sixty three successes and no
other conclusion, against the thirty runs above that have not reached one. A
history with no red in it tells you nothing about how a check behaves when it
goes red, and it cannot distinguish a check that never fails from one that has
not yet been given the chance.

`Review new dependencies against the advisory database` is the one that fails
the determinism test, and the map places it as a merge condition anyway. Its
verdict is a function of the commit and of an advisory database that moves
underneath it, so the same commit can be green today and red next week without
anything in this repository changing. That is not flakiness and the difference
matters: the second verdict is the true one and the first was merely early. It
is still a check whose failure can arrive with no change to fix, and an operator
of this repository will meet that as a pull request that stopped merging
overnight. Requiring it is defensible, and it is requested here, but it is
requested with that property written down rather than as though the check were
a pure function of the diff.

The other eleven pass all three tests on the evidence available. None is
proposed as advisory, and the flakiness half of the split is owed to the first
failure anybody sees rather than settled here.

## The thing to settle before any of this is applied

Every workflow in this tree triggers on `push` to every branch and on
`pull_request`, so a commit that is both pushed and in a pull request is judged
twice under each name:

    S=b4dab90ff3336ab6215af96fd8a528bcf618ed25
    gh api "repos/iderex/kontor/commits/$S/check-runs" --jq '[.check_runs[].name] | length'
    22
    gh api "repos/iderex/kontor/commits/$S/check-runs" --jq '[.check_runs[].name] | unique | length'
    13
    gh api "repos/iderex/kontor/commits/$S/check-runs" \
      --jq '[.check_runs[].name] | group_by(.) | map(select(length == 2)) | length'
    9

Twenty two runs under thirteen names, and nine of those names are carried by two
runs each. The nine are exactly the jobs of the four workflows that ran on both
events; the four names that appear once belong to workflows that ran on the pull
request only.

Requiring a context whose name is carried by two runs of two different events is
a request whose meaning depends on how the platform resolves the duplicate, and
this document does not know how it resolves it. Reading that behaviour, or
removing the duplication by deciding which event a gate belongs to, belongs to
#6, which owns what runs on a pull request. It is named here because applying
this request before that is settled would be requiring something nobody has
read.

## Signatures

Not requested, and the reason is that this repository already made the other
choice. The trailer of the Developer Certificate of Origin is what every commit
here carries, `DCO sign-off` is what refuses one that does not, and the two
mechanisms answer different questions: a signature says a key held the commit,
and the trailer says the author asserts the right to contribute it. Requiring
signatures as well would add a key management obligation to every contributor
for an assurance this project has not argued it needs. If it is ever wanted, it
is a decision record rather than a line added here.

## How a check joins the required set later

Four steps, in this order, so that the set can grow without this document being
rewritten.

The check runs on a pull request and is seen to run. Until a name appears in the
first command of this document against a real commit, requiring it would block
every merge on a verdict that never arrives.

Its job carries a name written out in the workflow file rather than derived from
a matrix value or a job id, because a required check is matched by that string
and a rename silently stops requiring it.

The issue that added the check states the name and links the run that produced
it, so the request to add it to the set carries its own evidence.

The maintainer adds the context to the `required_status_checks` rule of the
`gate` ruleset, and the read-back at the top of this document is run again and
recorded, because a request that was made and a request that was applied leave
different output and only one of them is a control.

## What has not happened

Nothing in this document has been applied. The read-back at the top is the state
as this was written, `checks` is empty, and no check stands in front of the
default branch today. That sentence is the whole of it and it does not get
softer.
