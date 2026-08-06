# Security policy

This project is built to hold the contact records, the deal history, the audit
trail and the mailbox connections of whoever runs it. A finding against it is
worth reporting, and this file says where to send one, what happens to it, and
what this project will not do.

## Report privately, not on the tracker

Use the repository's private reporting route:

    https://github.com/iderex/kontor/security/advisories/new

The route is open rather than announced, and one command says so rather than
this sentence:

    gh api repos/iderex/kontor/private-vulnerability-reporting --jq .enabled
    true

A report there is visible to the maintainer and to whoever the reporter adds,
and it carries a draft advisory, so the fix and the publication happen in the
same place as the report. There is no address to write to instead, and that is
deliberate: an address is a second route with no advisory attached to it and no
record that a report arrived.

Do not open a public issue for something you believe is exploitable. The
tracker is public, and an issue is the disclosure rather than the report of
one. If you have already opened one, say so in the private report and leave the
issue alone rather than editing it, because an edit is not a retraction and the
history stays readable.

## What a reporter can expect, and when

These are the targets a report is answered against. They are stated so a
reporter can tell a slow answer from no answer.

A report is acknowledged within five working days. The acknowledgement says a
human has read it and nothing more.

Within fifteen working days the report has an assessment: whether the finding
is accepted, what severity it is given with the reasoning behind that severity,
and either a repair or the reason there will not be one.

While an accepted report is open it gets an update at least every thirty days,
even where the update is that nothing has moved.

A reporter who wants credit is named in the advisory, and a reporter who does
not is not. Say which in the report.

Where one of these is missed, the reporter is free to say so publicly, and
saying so is not a breach of anything. The targets exist to be measured against.

## What is in scope

The code in this repository, the workflow files under `.github/`, and the
documentation where it tells an operator to do something that is not safe.

The tree today is smaller than the plan on the tracker, which changes what
there is to test:

    git ls-tree --name-only HEAD
    .gitattributes
    .github
    .gitignore
    .nvmrc
    CONTRIBUTING.md
    DCO
    LICENSE
    NOTICE.md
    README.md
    SECURITY.md
    build
    client
    docs
    rust-toolchain.toml
    server

`server/` and `client/` hold workspaces that compile and do nothing else. There
is no request path, no database, no authentication and no connector, so most of
the surface this policy will eventually cover does not exist to be attacked.
What does exist is the workflow files, the build script, the two dependency
trees the lock files pin, and the documents. A finding against any of those, such
as a way to make a workflow run attacker-controlled code or leak its token, is in
scope now.

## What is out of scope

An operator's own deployment. How somebody configures a reverse proxy, an
identity provider or a database in front of this software is theirs, and a
finding there belongs to them unless this project's documentation told them to
do it.

A scanner's output with no analysis attached. A report that a tool printed a
line is not a report that something is exploitable, and it will be closed
asking for the analysis rather than treated as a finding.

A behaviour that requires an actor who already holds the permission the
behaviour grants. An administrator being able to read the data an administrator
may read is the permission model working. Where the finding is that the
permission model itself is wrong, that is in scope and is a report about the
model.

Resource exhaustion by an authenticated caller with no limit stated yet. Rate
limits and request size limits are open work on the tracker rather than a
control that exists and fails, and a report that they are absent tells this
project what it already wrote down.

None of these are refused because they are unwelcome. They are refused because
a policy that accepts everything gives a reporter no way to know whether their
report will be acted on.

## Disclosure, and disagreement about severity

An accepted finding is published as an advisory on this repository when a fix
is available, or ninety days after the acknowledgement, whichever comes first.
The ninety day limit is not conditional on a fix existing. An advisory
published without one says what an operator can do instead.

A reporter may publish earlier than that, and this project would rather that
happened than that a reporter felt bound to silence by a policy. The request is
that the private report says so first, so the advisory and the reporter's
publication do not contradict each other on the facts.

Where the reporter and this project disagree about severity, both assessments
go in the advisory. This project states its severity and the reasoning that
produced it. The reporter's severity and reasoning are quoted rather than
summarised. Neither one blocks publication, and a disagreement is not a reason
to hold an advisory back, because a held advisory protects the disagreement and
not the operator. A reporter who thinks the published severity is wrong is free
to publish their own assessment and to say that this project disagreed.

Nothing here asks a reporter to sign anything or to wait for permission.

## Which versions get a fix

There is no release:

    gh release list --repo iderex/kontor
    (no output)

    git ls-remote --tags https://github.com/iderex/kontor
    (no output)

So there is no supported version, and a table of them here would be a table of
nothing that later goes stale as releases appear. The only ref that receives a
fix today is the default branch.

What is supported once releases exist is decided by the version policy, which
is open work rather than a settled thing, on issue #125. That issue owes the
support window and what ends it. When it lands, this section points at the
policy rather than repeating a list, because a list kept by hand in two places
drifts and the drift is invisible until somebody trusts it.

## A finding in a dependency

This project does not control its dependencies and will not pretend to.

Where a dependency has an advisory and this project is affected, this project
publishes its own advisory saying what an operator running this software has to
do about it. That advisory is about this project's exposure and it does not
restate or re-score the dependency's, which stays the upstream's to make.

Where a dependency has no advisory and the finding was made against this
project, the report goes upstream, and the reporter is told that it did. Where
the upstream is unresponsive, the repair here is to pin, to work around or to
remove the dependency, and the advisory says which of those was done.

Where this project is not affected, the answer is that it is not affected and
why, and no advisory is published. A dependency carrying a vulnerable function
that nothing here calls is not a vulnerability in this project, and saying it is
would make every future advisory worth less.

Dependency advisories are also watched by the repository itself rather than only
by hand. This file does not list the workflows that do it, because a list in a
document drifts against the thing it describes. One command prints them:

    gh workflow list --repo iderex/kontor

and the update route that is a repository setting rather than a workflow reads
out of the API:

    gh api repos/iderex/kontor --jq '.security_and_analysis.dependabot_security_updates.status'
    enabled

Neither of those is a substitute for a report. They cover published advisories
against pinned dependencies, and they cannot see a finding nobody has published.
