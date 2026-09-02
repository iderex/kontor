# The expression language, and the door it closes

A workflow needs to compute. It compares a field against a value, adds days to a
date, works out whether an amount crossed a threshold, and builds the text of a
task. That is a language, and the moment one exists somebody asks it to fetch
something.

This document defines it and closes that door on purpose, before there is an
implementation to argue with. `docs/decisions/0008-workflow.md` states that the
action set is closed, that no action runs arbitrary code, and that this language
is the bounded exception, bounded in the one direction that matters: it computes
over the record and the trigger entry, and it may not call out. What follows is
that sentence written out far enough to build from and to disagree with.

It is the first done condition of #55. The other six are about an evaluator, and
there is none.

## What an expression is for, and where one appears

An expression appears in exactly two places in a workflow: as the condition on a
branch or a trigger, where it has to produce a true or false; and as the value of
a declared input to an action, where it has to produce the type that input
declares.

It is never a statement, never a program and never a script. There is no
sequence, no assignment, no loop, no recursion and no way to define a function.
An expression is one finite tree evaluated once, and the sections on bounds
below are what that buys.

## What an expression can see

Three roots, and nothing else resolves to anything.

**`record`.** The record the triggering entry is about, with its fields reached
by the identifier the metadata layer holds, so `record.amount` and
`record.close_date`. That identifier is the one `docs/model.md` and
`server/crates/metadata/src/definition.rs` call `name`, which is what a rename
may not change, so an expression does not break when somebody relabels a field.

**`change`.** The change log entry that triggered the run, with the columns
`docs/decisions/0004-change-log.md` declares: `change.field`, `change.old`,
`change.new`, `change.actor`, `change.actor_kind`, `change.cause`,
`change.cause_ref` and `change.occurred_at`.

**`run`.** `run.started_at`, which is the moment the run began, and
`run.identity`, which is the principal the run holds under
`docs/permission-model.md`.

There is no fourth root. In particular there is no way to reach another record,
a list of records, an aggregate, a count, or anything the database holds beyond
the one record and the one entry above.

That absence is the load bearing one and it is refused for two reasons rather
than one. Reading another record is a read, so it is a permission question, and
an expression that could perform one would be a second place where
`docs/permission-model.md` is evaluated, which is exactly the copy that model
exists against. And a language that can walk from a record to a set is a query
language over a live database, which `docs/decisions/0006-api-shape.md` refuses
for the API on the grounds that it is a denial of service surface and a
permission bypass surface at the same time. Neither reason gets weaker for the
caller being a workflow rather than a person.

## The grammar

Written as a grammar rather than as a description, because the second condition
of this issue is about what the parser accepts and a description is not
checkable against one.

    expression   = if | or
    if           = "if" expression "then" expression "else" expression
    or           = and { "or" and }
    and          = comparison { "and" comparison }
    comparison   = sum [ ( "=" | "<>" | "<" | "<=" | ">" | ">=" ) sum ]
                 | sum ( "is absent" | "is present" )
    sum          = product { ( "+" | "-" ) product }
    product      = unary { ( "*" | "/" ) unary }
    unary        = [ "not" | "-" ] primary
    primary      = literal | path | call | "(" expression ")"
    path         = root { "." identifier }
    root         = "record" | "change" | "run"
    call         = identifier "(" [ expression { "," expression } ] ")"
    literal      = number | text | money | date | moment | "true" | "false"
                 | "absent"
    identifier   = letter { letter | digit | "_" }

`identifier` in a `call` has to be one of the functions named below. There is no
dynamic lookup: a name that is not in that table is refused when the workflow is
published rather than resolved at run time, which is what makes the absence of a
call out structural rather than a rule somebody enforces.

`identifier` after a `.` has to be a field of the object the trigger is on, or
one of the columns of the change log entry, or one of the two members of `run`.
A path of any other shape is refused at publication too.

## The types

Eight, and the first seven are the field types of
`server/crates/metadata/src/field_type.rs` collapsed onto what a comparison can
mean.

**`text`** carries what a `text`, `long_text`, `email`, `telephone`,
`web_address` or `picklist` field holds. **`whole number`** carries a
`whole_number`. **`money`** carries a `money`, being an amount and the currency
it is in, which travel together and never separately, under
`docs/decisions/0005-money-and-time.md`. **`date`** carries a `date`, with no
time and no zone. **`moment`** carries a `moment`. **`true or false`** carries a
`true_or_false`. **`reference`** carries a `reference`, and the only things it
supports are equality against another reference and the absence tests.

**`duration`** is the eighth and no field has it. It is what the difference of
two dates or two moments is, because that difference has to be something: for
dates it counts whole days, for moments it counts seconds, and the two are not
interchangeable. It has no literal form. One is built by subtracting, or by
`days` and `seconds` in the function table below, and never in a calendar unit,
because a month is not a duration and a language that offers one as though it
were is wrong twice a year.

A picklist is `text` to the language and not a type of its own, and the
comparison against it is checked more strictly than the type: comparing a
picklist field against a literal that is not one of the values that field
declares is refused at publication. A workflow whose condition can never be true
because somebody renamed a stage is the failure that catches.

## What the operators mean

Comparison is within one type. There is no coercion anywhere, in either
direction, so a date is never a number and a number is never a text.

`=` and `<>` are available on every type. The ordering comparisons are available
on `whole number`, `money`, `date`, `moment` and `duration`, and on `text` by the
collation `FIELD_TYPES` states for a text field, so that an expression orders
text the way a list view does. They are not available on `true or false` or on
`reference`, because neither has an order anybody would agree on.

Arithmetic:

    whole number + whole number   whole number
    whole number - whole number   whole number
    whole number * whole number   whole number
    whole number / whole number   whole number, truncated toward zero
    money + money                 money, same currency only
    money - money                 money, same currency only
    money * whole number          money
    date + whole number           date, the number being days
    date - whole number           date
    date - date                   duration in days
    moment + duration             moment
    moment - duration             moment
    moment - moment               duration in seconds
    duration + duration           duration
    duration - duration           duration

Everything not in that table is refused at publication. Three absences are
deliberate and each is refused for a reason rather than for want of writing it.

**`money / whole number` is not there.** Dividing minor units leaves a remainder
and no rule here says where it goes, so a language that offered it would be
taking a rounding decision invisibly, in the one place
`docs/decisions/0005-money-and-time.md` says a report becomes unable to reconcile
with itself. Proportions of money belong to the reporting engine, where #41 and
#43 own them and state their rule.

**`money * money` is not there**, because the product of two amounts is not an
amount and no unit in this product is one.

**`date + date` is not there**, for the same reason.

Every arithmetic result stays inside the range its type holds, which for a whole
number and for money is the checked `i64` the money crate already refuses to wrap
on. A result that would leave the range is an error value under the section
below, not a wrap and not a panic.

## Currency, and the half of the fourth condition a type cannot give

The fourth done condition of #55 is that money arithmetic follows the money
decision and that adding two amounts in different currencies is a **publish
time** error. Half of that is deliverable and half is not, and this document says
which half rather than letting an implementation discover it.

A money field declares no currency. The type is an amount and a currency
together, per value, and `FieldDefinition` carries no currency:

    git grep -n -A 16 'pub struct FieldDefinition' -- server/crates/metadata/src/definition.rs

so `record.amount + record.other_amount` has two currencies that are data rather
than type, and no reading of the definitions can tell whether they agree. What
the publication check can refuse, and does: an operation between `money` and any
other type; and an operation between two literals whose currencies differ, since
a literal names its currency.

What is left is a run time refusal, and it is the shape the sixth condition
already asks for: an addition across two currencies produces an error value a
branch can handle, carrying both currency codes. The type level absence that
`docs/decisions/0005-money-and-time.md` relies on is what makes this safe rather
than merely reported, because the evaluator reaches the same `try_add` the crate
exposes and there is no addition available to it that does not refuse.

    git grep -n 'pub fn try_add' -- server/crates/money/src/lib.rs

So the door the fourth condition wanted shut at publication is shut at
publication for every case a definition can see, and shut at run time with a
handleable value for the case only a record knows. That is weaker than the
condition as written, and it is weaker because of a property of the data model
rather than because of an implementation choice.

## Absence

A field with no value in it is `absent`, and so is `change.old` on a creation,
which `docs/decisions/0004-change-log.md` distinguishes from a field that existed
and was empty.

Every operator and every function applied to an `absent` operand produces
`absent`. The three exceptions are `is absent`, `is present`, and `=` and `<>`
against the `absent` literal, all of which produce a true or false.

The rule is stated in one place and applies everywhere because the alternative is
worse than either of its options. A language that treats absence as a zero, an
empty text or a false makes a condition quietly true for the records nobody
filled in, and a language that decides absence per function makes a reader check
each one. Propagation means a condition over a missing field is `absent`, and a
branch condition that evaluates to `absent` does not take the branch, which is
stated here rather than left to the engine.

## The functions

Text: `lower`, `upper`, `trim`, `length`, `contains`, `starts_with`,
`ends_with`, `join`.

Whole number and money: `absolute`, `minimum`, `maximum`.

Duration: `days` and `seconds`, each taking a whole number and returning the
duration of that many, which is the only way one is written down.

Date and moment: `day_of`, which takes a moment and returns the date it falls on
in the instance's zone, and `start_of_day`, which is the inverse. Both are the
functions `server/crates/money/src/lib.rs` already holds, so the boundary between
a date and a moment has one implementation rather than two.

Conversion, and only in the safe direction: `text_of`, which renders any value as
text for a message somebody reads. There is no function that parses a text into a
date, a number or an amount, because that is where a language acquires a locale
and every argument about which one.

The table is closed. A function is added to it in a change to this document and
to the register the evaluator reads, in the same commit, for the same reason
`docs/decisions/0008-workflow.md` closes the action set.

## Bounds

**With no loop, no recursion and no function definition, an expression's cost is
bounded by its own size.** That is the first bound and it is structural: the
grammar above cannot express a term whose evaluation visits a node twice.

Two limits stand behind it anyway, because a function's cost can be in its
arguments rather than in the tree. `join` over a long text is the case: the
expression is small and the value is not.

- A **step limit**, counted in evaluated nodes.
- A **time limit** on one evaluation.
- A **length limit** on any text a function produces.

All three are instance configuration under #90, held by the instance and never by
the caller, and exceeding any of them refuses the evaluation with a message
naming which limit was reached and what it is set to. That message is what the
second condition of this issue asks for, and naming the limit rather than saying
the expression was too complicated is the difference between an operator who can
fix it and one who cannot.

A fourth bound is at publication rather than at run time: an expression is
refused if its text or its tree depth exceeds a stated maximum, so a run time
limit is never the first thing an author hears about.

## Publication, and what is checked there

The third condition of this issue is that every expression is type checked
against the metadata layer when the workflow is published, so that a comparison
between a date and a number fails at publish rather than at run time.

What is checked at publication:

- Every path resolves, against the object the trigger is on, against the change
  log entry's columns, or against `run`.
- Every call names a function in the table above, with the right number of
  arguments and the types that function declares.
- Every operator has operands of the types the table above gives it.
- The whole expression has the type its position requires: a true or false for a
  condition, and the declared type of the input for an action's input.
- A picklist comparison names a value that field declares.
- Two money literals in one operation are in one currency.
- The expression is within the publication bounds above.

What is not checked at publication, listed rather than left to be met: whether
two money values in a record share a currency, which is the section above;
whether an arithmetic result stays in range; whether a division has a zero
divisor; and whether the record has a value in any field at all. Each of those is
data rather than definition, and each has an error value below.

Publication is the moment #50 defines. A published workflow keeps the definitions
it was checked against, so a field removed afterwards does not change what a
running instance computes, and republishing is what re-runs these checks.

## Determinism and the clock

Given the same record, the same entry and the same run, an expression produces
the same value.

Nothing reads the wall clock. `run.started_at` is the run's own moment, taken
once when the run begins and passed in, so two evaluations inside one run agree
and a replay of a run computes what the run computed. There is no random source,
no counter, no ordering that depends on how a value arrived, and no function
whose result depends on anything outside its arguments and the instance's zone.

The zone is the one instance configuration an expression depends on, and it is
the one `docs/decisions/0005-money-and-time.md` already names for period
boundaries. It is read from the instance rather than from the caller, so an
expression cannot be made to compute in another zone by whoever triggered it.

## Errors are values

The sixth condition of this issue is that an error at run time is a value a
branch can handle rather than an exception that kills a run without explanation.

An evaluation produces a value, `absent`, or an **error value**. An error value
carries a kind and enough detail to act on, it propagates through every operator
the way `absent` does, and `is error` tests for one. A branch condition that
evaluates to an error value does not take the branch and the engine records the
error against the step, which is what #59 makes readable.

The kinds are closed, and each names something the publication check above
deliberately cannot see:

    out_of_range        an arithmetic result outside what the type holds
    divide_by_zero      a whole number division with a zero divisor
    currency_mismatch   two amounts in different currencies
    too_long            a text longer than the length limit
    over_budget         the step or time limit reached

The last one is an error value rather than a refusal of the run for the same
reason as the rest: a workflow whose author wants a step to be skipped when a
budget is reached can write that, and one who wants the run to stop can branch to
a stop.

## What this document does not decide

The action set, which is #54, and what an action does with the value an
expression hands it.

Where a run's state lives, whether it resumes, and what at least once means for
an expression that was already evaluated once. That is durable execution, #56.

What a trigger is and when a run starts, which is #51 for the change log
triggers and #53 for the inbound and manual ones.

The publication moment itself and how a version is kept, which is #50. This
document says only what is checked at that moment.

Whether an expression may be evaluated over anything other than a workflow's
record and entry. It may not, and no other caller is contemplated here; a
reporting metric is a definition under #36 and is a different artefact with a
different reason to exist.

## What this document does not prove

Nothing in this tree parses a single line of the grammar above. The module that
would is comment only:

    git grep -c '' origin/main -- server/crates/workflow/src/lib.rs

so the parser, the publication check, the three limits and the five error kinds
are all requirements written against whoever builds them rather than descriptions
of anything that runs. Six of the seven conditions of #55 are about that
evaluator and this document meets none of them.

Two things adjacent to this are already true and are worth separating from it, so
that neither is read as coverage. Money arithmetic that refuses two currencies
exists as a type level absence in `server/crates/money`, proved by that crate's
own properties, and this document reaches it rather than restating it. And
`test-scan` refuses library code outside a connector that opens an outbound
connection, so the crate this evaluator will live in cannot reach the network
today; that is a fact about the crate and says nothing about a language, which is
the boundary this document is otherwise about.

The seventh condition, that the language is fuzzed, belongs to #111 and needs the
parser. A grammar is what a fuzz target is written against, so it moves that
issue's means question and not its block.
