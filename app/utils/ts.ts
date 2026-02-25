/**
 * Assert that a value has type `never`.
 * @param value - The value that should have type `never`.
 */
export function assertNever(value: never) {
  throw new Error(`Unexpected value: ${value}`);
}

/**
 * Like `Required<T>`, but allows each value to be `undefined`.
 * Useful when every field must be explicitly set, but can still be `undefined`.
 */
export type RequiredOrUndefined<T> = { [K in keyof Required<T>]: T[K] | undefined };
