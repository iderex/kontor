#!/usr/bin/env node
// SPDX-License-Identifier: AGPL-3.0-only
// The examples in a protocol document, judged against the shapes beside them.
//
//     sync-example-scan.js docs/sync-protocol.md
//
// A specification carries examples so that a reader sees the thing rather than
// a description of it, and an example is the part of a document that rots
// first: a field is added to a table, the example beside it is not touched, and
// from that day the document says two things. The reader who copies the example
// is the one who finds out.
//
// So the examples here are read by a machine. Every shape section is a heading,
// a table of fields, and one example, and this refuses the example that has a
// field its table does not declare, the one that omits a field its table
// requires, and the one whose `type` is not the shape it sits under. Each of
// those is a one-line edit somebody will actually make.
//
// IT JUDGES THE BYTES AND NEVER THE MEANING. That an example carries every
// field its table declares is a fact about the document. That the table
// describes what an instance would really send is not, and no reading of this
// tree decides it, because nothing in this tree speaks the protocol yet. #74 is
// where a client is driven through the whole of it against a stubbed server,
// and that is the check this one is not. A green run here says the document
// agrees with itself.
//
// THE TWO HEADINGS ARE WRITTEN HERE and that is a copy, which this tree
// otherwise avoids. It is deliberate and it is bounded: the document is the
// subject rather than the authority, so there is nothing else to read them out
// of, and the failure the copy could cause is closed by refusing a document
// that holds neither heading. A rename that this file did not follow reddens
// the run rather than passing an unjudged document.
//
// TOP LEVEL KEYS ONLY. A field whose value is an object or an array is checked
// as a field and not descended into, so a nested shape is judged where it has a
// section of its own and nowhere else. That is why the shapes that travel
// inside a message have their own sections rather than being described in the
// prose of the shapes that carry them.
//
// FAIL CLOSED IN FIVE PLACES. A file that is not there, a document holding
// neither heading, a document holding no shape under them, a document with no
// requirement list, and a fenced example that is opened and never closed are
// each refused rather than reported clean. A scanner that read an empty
// document as a clean one would turn a document somebody deleted the body of
// into a green tick, and the last of the five is the same failure arriving one
// character at a time: everything below an unclosed fence is read as part of
// the example above it, so a document could lose every rule below the middle of
// it and stay green.
//
// Node rather than the POSIX sh most scanners here are written in, and the
// reason is the input rather than a preference, which is the same reason
// finding-scan.js gives. An example is JSON, the question is which keys it
// carries, and a shell reading JSON by pattern is the defect this check exists
// against arriving inside the check. Node parses it with nothing installed and
// this tree already pins a version of it.
//
// prove-sync-examples is the evidence that every refusal above bites, against
// documents it writes rather than against this tree's own.

'use strict'

const fs = require('node:fs')

// The headings the shapes sit under. A shape under the first declares `type`
// and a shape under the second may not, because the first set is what a client
// dispatches on and the second is what it finds inside one.
const MESSAGES = '## Message shapes'
const INNER = '## The shapes inside a message'
const REQUIREMENTS = '## The requirements'

const file = process.argv[2]

if (process.argv.length !== 3) {
    process.stderr.write(
        'sync-example-scan: one document, named. It judges a protocol document against the shapes inside it, so a run with no argument would judge nothing and must not read as one that judged a document and found nothing.\n',
    )
    process.exit(2)
}

let refused = 0

function refuse(rule, where, sentence) {
    process.stderr.write(`sync-example-scan: ${rule}: ${where}: ${sentence}\n`)
    refused += 1
}

let text

try {
    text = fs.readFileSync(file, 'utf8')
} catch {
    process.stderr.write(
        `sync-example-scan: ${file} is not a file this can read. The document is the subject, so an absent one is a refusal rather than an empty judgement.\n`,
    )
    process.exit(2)
}

const lines = text.split('\n')

if (!lines.includes(MESSAGES) && !lines.includes(INNER)) {
    process.stderr.write(
        `sync-example-scan: ${file} holds neither "${MESSAGES}" nor "${INNER}", so nothing here knows where its shapes are. Refusing rather than passing a document this cannot find the subject of.\n`,
    )
    process.exit(2)
}

// One pass. A shape is a `### ` heading under one of the two headings above,
// and it collects the rows of its table and the fenced examples that follow it
// until the next heading of any level.
const shapes = []
const requirements = []

let under = null
let shape = null
let fence = null

for (let at = 0; at < lines.length; at += 1) {
    const line = lines[at]

    if (fence !== null) {
        if (line.trim() === '```') {
            shape.examples.push({ line: fence.line, body: fence.body.join('\n') })
            fence = null
        } else {
            fence.body.push(line)
        }
        continue
    }

    if (line.startsWith('## ')) {
        under = line === MESSAGES ? 'message' : line === INNER ? 'inner' : null
        shape = null
        continue
    }

    if (line.startsWith('### ')) {
        shape = null
        if (under === null) {
            continue
        }
        const name = line.slice(4).trim().replace(/^`|`$/g, '')
        shape = { name, kind: under, line: at + 1, rows: [], examples: [] }
        shapes.push(shape)
        continue
    }

    if (under === null && line.startsWith('R') && /^R\d+\. /.test(line)) {
        requirements.push({ number: Number(line.slice(1, line.indexOf('.'))), line: at + 1 })
        continue
    }

    if (shape === null) {
        continue
    }

    if (line.startsWith('```')) {
        fence = { line: at + 1, body: [] }
        continue
    }

    if (line.startsWith('|')) {
        shape.rows.push({ line: at + 1, text: line })
    }
}

if (fence !== null) {
    process.stderr.write(
        `sync-example-scan: example-not-closed: ${file}:${fence.line}: a fenced example is opened and never closed, so everything below it was read as part of it and nothing after it was judged. Refusing rather than reporting on the part that happened to be above the fence.\n`,
    )
    process.exit(2)
}

if (shapes.length === 0) {
    process.stderr.write(
        `sync-example-scan: ${file} holds a shape heading and no shape under it. Refusing rather than reporting a document with nothing in it as one whose examples all agree.\n`,
    )
    process.exit(2)
}

// A table row is `| ` + four cells. The header and the separator are the two
// rows every table opens with and neither declares a field.
function fieldsOf(shape) {
    const declared = []

    for (const row of shape.rows) {
        const cells = row.text.split('|').slice(1, -1).map((cell) => cell.trim())

        if (cells.length !== 4) {
            refuse(
                'row-is-not-a-field',
                `${file}:${row.line}`,
                `a row of ${shape.name}'s table has ${cells.length} cell(s) and a field is declared in four: the field, its type, whether it is required, and what it means.`,
            )
            continue
        }

        if (cells[0] === 'field' || /^-+$/.test(cells[0])) {
            continue
        }

        const name = cells[0].replace(/^`|`$/g, '')

        if (name === cells[0]) {
            refuse(
                'field-not-quoted',
                `${file}:${row.line}`,
                `${shape.name} declares ${cells[0]} without backticks. A field name is somebody else's word and is quoted, which is also what keeps it out of the terminology rule.`,
            )
            continue
        }

        if (cells[2] !== 'yes' && cells[2] !== 'no') {
            refuse(
                'required-not-answered',
                `${file}:${row.line}`,
                `${shape.name}'s ${name} says "${cells[2]}" where the answer is yes or no. A field whose requiredness is a sentence is one no client and no check can act on.`,
            )
            continue
        }

        declared.push({ name, required: cells[2] === 'yes', line: row.line })
    }

    return declared
}

let examples = 0
let fields = 0

for (const shape of shapes) {
    const declared = fieldsOf(shape)
    fields += declared.length

    if (declared.length === 0) {
        refuse(
            'shape-without-a-table',
            `${file}:${shape.line}`,
            `${shape.name} declares no field. A shape with no table is one an example cannot be judged against, and it is how a shape gets described in prose and then drifts.`,
        )
    }

    const declaresType = declared.some((field) => field.name === 'type')

    if (shape.kind === 'message' && !declaresType && declared.length > 0) {
        refuse(
            'message-without-a-type',
            `${file}:${shape.line}`,
            `${shape.name} sits under the message heading and declares no type. A client dispatches on that field, so a message without one is a message nothing can route.`,
        )
    }

    if (shape.kind === 'inner' && declaresType) {
        refuse(
            'inner-shape-with-a-type',
            `${file}:${shape.line}`,
            `${shape.name} sits under the inner heading and declares a type. The two sets are handled differently by every client, and a shape that drifted between them would be handled by neither.`,
        )
    }

    if (shape.examples.length === 0) {
        refuse(
            'shape-without-an-example',
            `${file}:${shape.line}`,
            `${shape.name} carries no example. The example is the part a reader copies, and a shape with none is a shape nobody has written down whole.`,
        )
        continue
    }

    if (shape.examples.length > 1) {
        refuse(
            'shape-with-two-examples',
            `${file}:${shape.line}`,
            `${shape.name} carries ${shape.examples.length} examples. Two examples of one shape are two things to keep in step, and the second is the one that stops being updated.`,
        )
    }

    for (const example of shape.examples) {
        examples += 1

        let carried

        try {
            carried = JSON.parse(example.body)
        } catch (whyNot) {
            refuse('example-not-json', `${file}:${example.line}`, `${shape.name}'s example is not JSON: ${whyNot.message}.`)
            continue
        }

        if (carried === null || typeof carried !== 'object' || Array.isArray(carried)) {
            refuse('example-not-an-object', `${file}:${example.line}`, `${shape.name}'s example is not an object, and every shape here is one.`)
            continue
        }

        const keys = Object.keys(carried)

        if (shape.kind === 'message' && carried.type !== shape.name) {
            refuse(
                'type-does-not-name-the-shape',
                `${file}:${example.line}`,
                `${shape.name}'s example carries a type of ${JSON.stringify(carried.type)}. A client routes on that value, so an example naming a different shape sends the reader who copied it somewhere else.`,
            )
        }

        if (shape.kind === 'inner' && keys.includes('type')) {
            refuse(
                'inner-example-with-a-type',
                `${file}:${example.line}`,
                `${shape.name}'s example carries a type and this shape never travels alone, so there is nothing for that field to route.`,
            )
        }

        for (const key of keys) {
            if (!declared.some((field) => field.name === key)) {
                refuse(
                    'field-not-declared',
                    `${file}:${example.line}`,
                    `${shape.name}'s example carries ${key}, which its table does not declare. A field a reader can see and a table cannot is a field nothing is promised about.`,
                )
            }
        }

        for (const field of declared) {
            if (field.required && !keys.includes(field.name)) {
                refuse(
                    'required-field-absent',
                    `${file}:${example.line}`,
                    `${shape.name}'s table requires ${field.name} and its example omits it, so the example is a message this document says would be refused.`,
                )
            }
        }
    }
}

if (!lines.includes(REQUIREMENTS)) {
    process.stderr.write(
        `sync-example-scan: ${file} holds no "${REQUIREMENTS}". The numbered requirements are what a conformance suite maps a case to, and a document with none leaves that suite nothing to be complete against.\n`,
    )
    process.exit(2)
}

if (requirements.length === 0) {
    process.stderr.write(
        `sync-example-scan: ${file} holds a requirement heading and no requirement under it. Refusing rather than passing an empty list as one nothing is missing from.\n`,
    )
    process.exit(2)
}

// Dense and in order. A hole is a requirement that was removed, and a suite
// reading the list cannot tell that from one nobody wrote a case for.
requirements.forEach((requirement, index) => {
    if (requirement.number !== index + 1) {
        refuse(
            'requirement-out-of-order',
            `${file}:${requirement.line}`,
            `R${requirement.number} is the requirement in position ${index + 1}. The list is dense and in order so that a suite can say which requirement has no case, and a hole reads as a case nobody wrote.`,
        )
    }
})

if (refused > 0) {
    process.stderr.write(
        `sync-example-scan: ${refused} refusal(s) in ${file}. An example that disagrees with its table is a document saying two things, and the reader who copies the example is the one who finds out.\n`,
    )
    process.exit(1)
}

process.stdout.write(
    `sync-example-scan: ${shapes.length} shape(s) in ${file}, ${fields} declared field(s), ${examples} example(s), every one carrying what its table declares and nothing else, and ${requirements.length} requirement(s) numbered densely. Whether a table describes what an instance would send is not judged here.\n`,
)
