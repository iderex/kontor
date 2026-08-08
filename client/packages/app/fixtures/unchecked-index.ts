// SPDX-License-Identifier: AGPL-3.0-only
// A fixture for client/prove-gates, and the only file in the client workspace
// that is meant to be red. tsconfig.json includes `src` alone, so neither the
// build nor the workspace type check reads this directory; the two fixture
// projects beside it are the only things that do.
//
// The mistake is one character wide and everybody makes it. An element read out
// of an array by index is typed as present, and at run time the array may be
// empty. Plain `strict` accepts this file. It is red only because
// noUncheckedIndexedAccess is on, which is what makes it evidence about that
// setting rather than about type checking in general.

export function first(values: readonly string[]): string {
  return values[0];
}
