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

    const n = Number.parseInt(raw);
    if (!Number.isFinite(n)) return null;
    return n;
  });
};
