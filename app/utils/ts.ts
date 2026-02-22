/**
 * Assert that a value has type `never`.
 * @param value - The value that should have type `never`.
 */
export function assertNever(value: never) {
  throw new Error(`Unexpected value: ${value}`);
}
