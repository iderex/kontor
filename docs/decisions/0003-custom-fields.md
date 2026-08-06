# 0003. A custom field is a column

Status: accepted
Issue: #14
Supersedes: nothing
Superseded by: nothing

## The decision

A custom field is a real column on the real table of the object it belongs to,
created by a migration the system generates. A custom object is a real table.
Nothing is stored as a row in a generic table of record, field and value.

The metadata layer of #17 holds what the fields are. The generated migration of
#18 is how the database is brought to match. Neither of those is this record;
this one says which of the two shapes they are working towards, and what the
shape costs.

## The alternative, and what it costs

The other shape keeps custom values in one table with a row per value, roughly
`(record, field, value)`. It is easy to build, it needs no migration, and an
operator can add a field with an insert. Most products in this field are built
that way, and the reason is that the first version of it takes an afternoon.

What it costs is the whole of M4.

A sum over one custom field becomes a join against the value table filtered to
one field. A report over five custom fields becomes five of them, and the query
planner has no useful statistics for any because every row in that table is the
same shape and the distribution it sees is the distribution of all fields mixed
together. Grouping by a custom field over a million records is a query nobody
can make fast without caching the answer. A cached answer cannot be asked as of
last Tuesday, and asking as of last Tuesday is what #37 exists for.

The type is the second cost and it is quieter. A value column holding every
field's value is text, or it is a union of columns of which one is set. The
database cannot then refuse a date that is not a date, cannot index a number as
a number, and cannot order a currency amount without casting every row. Every
one of those becomes an application rule, which is a rule that holds until one
code path forgets it.

With a column per field, none of that is true. A query over a custom field is an
ordinary query. The database can index it, the planner can see its statistics,
and #39 can push its aggregation down into the database rather than pulling rows
out to add them up.

## What the decision costs

Adding a field becomes a schema change on a live database, and the honest list
of what that implies is:

A lock is taken, however briefly, on a table that other transactions are using.

A migration can fail halfway, which a row insert cannot.

The database has a hard limit on columns per table, which a value table does not,
and the limit is reachable by an operator who is using the product as intended.

Those are known problems with known handling, and the rest of this record is that
handling. They are the problems this project would rather have, because each one
is visible at the moment it happens rather than six months later in a report
nobody can reconcile.

## The bound on fields per object

The bound is 250 custom fields per object.

PostgreSQL allows at most 1600 columns on a table, and the limit counts a
dropped column too: dropping a column marks its slot dead and does not return it
until the table is rewritten. So a bound has to leave room for the core columns
of the object, for the churn of an operator adding and removing fields over
years, and for the slots that churn consumes permanently.

    core columns per object          roughly 20
    live custom fields at the bound             250
    remaining slots for dead ones     1600 - 270 = 1330
    full turnovers of a bounded object       1330 / 250 = 5.3

Five complete replacements of every custom field on an object before the table
has to be rewritten to continue. That is the number the bound is chosen for, and
it is arithmetic on a documented database limit rather than a measurement of
this product, which does not exist yet.

The bound is enforced in the metadata layer at the point a field is created,
before any migration is generated, and the refusal names the bound, the count
this object is at, and how many dead slots the table is carrying. A bound whose
message says only that a limit was reached teaches an operator nothing about
what to do next, and what to do next is different depending on which of the two
numbers is the one that ran out.

The bound applies per object, not per instance. An operator with 40 objects may
hold 10,000 custom fields, and nothing here bounds that.

## Deleting a field

Deletion is two steps and the steps are days apart.

The first step hides the field. The column stays, the data stays, and the field
stops appearing in list views, record pages, the API description and new
reports. A report that already names the field keeps working and says the field
is hidden.

The second step drops the column, and it happens no earlier than 30 days after
the first, as a separate act that the operator confirms. Until it happens,
restoring the field is one metadata change and no data is lost.

That window is the same shape as the record deletion of #20 and the same number,
and where the two differ this record is wrong rather than #20.

Two things about the second step are worth saying plainly. Dropping the column
destroys the values in it and no restore brings them back short of a database
restore under #95. And the dropped column keeps its slot against the 1600 limit
until the table is rewritten, which is why the refusal message above counts dead
slots separately from live fields.

The change log of #15 is not touched by either step. Entries about a dropped
field stay in the log and remain readable, because a history that quietly loses
the fields somebody deleted is a history that answers a past question with
today's schema, which is exactly what #37 may not do.

## Changing a field's type

A type change that PostgreSQL can make without rewriting the table is applied by
a generated migration. Widening a `varchar` bound and changing between two types
the database considers binary coercible are the cases that qualify.

Every other type change is refused, with a message naming the current type, the
requested type, and the reason the database would have to rewrite the table.

The refusal is deliberate and it is not a limitation to be lifted later. A
rewrite of a large table under `ACCESS EXCLUSIVE` is an outage of that object for
as long as the rewrite takes, and the operator asking for the change has no way
to know in advance how long that is. Worse, a type change that reinterprets data,
such as text to number, either loses the rows it cannot convert or fails in the
middle, and both outcomes are discovered after the fact.

What the operator does instead is create a field of the new type, move the data
with a bulk operation under #31 that reports what it could not convert, and
delete the old field through the two steps above. It is more work and every step
of it is visible.

## What a generated migration may do to a lock

The maximum this project accepts is `ACCESS EXCLUSIVE` held briefly, taken with a
lock timeout, on the one table being changed.

Concretely, every migration generated by #18:

Sets `lock_timeout` before it starts, and fails rather than queueing. A migration
waiting on `ACCESS EXCLUSIVE` blocks every later reader of that table behind it,
so a migration that waits turns a slow transaction somewhere else into an outage
here. Failing is recoverable; the queue is not.

Adds a column with no default, or with a constant default, both of which
PostgreSQL records in the catalogue without touching the rows. A default that is
not constant is refused, because it rewrites the table.

Builds any index for the new field with `CREATE INDEX CONCURRENTLY`, outside the
transaction, which does not block writes.

Adds a not-null constraint as a `NOT VALID` check first and validates it in a
second statement, which takes a weaker lock, rather than as a column constraint
that scans the table under the strong one.

Touches one table. A migration that has to touch two is two migrations, so a
failure of the second leaves the first complete rather than leaving the operator
inside a partial change nobody described.

Nothing in this repository enforces any of that today, and no check reads a
migration, because no migration exists. #18 is where these become the properties
its generator is tested against, and it is the issue this paragraph is owed by.

## The escape hatch above the bound

An operator who needs more than 250 fields on one object creates a custom object
holding the extra fields and relates it one to one to the first.

That is a real table with real columns, so it keeps every property this record
is about: the reporting engine can group by those fields, the database can index
them, and the types are types. The cost is a join in queries that span both,
which is an ordinary join on a key rather than the join per field the value table
shape would have made.

It is deliberately not a smooth path. An object with more than 250 fields is
usually several objects that have not been separated yet, and the moment somebody
has to make the second object is the moment that question gets asked.

## What would reverse this

One measurement reverses it: a schema change on a table of representative size
that cannot be made inside the lock budget above, on the PostgreSQL floor this
project pins, using the statements listed above. If adding a column to a
50,000,000 row table cannot be made metadata-only, the argument for columns loses
its foundation, because the operator's alternative to a fast schema change is
downtime and the alternative to downtime is the shape this record rejects.

The measurement belongs with #126, which measures the footprint on a stated
machine, and it has not been made. Nothing in this record is measured. Everything
in it is either arithmetic on a documented database limit or a property of
PostgreSQL that the migration generator of #18 will be tested against.

Two things that are not reasons to reverse it, because both will be argued.

An operator hitting the bound is not one. That is the escape hatch above, and
the bound is a consequence of the database rather than of this decision.

A migration that failed is not one either. A migration that fails and says so is
this decision working. The value table shape has no failure to see, which is not
the same as having none.
