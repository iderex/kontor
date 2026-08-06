// The one-change neighbour of unchecked-index.ts, and the leg that stops the
// evidence from being satisfied by a gate that refuses everything. The same
// function, written the way the setting asks for, has to pass under exactly the
// same configuration.

export function first(values: readonly string[]): string {
  const head = values[0];
  if (head === undefined) {
    throw new Error("first: the list is empty");
  }
  return head;
}
