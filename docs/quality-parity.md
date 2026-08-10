# Quality parity: the target gate mapped onto this project

The standard this project is held to is not invented here. It is the gate that
already stands in front of the default branch of the public repository
`iderex/jellyfin-plugin-sso`, adapted to a product of a different shape. Reading
it rather than remembering it:

    gh api repos/iderex/jellyfin-plugin-sso/rulesets --jq '.[] | "\(.id) \(.name)"'
    18802863 Protect main and 5.0
    gh api repos/iderex/jellyfin-plugin-sso/rulesets/18802863 \
      --jq '{enforcement, bypass:.bypass_actors, required:[.rules[].parameters.required_status_checks[]?.context]}'
    {"bypass":[],"enforcement":"active","required":["build","ABI floor build","Package (JPRM) / Build package","Package (JPRM) / Generate SBOM","CodeQL","Analyze (csharp)","DCO sign-off","Deterministic PR-hygiene checks","Enforce greppable invariants","Reject Trojan Source Unicode","Audit workflows (zizmor)","prettier","dependency-review"]}

Parity does not mean copying thirteen names. Some of them are about building a
plugin against a host application and publishing it to a catalogue, which a self
hosted product with its own database does not do. Some have a counterpart under a
different name because the languages differ, and this project has two toolchains
where that one has one, so several entries become two here. And this project
needs gates that one does not, because it holds an entire company's customer
records, executes operator authored automation, takes untrusted bytes on an
inbound trigger route, and ships a client that stores personal data on a phone.

What this document does not do is list the checks that exist here. That list
would drift against the thing it describes. These print it:

    gh api repos/iderex/kontor/actions/workflows --jq '[.workflows[].name]'
    gh api repos/iderex/kontor/rulesets/20486686 \
      --jq '[.rules[].parameters.required_status_checks[]?.context]'
    []

The second output is the state of the gate here today and it is the reason this
document exists. A pull request is required and nothing else is. Every entry
below that says matched, replaced or added describes what should stand there, not
what does.

## The required set of the target gate

`build`. Replaced by two counterparts, and this is one of the entries the second
toolchain doubles: a server build and a client build, from the workspace #2
creates, run on every pull request under stable names by #6.

`ABI floor build`. Replaced by a floor build against the oldest database and
runtime versions this project claims to support, which `docs/decisions/0001-means.md`
states. There is no host application here, but the failure the target check
prevents is the same one: a claim of support that nothing checks. #131 delivers
it, and it was opened by this map rather than found on the board.

`Package (JPRM) / Build package`. Replaced by the container image build of #119.
The target packages a plugin for a catalogue; the artefact here is an image an
operator runs beside a database, and #119 requires that it actually runs rather
than only that it builds.

`Package (JPRM) / Generate SBOM`. Matched by #103, which publishes third party
notices and a bill of materials. The obligation is stronger here than there,
because the licence in this tree makes the notices a legal artefact rather than a
courtesy.

`CodeQL` and `Analyze (csharp)`. These two contexts are one mechanism in the
target, the second being the language specific job the first schedules. Replaced
here by the code scanning gate of #109, which is three jobs because there are
three languages: the two toolchains, and the workflow files, which are code
holding a token and are read as source rather than left to the audit that reads
them for another question. So two contexts there become three here for a
different reason, and counting them as a match on the number would hide that.
This passage said two until the gate landed, on the count of toolchains, and the
third language was in the issue's conditions before it was in this sentence.

`DCO sign-off`. Matched, and already running under exactly that name, which is
readable in the tree:

    git grep -n 'name: DCO sign-off' -- .github/workflows/
    .github/workflows/dco.yml:24:    name: DCO sign-off

#9 states the requirement in the contributor guide, which the gate's own error
message already points at.

`Deterministic PR-hygiene checks`. Matched by #132, which this map opened, and
which is now in the tree. #12 added the templates, which is the prompting half;
what reads the answer is `pr-body-scan`, under four names rather than one,
because the target's single context covers three refusals and their proof and
this tree gives each its own sentence:

    git grep -n '    name: ' -- .github/workflows/pr-hygiene.yml
    .github/workflows/pr-hygiene.yml:37:    name: Reject a pull request body missing a section the template asks for
    .github/workflows/pr-hygiene.yml:54:    name: Reject a pull request body that names no issue
    .github/workflows/pr-hygiene.yml:71:    name: Reject an issue reference that resolves to nothing
    .github/workflows/pr-hygiene.yml:95:    name: Prove the pull request body checks bite

It stays deliberately narrow, and the narrowness is in the check's own output
rather than only here: a body either carries a section or it does not, which is
checkable, and whether what is written under it is true is not. The scanner
prints that sentence on every green verdict so a tick cannot be read as the
stronger claim.

`Enforce greppable invariants`. Matched by #113. The invariant vocabulary differs,
because the shapes worth grepping for here are the ones the architecture rules
name, and #116 holds the half that a search cannot refuse.

`Reject Trojan Source Unicode`. Matched, already running under exactly that name:

    git grep -n 'name: Reject Trojan Source Unicode' -- .github/workflows/
    .github/workflows/unicode-guard.yml:23:    name: Reject Trojan Source Unicode

`Audit workflows (zizmor)`. Matched, already running under exactly that name:

    git grep -n 'name: Audit workflows (zizmor)' -- .github/workflows/
    .github/workflows/zizmor.yml:41:    name: Audit workflows (zizmor)

`prettier`. Replaced by two, the formatter check for each toolchain, both
delivered by #3. The target has one formatter because it has one language.

`dependency-review`. Matched, already running. The job carries no explicit name,
so the context is the job id, which is the arrangement the target relies on too.

The thirteen therefore account for as follows, and the arithmetic is written out
because a count is the thing a reader is most likely to take on trust. Four are
already matched by a check running here under the identical context name, which
are sign off, the Unicode guard, the workflow audit and dependency review. Two
are matched by a named counterpart that does not exist yet, which are the bill of
materials and the greppable invariants. Two become two jobs each because of the
second toolchain, which are the build and the formatter. Two are one mechanism
appearing as two contexts and are replaced by the three language jobs of #109. Two
are replaced by counterparts for a product of this shape, which are the floor
build and the image build. And one had no delivering issue at all until this map
opened #132. That is four, two, two, two, two and one, which is thirteen.

## The practices the target runs without requiring them

Mutation testing. Matched by #112, which measures the tests rather than the
coverage of them.

Fuzzing. Matched by #111, and proposed here as a merge condition rather than as a
practice beside one, for the reason in the split below.

A second static analyser with a different lens. Matched by #110.

An end to end harness. Replaced by two, because there are two client shapes: the
web client harness of #63 and the device harness of #80. #80 is named for what it
needs rather than for what it covers, which is the rule #7 sets.

A documentation lint. Matched by #115.

## What this project needs that the target does not

Each of these exists because of something this product does and that one does
not, and each names the issue that delivers it.

A headless conformance gate. The default suite runs with no display, no elevated
rights and no reachable network, and something refuses a change that breaks that.
#7 builds it and #117 proves it still holds across the whole tree once the tree
is large. The target is a plugin inside a host process and does not have this
problem in the same shape.

An outbound network gate. This project claims that nothing phones home, and a
claim of that kind is worth exactly what checks it. #97 is the test that says so.

A sync protocol conformance suite. #74. A protocol with more than one
implementation needs a definition that is executable, or the second
implementation defines it by accident.

A reporting determinism suite. #45. A forecast that gives two answers to one
question is worse than no forecast, and determinism is the property that makes
the rest of `docs/decisions/0007-reporting.md` checkable at all.

An input fuzz gate. #111, over the surfaces that take bytes from strangers: the
import parser, the API request path, the expression language and the sync
protocol decoder. The target takes configuration from an administrator; this one
takes files from whoever the operator's staff were sent them by.

Beyond the five the plan already named, four more follow from the same reasoning
and are recorded here so the list is not read as complete at five: the secret
handling gate of #91, since a self hosted product's logs and bug reports are
written by the operator rather than by this project; the audit trail of #96; the
migration review gate of #18, because a metadata change here rewrites a live
operator's database; and the restore proof of #95, since a backup nobody has
restored is a claim rather than a backup.

## Which of these are merge conditions, and why the split falls there

Nothing here is a merge condition today. The empty output at the top of this
document is the whole of the current state, and #108 is the separate request that
the branch protection actually change, since changing a repository's protection
is a maintainer action rather than something a plan performs.

The rule proposed for the split, so that it can be applied to a check nobody has
written yet rather than argued case by case.

A check is a merge condition when it is deterministic, meaning the same commit
gives the same verdict on a re-run; bounded, meaning it finishes fast enough that
nobody has a reason to want it bypassed; and unambiguous, meaning its failure
names one thing to fix. Build, format, lint, types, the unit suite and its
coverage floor, the headless gate, the greppable invariants, the Unicode guard,
the workflow audit, dependency review, sign off, code scanning, the reporting
determinism suite and the sync conformance suite all satisfy those three.

A check is advisory when it scores rather than judges, or when it samples. The
supply chain score is a score: it moves for reasons that have nothing to do with
the change in front of it, and requiring it makes an unrelated upstream event
block an unrelated pull request. Mutation testing is slow and its number moves
with the test suite as a whole rather than with the diff. A second analyser with
a different lens is valuable because its false positives are different, which is
the same sentence as saying it should not block.

Fuzzing is the one entry that is split rather than placed. A short run over the
committed seed corpus is deterministic and bounded, and it is a merge condition. A
long campaign is neither, and it runs on a schedule with its findings arriving as
issues. Counting the second as a gate would mean either a slow queue or a
timeout dressed as a pass.

## How this document is kept honest

Every entry above names an issue, so this is a plan rather than a promise, and an
entry whose issue closes without the check appearing is a visible contradiction
rather than a quiet one.

Two entries name issues this map opened, #131 and #132, which is the shape a gap
should take: the map is not allowed to say a thing is covered when nothing covers
it, and it is not allowed to leave a hole with no home either.

What the map cannot do is judge whether a counterpart is a good one. That the
question was asked for each of the thirteen is checkable, because the answer is
written down. Whether an answer is right is a judgement, and the review is where
a wrong one is caught.
