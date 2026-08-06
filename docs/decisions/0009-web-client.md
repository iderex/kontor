# 0009. The web client: no framework, no bundler, one component per field type

Status: accepted
Issue: #62
Supersedes: nothing
Superseded by: nothing

## The decision

A single page application, rendered in the browser, driven by the description
document of `docs/decisions/0006-api-shape.md` and by nothing written per object.

No framework. The components are custom elements, which is the browser's own
component model, and the rendering is a small typed helper that produces DOM.

No bundler. The client ships as ES modules, served by the instance the operator
is already running.

The build is `tsc` and nothing else, which is the workspace that exists today.

Routing is the History API against a route table derived from the description
document, so an object the operator created has a URL without anybody adding
one.

State is one store per instance holding the description document, the current
identity and its permissions, and a cache of records by identifier. Views read
from it and never from each other.

Nothing is rendered on the server at first release.

## Why no framework

The interface cannot be written in advance. Objects and fields belong to the
operator, so a list view, a record page and a filter builder are generated from
the description document at run time. What that leaves is a small set of
components chosen by field type and a lot of data flow, which is the part of an
application a framework helps with least.

Against that sits what a framework costs here. It is a dependency the operator
runs, on a product whose claim is that it runs on their machine and phones
nowhere. It is a version treadmill on a project that intends to be maintained
slowly. Its rendering model has to be reconciled with an interface whose shape
arrives at run time, which is the case every framework's tooling is worst at,
because the types are not known when the code is compiled.

The rejected alternative is React, and it is rejected on cost rather than on
merit. It would give the component ecosystem, the hiring pool and a settled
answer for every question below, and those are real. What it takes is a
dependency tree that is the largest thing in this repository by an order of
magnitude, a rendering runtime shipped to every operator, and a build that needs
a bundler to exist at all. A CRM interface that is mostly generated tables and
generated forms is the case where that trade is worst.

Vue and Svelte were considered together and rejected for the same reason with
smaller numbers. Svelte in particular removes the runtime argument, and what it
leaves is the compiler, which is a build system in front of `tsc` and therefore
the thing `docs/decisions/0001-means.md` argues against for the server for
reasons that apply here unchanged.

Custom elements are the answer that costs nothing to run: they are in the
browser, they have no version, and a component written against them is still a
component in ten years.

## Why no bundler

The instance is on the operator's own machine or their own network. There is no
content delivery network, no cold cache of anonymous visitors, and no round trip
across an ocean. Modules are served by the same process that answers the API, to
an authenticated user who will keep the tab open all day.

That removes the reason bundlers exist. What is left is a build step nobody has
to maintain, a supply chain that is not there, and a stack trace that names the
file the code is in.

The rejected alternative is a bundler, with the honest reason to want one: a
first paint that fetches many small modules is slower than one that fetches a
few large ones, and that matters over a slow link. The answer is that the mobile
milestone owns the slow link case, which is why it is a milestone rather than a
responsive layout, and that the budget below is what would say the answer had
stopped being true, on the day something measures it.

## How the metadata reaches the interface

The client fetches the description document after authenticating and before
rendering anything that names a field. It is one request, it is cached in the
browser's storage under the description version it was fetched at, and every
later start revalidates it rather than refetching it.

The version is the whole mechanism, and `docs/decisions/0006-api-shape.md` is
where its behaviour is decided rather than here. The client sends the version it
holds with every request. Where the instance has only gained fields since, the
request is served and the response carries the current version, so the client
learns it is behind on its next successful call rather than on a failure. Where
the request names a field that has been removed or narrowed, the request is
refused and the error names the field, the change and the current version. The
client's job in both cases is the same: take the version out of the response,
notice it is not the one it holds, and fetch the description again.

What the interface does while it is stale is stated rather than left to
behaviour. A view that is open keeps rendering with the description it started
with and shows that a newer one exists. It does not reshape itself underneath
somebody who is typing. A save against a stale description is not refused by the
client: it is sent, and the server refuses it if the field is gone, because the
client's copy is a cache and the server is the authority.

An instance whose description cannot be fetched does not render a partial
interface. It says the instance is unreachable and offers a retry, which is a
worse first screen and a better one than a list of fields that are missing for a
reason nobody can see.

## Where the client's types come from, and where they stop

`docs/decisions/0001-means.md` sends the client half of this question here, so
this record answers it.

The fixed core of `docs/decisions/0006-api-shape.md` is the half a compiler can
hold. It never changes shape between instances, so its types are generated from
the server's own description of it and committed, and a server change that
breaks the client fails on a machine rather than in somebody's browser. Written
twice is the thing being avoided: a hand written copy of the core agrees with
the server until the day it does not, and nothing says which day that was.

The generated half cannot be typed at compile time and this record does not
pretend otherwise. An operator's objects and fields exist only on their
instance, so the strongest statement a compiler can make about them is the shape
of the description itself, meaning that a field has a type, validation rules and
the three flags below. What the field is called and what its type is are data at
run time. That boundary is where the client's type safety stops, and every
component below it is written to be wrong safely rather than to be proved right.

Neither half exists today. `client/packages/app/src/main.ts` exports nothing,
the server serves no description, and no generated file is committed. #34 is
where the description becomes something a machine produces, and the type gate
that would check the generated core against the client is the third condition of
#4, which is open.

## One component per field type

A field type maps to exactly one component, and that component knows three
things: how to display a value, how to edit one, and how to filter on one.

A record page is then the field list from the description document, in order,
each field rendered by its type's component. A list view is the same choice made
per column. A filter builder is the same choice made per criterion. There is no
page written for the deal that is not also the page written for an object an
operator created this morning, and that is the property this decision exists to
keep.

An operator's custom type has to supply the same three things on the client, and
on the server the things the description document carries for every field under
`docs/decisions/0006-api-shape.md`: its type, its validation rules, and whether
it may be filtered, whether it may be sorted, and whether it may be used as an
aggregation group key. Underneath that it is a column, which is what
`docs/decisions/0003-custom-fields.md` decides and what makes the three flags
answerable rather than aspirational. A type that supplies only display renders
as read only rather than as an editable field that silently discards what was
typed.

The set of built in types is not listed here, because a list in a document
drifts against the thing it describes. The description document carries it and
#34 publishes it.

## The browser floor

The floor is a set of features rather than a set of version numbers, because a
version number is a proxy that goes out of date and a feature is what the code
actually needs.

    custom elements, version 1
    ES modules, including dynamic import
    the dialog element
    the :has() selector
    structuredClone
    AbortController and AbortSignal
    Intl.NumberFormat with currency display
    Intl.DateTimeFormat with an explicit time zone

Where that floor is enforced is a shorter answer than the list, and the short
answer is that most of it is not.

The compiler has one mechanism for this and it works.
`client/packages/app/tsconfig.json` sets `target` and `lib` to `es2023`, and
code calling a language feature newer than that does not compile. Run from the
repository root, with the second call as the one change neighbour that places
the refusal with the `lib` setting rather than with something else in the line:

    printf 'export const g = Object.groupBy([1], (n: number) => String(n));\n' > groupby.ts
    client/node_modules/.bin/tsc --noEmit --strict --target es2023 --lib es2023,dom groupby.ts
    groupby.ts(1,25): error TS2550: Property 'groupBy' does not exist on type 'ObjectConstructor'. Do you need to change your target library? Try changing the 'lib' compiler option to 'es2024' or later.
    client/node_modules/.bin/tsc --noEmit --strict --target es2023 --lib es2024,dom groupby.ts

That is the compiler the workspace resolves rather than one on a path, and it
needs `npm ci --prefix client` first.

What the `lib` line does not do is hold any entry in the list above, and the
list is where it is worth being exact.

One of the eight is held, by a different setting. ES modules and dynamic import
are syntax, and `module` and `target` decide whether the code may use them.

Four are in `dom`, which carries no year and moves with whatever compiler
version the workspace resolves, so an API that entered browsers last month type
checks the moment the compiler knows about it. Two more are the `Intl` entries,
which are declared as far back as `lib.es5.d.ts` and therefore type check under
every setting this project could choose:

    lib=$(dirname "$(find client/node_modules -name lib.dom.d.ts | head -1)")
    for s in customElements HTMLDialogElement structuredClone AbortController currencyDisplay; do
      printf '%s ' "$s"; grep -l "$s" "$lib"/lib.dom.d.ts "$lib"/lib.es5.d.ts | sed "s|$lib/||" | tr '\n' ' '; echo
    done
    customElements lib.dom.d.ts
    HTMLDialogElement lib.dom.d.ts
    structuredClone lib.dom.d.ts
    AbortController lib.dom.d.ts
    currencyDisplay lib.es5.d.ts

The last is a CSS selector, and nothing in this toolchain reads a stylesheet.

Nothing holds the run time half either. A browser missing one of these features
compiles nothing and fails when it reaches the feature, and there is no startup
check that says so plainly. That check is owed rather than done, and #63 is
where the client acquires a test route that could run it.

## The bundle budget

The first paint path is budgeted at 250 KB compressed, meaning everything the
browser must fetch before an authenticated user sees their first list of
records, excluding the description document, which is instance shaped and is
therefore not a number this project can budget for an operator.

The number is a budget rather than a measurement, and nothing in the plan
measures it. There is nothing to measure yet, since the application has no
behaviour, but the gap outlasts that: #81 budgets the mobile clients on device
and network, #126 measures the server's footprint on a stated machine, and
neither of them covers the first paint path of a page served to a browser. No
open issue owns this measurement today.

A budget nobody checks is a wish, and this one is a wish until an issue is
opened for the check.

## What would reverse the server rendering decision

One thing, and it is not performance.

If the product acquires a surface that has to be readable without authenticating
and without JavaScript, server rendering follows immediately, because a single
page application cannot serve that surface at all. A public form that a customer
fills in, a shared read only report link, or a page that has to be indexed are
each that surface. None of them is in the plan today.

Performance is not the reversing condition, because the answer to a slow first
paint is the budget above and the mobile clients, and reaching for a rendering
runtime on the operator's host to fix a bundle that got too large is treating
the symptom.

The no-bundler decision has its own condition and it is measurable once anything
measures it: if the first paint path exceeds the budget above with modules
served individually, the bundler comes back before the framework does. Those are
two decisions and this record keeps them apart on purpose.

## What is not enforced

Everything above except one entry of the browser floor, which is the module
syntax the compiler settings decide.

Nothing refuses a framework being added, nothing refuses a bundler, nothing
measures the bundle, and nothing checks that a page was not written by hand for
one object. The last is the one worth naming, because it is the property this
record is really about, and the shape of a check for it is a pattern over the
client source that #113 owns.
