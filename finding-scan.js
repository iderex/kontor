#!/usr/bin/env node
// SPDX-License-Identifier: AGPL-3.0-only
// One refusal over what a scanner reported: a result at or above the severity
// this file states fails the run rather than sitting in a tab somebody visits.
//
//     finding-scan.js sarif/rust.sarif
//     finding-scan.js sarif/rust.sarif sarif/client.sarif
//
// A code scanning analysis uploads what it found and exits successfully whether
// it found anything or not, so the analysis on its own is a report rather than
// a gate. This is the part that refuses, and it is separate from the scanner
// for a reason worth knowing: the scanner is a third party program whose
// verdicts move as its queries move, and what may not move quietly is the line
// this project draws through them.
//
// THE STATED SEVERITY IS 4.0 AND IT IS A LINE THROUGH THE SCANNER'S OWN SCALE.
// A rule that carries a `security-severity` property carries a number from 0 to
// 10, and the surface these results are uploaded to bands it: below 4.0 is low,
// 4.0 to 6.9 is medium, 7.0 to 8.9 is high, and 9.0 and above is critical. The
// line is drawn at medium because a low finding in a tree this size is a thing
// to read rather than a thing to stop for, and because a line drawn at high
// would let a whole band through in silence. Moving the line is a change to
// this file, argued in the issue that moves it, and lowering it is the same
// change and the one to be slowest to agree with.
//
// A RULE THAT CARRIES NO SECURITY SEVERITY IS NOT JUDGED HERE. The query set
// this project runs includes maintainability queries, and those carry no such
// number because they are not about an attack. They are uploaded, they are
// counted in the sentence this prints, and they do not fail a run. That is the
// difference between the security surface and the gate, and it is deliberate:
// a gate that stopped a merge for a style finding is a gate somebody switches
// off.
//
// IT FAILS CLOSED. A file that is not there, bytes that are not JSON, a report
// holding no run at all, a result naming a rule the report does not describe,
// and a severity that is not a number are each refused rather than passed over.
// The case worth naming is the third: an analysis that ran over nothing
// produces a report with no run in it, and a check that read that as a clean
// tree would turn a scanner that quietly stopped scanning into a green tick.
//
// THE COMPARISON IS ON THE NUMBER AND NOT ON THE TEXT. `"10.0"` sorts before
// `"4.0"` as text, so a check written the easy way would pass exactly the
// critical findings it exists for. prove-finding-scan has that leg.
//
// WHAT IT DOES NOT DECIDE, said plainly because a green run says nothing about
// any of it. Whether the scanner found everything is not decided here: a report
// with no results is a statement about the queries that ran, not about the
// tree. Whether a finding is real is not decided here either, and a false one
// is refused exactly as a true one is, because this reads a number a query
// wrote and nothing else. And it reads a report rather than a repository, so
// the analysis that produced the report is what says which files were read.
//
// Node rather than the POSIX sh every other scanner here is written in, and the
// reason is the input rather than a preference. A report is JSON, a result's
// severity lives on a rule declared somewhere else in the same document, and
// resolving one against the other is a traversal rather than a question about
// lines. Node parses JSON with nothing installed and this tree already requires
// it at the version .nvmrc pins, so this costs no dependency; awk over JSON
// would be the wrong means and jq is absent from a fresh clone on Windows.
//
// IT CARRIES AN EXTENSION WHERE EVERY OTHER SCANNER HERE CARRIES NONE, and the
// reason is a check rather than a preference. `licence-scan headers` derives the
// comment mark a header is owed from the extension, defaulting to the one a
// shell script uses, so an extensionless file holding a Node program is refused
// for a header it cannot legally write. The extension is the smaller change and
// it says what the file is at the place it is called.
//
// prove-finding-scan is the evidence that this refuses what it names, against
// reports it writes rather than against anything a scanner produced here.

'use strict'

const fs = require('node:fs')

// The line, and the bands it sits in. Both are printed rather than left here,
// so a reader of the output does not have to open this file to know what was
// applied.
const STATED_SEVERITY = 4.0

const advice =
    'Fix what the rule names, or argue in the issue that the query is wrong for this tree and take it out of the query set with the reason beside it. Raising the line to pass is the change to be slowest to agree with.'

const files = process.argv.slice(2)

if (files.length === 0) {
    process.stderr.write(
        'finding-scan: no report was named. It reads a file a code scanning analysis wrote, so a run with no argument would judge nothing and must not read as one that judged a tree and found nothing.\n',
    )
    process.exit(2)
}

let refused = 0
let results = 0
let carrying = 0

// The rules a report describes, keyed by the id a result names. A tool declares
// them in two places: the driver, and an extension the driver loaded. Both are
// read, because a code scanning report puts the queries in the extension and a
// check reading only the driver would resolve nothing and refuse everything.
function rulesOf(run) {
    const tool = run.tool || {}
    const components = [tool.driver, ...(tool.extensions || [])]
    const byId = new Map()

    for (const component of components) {
        for (const rule of (component && component.rules) || []) {
            if (rule && typeof rule.id === 'string') {
                byId.set(rule.id, rule)
            }
        }
    }

    return byId
}

// Where a result was found, for the reader who has to go and look. A report
// carries no location for a finding about the run itself, and the sentence says
// so rather than printing an empty pair of brackets.
function whereIs(result) {
    const location = (result.locations || [])[0]
    const physical = location && location.physicalLocation
    const uri = physical && physical.artifactLocation && physical.artifactLocation.uri
    const line = physical && physical.region && physical.region.startLine

    if (!uri) {
        return 'no file named in the report'
    }

    return line ? `${uri}:${line}` : uri
}

function refuse(sentence) {
    process.stderr.write(`finding-scan: ${sentence}\n`)
    refused += 1
}

for (const file of files) {
    let text

    try {
        text = fs.readFileSync(file, 'utf8')
    } catch {
        refuse(
            `${file} is not a file this can read. The report is written by the analysis step, so an absent one means the analysis did not get as far as writing it.`,
        )
        continue
    }

    let report

    try {
        report = JSON.parse(text)
    } catch (whyNot) {
        refuse(`${file} is not JSON: ${whyNot.message}. A report that cannot be read is not a report that found nothing.`)
        continue
    }

    const runs = (report && report.runs) || []

    if (runs.length === 0) {
        refuse(
            `${file} holds no run. An analysis that read nothing writes exactly this, so passing it would turn a scanner that stopped scanning into a green tick.`,
        )
        continue
    }

    for (const run of runs) {
        if (!Array.isArray(run.results)) {
            refuse(`${file} holds a run with no result list. A run that found nothing carries an empty list rather than no list.`)
            continue
        }

        const rules = rulesOf(run)

        for (const result of run.results) {
            results += 1

            const id = result.ruleId || (result.rule && result.rule.id)
            const rule = id === undefined ? undefined : rules.get(id)

            if (rule === undefined) {
                refuse(
                    `${file} holds a result naming ${id === undefined ? 'no rule' : `the rule ${id}`}, which the report does not describe, so its severity cannot be read. Refusing rather than passing a result nothing here can judge.`,
                )
                continue
            }

            const stated = (rule.properties || {})['security-severity']

            if (stated === undefined) {
                continue
            }

            carrying += 1

            const severity = Number(stated)

            if (!Number.isFinite(severity)) {
                refuse(
                    `${file}: ${id} states a security severity of "${stated}", which is not a number. Refusing rather than reading it as zero.`,
                )
                continue
            }

            if (severity >= STATED_SEVERITY) {
                refuse(`${file}: ${id} at severity ${severity} in ${whereIs(result)}. ${advice}`)
            }
        }
    }
}

if (refused > 0) {
    process.stderr.write(
        `finding-scan: ${refused} finding(s) at or above the stated severity of ${STATED_SEVERITY.toFixed(1)}, or reported in a shape this could not judge.\n`,
    )
    process.exit(1)
}

process.stdout.write(
    `finding-scan: ${results} result(s) across ${files.length} report(s), ${carrying} of them carrying a security severity and none of those at or above the stated ${STATED_SEVERITY.toFixed(1)}. A result below the line is in the security surface rather than absent, and whether the scanner found everything is not decided here.\n`,
)
