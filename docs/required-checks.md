# The checks requested in front of the default branch

Changing a repository's protection is an act performed by the maintainer on the
repository, and nothing in this tree can perform it, so what follows is a
request. What a plan can do is name the exact contexts with what each one costs,
and then record the state that was read back afterwards.

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

## The settings requested alongside the contexts

Three of the four are already in force, and each is stated below with the reason
it is wanted. A setting that is in force for a reason nobody wrote down is a
setting the next person will remove.

A pull request is required, and it is wanted. It is the only moment in this
repository where a change is a readable object with a body attached to it rather
than a commit that has already happened. What it costs is the case it does not
distinguish: the pull request rule's own parameters say who has to read it.

    gh api repos/iderex/kontor/rulesets/20486686 --jq '.rules[] | select(.type=="pull_request") | .parameters'
    {"allowed_merge_methods":["merge","squash","rebase"],"dismiss_stale_reviews_on_push":false,"require_code_owner_review":false,"require_last_push_approval":false,"required_approving_review_count":0,"required_review_thread_resolution":false,"required_reviewers":[]}

A pull request is required and an approval is not. That is the honest state of a
project with one person in it, and it is stated here rather than left for
somebody to discover from a merge that nobody reviewed. Raising the approval
count above zero is not requested, because a requirement one person cannot
satisfy is a requirement that gets bypassed or removed, and this repository has
no bypass actors to do the first with.

Force pushes are refused, by the `non_fast_forward` rule, and it is wanted for a
reason narrower than tidiness. Every claim in this repository's documents is
anchored to a commit, and the whole method depends on a reader being able to
resolve that commit later and get what the writer saw. A force push to the
default branch turns those anchors into commits that are not reachable, and it
does it silently. What it costs is the repair route: a bad commit on the default
branch is corrected by a commit that reverts it and is therefore visible, rather
than by history that no longer contains it.

Deletion is refused, by the `deletion` rule. What it costs is close to nothing,
since nobody has a use for deleting the default branch, and what it prevents is
the one accident from which there is no local recovery.

Signatures are not requested, and the reason is that this repository already
made the other choice. The trailer of the Developer Certificate of Origin is
what every commit here carries, `DCO sign-off` is what refuses one that does
not, and the two mechanisms answer different questions: a signature says a key
held the commit, and the trailer says the author asserts the right to contribute
it. Requiring signatures as well would add a key management obligation to every
contributor for an assurance this project has not argued it needs. If it is ever
wanted, it is a decision record rather than a line added here.

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

That last sentence is not a formality here, and the drift it warns about has
already happened once inside this file. An earlier revision of this document
named twelve contexts, read off the commit that was then the most recent to
reach the default branch through a pull request. Three gates landed after it was
written and before it was applied, and the set below is more than twice the size.
Nothing was applied in between, so the cost was a stale document rather than a
wrong protection, but the shape of the mistake is the one this repository calls
`restated-not-referenced` and it is recorded here rather than quietly corrected.

A required check is matched by the name a run reports under, not by the name of
the workflow or the job in the file, so the set has to be read off a commit that
has actually been judged. On a commit reached through a pull request:

    gh api "repos/iderex/kontor/commits/$(git rev-parse HEAD)/check-runs" --jq '.check_runs[].name'

Read against `fb5cf5a42eb1470b1cf620cc12f5008f245ccf00`, the head of the last
pull request to reach the default branch, the pull request event produced these
twenty five names, shown here with the workflow that produced each so that a
reader can find it:

    Build both layers                                                     build
    Type check the client workspace                                       client-types
    Prove the client type gates bite                                      client-types
    Reject an unreasoned escape from the type system                      client-types
    Check the client formatting                                           format-and-lint
    Check the server formatting                                           format-and-lint
    Lint the client                                                       format-and-lint
    Lint the server                                                       format-and-lint
    Reject a lint suppression naming no rule or giving no reason          format-and-lint
    Prove the formatter and the lint gate bite                            format-and-lint
    Reject a carriage return in tracked text                              text-determinism
    Reject non UTF-8 and byte order marks in tracked text                 text-determinism
    Reject a generated file that is not what its generator writes         text-determinism
    Prove the determinism gates bite                                      text-determinism
    Run the unit suite                                                    tests
    Run the unit suite where there is nothing to reach                    tests
    Run the suite that needs a database                                   tests
    Reject a test that reaches for what the default suite cannot give it  tests
    Prove the marking bites                                               tests
    Prove the seal bites                                                  tests
    Hold the coverage floor                                               tests
    Reject Trojan Source Unicode                                          unicode-guard
    DCO sign-off                                                          DCO
    Review new dependencies against the advisory database                 Dependency review
    Audit workflows (zizmor)                                              Workflow Security Analysis

The mapping is the second command rather than a reading of the files:

    for id in $(gh api "repos/iderex/kontor/actions/runs?head_sha=$(git rev-parse HEAD)&per_page=100" \
                  --jq '.workflow_runs[] | select(.event=="pull_request") | .id'); do
      printf '%s :: ' "$(gh api "repos/iderex/kontor/actions/runs/$id" --jq .name)"
      gh api "repos/iderex/kontor/actions/runs/$id/jobs" --jq '[.jobs[].name] | join(" | ")'
    done

Two more names appear around that commit and neither belongs in the requested
set.

`zizmor` is a code scanning check run, produced by the upload of the analysis
rather than by a job, so it is not one of the twenty five and its name is the
tool's rather than a sentence anybody here chose. The workflow's own job reports
under `Audit workflows (zizmor)`, which is in the set.

`Scorecard supply-chain security` does not run on a pull request at all, so it
contributes no check run to a commit that arrives by one and cannot be required
of a merge. Requiring it would leave every pull request waiting for a verdict
that never comes. `docs/quality-parity.md` places it as advisory on its own
grounds, which is the same answer reached from the other direction.

Those two are the whole of the difference between the ten workflows
`gh workflow list --repo iderex/kontor` prints and the nine that produced a
check run on the pull request event above. Every other workflow the repository
knows about is represented in the set.

Both counts in that sentence are the ones read at `fb5cf5a4`, and two gates have
landed since. `supply-chain.yml` reports under `Prove the lock file gates bite`,
and `workflow-comments.yml` reports under `Reject a workflow comment naming a
file the tree does not hold` and `Prove the workflow comment rule bites`. All
three trigger on the pull request event, so they join the set the paragraphs
above describe rather than the two exceptions beside it. The numbers are
deliberately not rewritten here, because a number in this file is what went
stale the last time and the two commands above are what answer it at the commit
a reader actually holds.

## What each requirement costs

The cost of requiring a check is what it blocks. These are read as a
requirement, meaning what a merge cannot do once the context is required, rather
than as a description of what the check does.

`Build both layers` blocks a merge that does not compile in either language.
There is no case for merging one, and what it costs to run is the twenty seven
seconds measured below. What it costs to wait for is a different number and the
section below separates the two.

The three client type contexts block, in order, a workspace that does not type
check, a change that loosened a compiler setting the proof depends on, and an
escape from the type system with no reason written beside it. The third is the
one that will be argued with, because it blocks a merge for a missing sentence.
That is the intended cost: the sentence is what makes the escape reviewable.

The six formatting and lint contexts block unformatted source in either
language, a lint finding in either language, a suppression that names no rule or
gives no reason, and a change that loosened any of those where they live. The
suppression context has the same shape of cost as the escape context above, and
it is worth paying for the same reason. The formatting pair is the cheapest
thing on this list to satisfy and the one most likely to be met by somebody who
did not run the gate before pushing, since a formatter disagreement is invisible
in an editor that is configured differently.

The four determinism contexts block a carriage return in tracked text, a file
that is not UTF-8 or carries a byte order mark, a generated file that is not
what its generator writes, and a change that loosened any of those three where
they live. The third has the largest cost, since it needs a Node install and a
cargo metadata resolution, and it is also the only one that catches a hand edit
to a lock file.

The seven test contexts are the largest group and they block different things.
Two of them run the suite, once in the ordinary environment and once where
nothing outside the process can be reached, and requiring both is what makes the
headless rule a property of the merge rather than of somebody's habit. One runs
the suite that needs a database, which is the only context on this list that
depends on a service being stood up beside the runner, and it is therefore the
one whose failure is most likely to be about the runner rather than about the
diff. One refuses a test that reaches for something the default suite cannot
give it and is not marked for it. The remaining three are proofs: that the
marking bites, that the seal bites, and that the coverage floor holds. The
coverage floor is the only context in the whole set whose verdict moves with a
number rather than with a property, and it will be the first one somebody wants
lowered.

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
section does is apply that rule to the twenty five names above with a
measurement rather than an impression.

Bounded is measurable and is measured below. Unambiguous is a judgement about
each check's failure message. Deterministic is the test that turns out to be
interesting, and one of the twenty five does not pass it cleanly.

Bounded turns out to be two numbers rather than one, and reading only the first
of them is how this section would have got the answer wrong. What a check costs
to run is one number. How long a merge waits for its verdict is another, and
they are not always close.

What the checks cost, read off the same commit as the names above, from each
check run's own start and finish rather than from its workflow run's:

    gh api "repos/iderex/kontor/commits/fb5cf5a42eb1470b1cf620cc12f5008f245ccf00/check-runs?per_page=100" \
      --jq '.check_runs[] | select(.status=="completed") | [.name, .started_at, .completed_at] | @tsv'

Forty eight completed check runs under twenty six names, because most of the
names on that commit were produced twice and the section after this one is about
that duplication. Every one of them finished between two and thirty nine
seconds. The longest is `Run the suite that needs a database` at thirty nine,
with `Build both layers` next at twenty seven, and the three test proofs behind
them in the low twenties. The shortest are the text scans and the type system
escape scan, which install nothing and finish in four. Nothing in this set is
slow to run, and the group that grew the set since the earlier revision did not
change that.

That is not necessarily what a merge waits for. A workflow run's own timestamps
include the time it spends queued before a runner takes it, and the run level
figure is therefore execution plus queue rather than execution:

    gh api --paginate "repos/iderex/kontor/actions/runs?per_page=100&status=completed" \
      --jq '.workflow_runs[] | [((.updated_at|fromdateiso8601) - (.created_at|fromdateiso8601)), .name, .created_at] | @tsv' \
      | sort -rn | head -3
    21992   text-determinism   2026-08-06T16:25:04Z
    21980   format-and-lint    2026-08-06T16:25:04Z
    21600   DCO                2026-08-06T16:25:04Z

Six hours and six minutes, for check runs that execute in under half a minute.
That is the worst this repository has recorded and it is what an earlier
revision of this section measured, when twenty nine runs of one hundred and
ninety three had been created and not started.

The same call now returns nothing waiting:

    gh api --paginate "repos/iderex/kontor/actions/runs?per_page=100" \
      --jq '.workflow_runs[].status' | sort | uniq -c
    272 completed

And on the commit the names above come from, the fifteen runs each finished
within forty two seconds of being created:

    gh api "repos/iderex/kontor/actions/runs?head_sha=fb5cf5a42eb1470b1cf620cc12f5008f245ccf00&per_page=100" \
      --jq '.workflow_runs[] | [((.updated_at|fromdateiso8601) - (.created_at|fromdateiso8601)), .name, .event] | @tsv' \
      | sort -rn | head -2
    42  tests   push
    35  tests   pull_request

So the wait is not a property of any check in this set and it is not a constant
of the repository either. It is the runner queue, it has been six hours and it
is currently nothing, and the only safe reading is that it has to be measured at
the moment the request is applied rather than quoted from here. An advisory
check on a slow queue reports just as late; it stops mattering, which is a
different property from being fast, so a queue is never the argument for making
one of these advisory. Whoever adds the first context should run the status
command above first, and the procedure at the end of this document is where that
belongs.

Flakiness now has one data point where the earlier revision had none, and it is
worth more than the reassuring sentence it replaces. Across the whole history
there is exactly one red:

    gh api --paginate "repos/iderex/kontor/actions/runs?per_page=100" \
      --jq '.workflow_runs[].conclusion' | sort | uniq -c
      1 failure
    271 success

The success count moves with every push and is not the figure to read. The
figure is the one on the other line, and it is the whole of the red this
repository has recorded.

    gh api --paginate "repos/iderex/kontor/actions/runs?per_page=100" \
      --jq '.workflow_runs[] | select(.conclusion=="failure") | [.name, .event, .head_sha[0:7]] | @tsv'
    tests   push    7d15599

    gh api "repos/iderex/kontor/actions/runs/31141202682/jobs" --jq '.jobs[] | [.name, .conclusion] | @tsv'
    Reject a test that reaches for what the default suite cannot give it     success
    Run the unit suite where there is nothing to reach                       failure
    Run the unit suite                                                       success
    Prove the seal bites                                                     failure
    Hold the coverage floor                                                  success
    Prove the marking bites                                                  success
    Run the suite that needs a database                                      success

Two of the seven test contexts went red on one commit, and they are the two
about the sealed environment. The commit is reachable from the default branch,
its successor on the same branch is green, and every run of that workflow before
and after it succeeded. That reads as a check catching the thing it was written
for on the commit that introduced it, rather than as a check that fails at
random, and it is the strongest evidence in this document that the sealed pair
bites. It says nothing about the other twenty three, which have still never been
observed red, and a check that has not been given the chance to fail is not the
same as one that does not.

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

`Hold the coverage floor` deserves the same honesty from the other direction. It
is deterministic and bounded and it passes the rule, but its verdict is a
threshold rather than a property, so it is the one context in the set that can
be satisfied by moving the number instead of by fixing the change. That is a
review problem rather than a reason to make it advisory, and it is named so that
the first request to lower the floor is recognised as what it is.

The other twenty three pass all three tests on the evidence available. None is
proposed as advisory.

## The thing that had to be settled before any of this is applied

It is settled, and the state that made it a blocker is kept here rather than
replaced, because a document that quietly rewrites what it once measured is a
document nobody can check against.

What it was. Every workflow in this tree that triggered on `push` did so for
every branch and also triggered on `pull_request`, so a commit that was both
pushed and in a pull request was judged twice under each name:

    S=fb5cf5a42eb1470b1cf620cc12f5008f245ccf00
    gh api "repos/iderex/kontor/commits/$S/check-runs?per_page=100" --jq '[.check_runs[].name] | length'
    48
    gh api "repos/iderex/kontor/commits/$S/check-runs?per_page=100" --jq '[.check_runs[].name] | unique | length'
    26
    gh api "repos/iderex/kontor/commits/$S/check-runs?per_page=100" \
      --jq '[.check_runs[].name] | group_by(.) | map(select(length == 2)) | length'
    22

Forty eight runs under twenty six names, and twenty two of those names are
carried by two runs each. The four that appear once are the three workflows that
trigger on the pull request only, meaning sign off, dependency review and the
workflow audit, together with the code scanning run.

This has got worse rather than better since it was first written, when the same
three numbers were twenty two, thirteen and nine. Every gate that landed in
between doubled with the rest, so twenty two of the twenty five names requested
above are ambiguous in exactly this way.

Requiring a context whose name is carried by two runs of two different events is
a request whose meaning depends on how the platform resolves the duplicate, and
this document did not know how it resolves it. Two routes closed that, and only
one of them was available from inside the tree. Reading the platform's behaviour
means applying a required check in order to find out what applying it means,
which is a repository setting rather than a change to any file here. Removing
the duplication by deciding which event a gate belongs to is the other, it
belonged to #6, and it is what happened.

What it is now. Six workflows trigger on the pull request event and on a push to
the default branch alone, which is the shape `zizmor.yml` already had. Read
against `16b7ac4d24ed89fa7bee9f5dfc8d026ea21dcc24`, the head of the pull request
that made the change, with the same three calls as above:

    S=16b7ac4d24ed89fa7bee9f5dfc8d026ea21dcc24
    gh api "repos/iderex/kontor/commits/$S/check-runs?per_page=100" --jq '[.check_runs[].name] | length'
    30
    gh api "repos/iderex/kontor/commits/$S/check-runs?per_page=100" --jq '[.check_runs[].name] | unique | length'
    30
    gh api "repos/iderex/kontor/commits/$S/check-runs?per_page=100" \
      --jq '[.check_runs[].name] | group_by(.) | map(select(length > 1)) | length'
    0

Thirty runs under thirty names, and no name carried by more than one. The third
call is widened from `length == 2` to `length > 1`, because the old form counted
exactly the duplication it was written for and would report zero for a name
carried by three runs as well as for one carried by one.

The set is larger than the twenty five named above, and this document does not
restate it. Four of the difference are the pull request body checks, and the
fifth is the code scanning run that section already places outside the requested
set. Whoever applies the request reads the first command of this document
against the commit in front of them rather than against either list here.

What this does not settle. Nothing is required of a merge yet, and none of the
arithmetic above changes that. It removes the reason a reader was given for not
applying the request, and it leaves the applying to the section below and to the
sentence at the end of this document, which still says what it says.

One property is given up by the change and is named rather than left for
somebody to notice. A branch that is pushed and never becomes a pull request is
no longer judged by these six. What that bought was a verdict on work that
cannot reach the default branch, since pushing to it is refused by the ruleset
at the top of this document, and what it cost was every one of those names being
ambiguous.

## How a check joins the required set later

Four steps, in this order, so that the set can grow without this document being
rewritten. The set has already grown by thirteen names since this document was
first written, which is why the steps exist and why this revision does not treat
its own list as settled.

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
