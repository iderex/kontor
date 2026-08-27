# The schema file

One object per file, one key and one value per line. It is what
`server/crates/metadata/src/text.rs` writes and reads, and this document is the
grammar rather than a description of it: where the two disagree, the disagreement
is a defect in the code.

It exists for two things #17 asks for. A schema is reviewed before it is applied,
which means a person reads a diff of it and has to be able to argue with what
they see. And a schema is applied to a second instance, which means the file has
to mean on that instance exactly what it meant on the one it came from, down to
the space at the end of a label.

## An example

    # The object a deal is kept as.
    object opportunity
    label Opportunity

    field stage
    label Stage
    type picklist
    required yes
    value Qualification
    value Proposal
    value Closed won

    field owner
    label Owner
    type reference
    required no
    references person

## The lines

A line is a key, then one space, then the value. A line that is the key alone is
that key with an empty value, and that is how an empty value is written, so that
nothing this format writes ends in a space.

A line whose first character is `#` is a comment and is passed over. A line with
no characters at all is passed over too. Neither is part of the definition, so a
file that is written back out after being read carries neither.

A line that starts or ends in a space or a tab is refused. The value is
everything after the first space, so whitespace at either end would join the
value without showing on the screen, and an editor that trims it would change
what the file means. Nothing is indented for the same reason: what looks like
structure would be a value.

## The keys

`object` opens the file and there is exactly one. The value is the identifier
everything else refers to the object by.

`label` is what a person reads on a screen. It belongs to the object before any
field, and to the field it follows after one.

`field` opens a field. The value is the identifier, which is what a rename may
not change.

`type` is one of the field types, written the way the type register writes it.
The set is closed and it is not listed here, because a list in a document drifts
against the register that decides it:

    git grep -n 'written_as:' -- server/crates/metadata/src/field_type.rs

`required` answers `yes` or `no`, and nothing else. Every other spelling of true
and false has a system somewhere that reads it the other way round.

`value` is one of the values a type that constrains its values offers. It is
written once per value, in the order they are offered and ordered in, because
that order is what a picklist sorts by.

`references` is the object a reference field points at.

A key that the block takes once and is given twice is refused rather than
overwritten. Which of the two an operator meant is not something a reader could
work out, and taking the later one quietly is the mangling this format exists
against.

## The escapes

Five, and no others.

    \\   a backslash
    \n   a line ending
    \r   a carriage return
    \t   a tab
    \s   a space

A value may hold anything an operator can type, including the line ending that
would otherwise end the line it is written on, so the four control characters
above are always written escaped.

The space is the one worth reading twice. Only a space at the start or the end of
a value is written `\s`; a space inside one is left alone, because a line's own
edges are the only place a space is lost and escaping every space would make an
ordinary label unreadable. Reading accepts `\s` anywhere.

A backslash followed by anything else is refused, and so is a line that ends on
one. Passing an unknown escape through as whatever character followed it is how a
format quietly eats the backslash in a path.

## What reading does not do

Reading refuses the shape of the text and never the meaning of it. A file can be
read successfully and still hold a definition nothing could realise: an
identifier that is not one, a picklist offering no values, more fields than one
table has room for. Those rules live in
`server/crates/metadata/src/definition.rs` and are not repeated in the reader,
because a second copy of a rule is a statement that drifts.

So applying a file is two steps rather than one. Read it, then check what came
back. A caller that skips the second step has skipped it visibly, and a schema
being reviewed is exactly the case where the file has to be able to carry the
definition somebody is asking for help with.

Every rule a file breaks comes back at once rather than the first one, for the
reason the definition rules already give: an operator fixing a file one refusal
at a time makes one round trip per mistake.

## What this format is not

It is not a general configuration format and nothing else in this tree reads it.
It carries one object definition, which is the thing #17 asks to round trip, and
a second use for it is a reason to argue about it again rather than to widen it
quietly.

It carries no comment written by an operator across a round trip. Comments are
read and passed over, and a file written out from a definition has none, because
a definition does not hold them. Whether a schema file kept by hand should
survive being rewritten by the instance is a question about how schemas are
managed rather than about this grammar, and it is not answered here.
