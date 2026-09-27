export interface BalanceRange {
  minimum: number | null
  maximum: number | null
  minimumOperator: ">=" | ">"
  maximumOperator: "<=" | "<"
}

export function createBalanceRange(): BalanceRange {
  return { minimum: null, maximum: null, minimumOperator: ">=", maximumOperator: "<" };
}

export function matchesBalanceRange(balance: number, range: BalanceRange): boolean {
  const { minimum, maximum, minimumOperator, maximumOperator } = range;
  return (minimum === null || (minimumOperator === ">=" ? balance >= minimum : balance > minimum))
    && (maximum === null || (maximumOperator === "<=" ? balance <= maximum : balance < maximum));
}
