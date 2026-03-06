export const useRouteParamInt = <
  TParams extends Record<string, unknown>,
  K extends keyof TParams & string
>(
  route: { params: TParams },
  key: K
) => {
  return computed<number | null>(() => {
    const raw = route.params[key];
    if (typeof raw !== "string") return null;

    const n = Number(raw);
    if (!Number.isSafeInteger(n)) return null;
    return n;
  });
};
