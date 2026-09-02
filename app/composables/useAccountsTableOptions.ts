import type { SelectItem } from "@nuxt/ui";
import type { ExpandedState, SortingState } from "@tanstack/vue-table";
import type { MaybeRefOrGetter } from "vue";
import type { ActivityPeriod } from "~/generated/bindings";

export type AccountGroupBy = "none" | "institution" | "type";
export type AccountsHideColumn = "institution";

interface UseAccountsTableOptionsArgs {
  scope: MaybeRefOrGetter<string>
  hideColumns?: AccountsHideColumn[] | Ref<AccountsHideColumn[]>
}

interface AccountsTableOptions {
  groupBy: AccountGroupBy
  hideEmpty: boolean
  activityPeriod: ActivityPeriod
  sorting: SortingState
  expanded: ExpandedState
}

export function useAccountsTableOptions(args: UseAccountsTableOptionsArgs) {
  const views = useState<Record<string, AccountsTableOptions>>("accountsTableOptions", () => ({}));

  watch(() => toValue(args.scope), (scope) => {
    views.value[scope] ??= {
      groupBy: "none",
      hideEmpty: false,
      activityPeriod: "1M",
      sorting: [{ id: "name", desc: false }],
      expanded: {}
    };
  }, { immediate: true, flush: "sync" });

  const options = computed(() => views.value[toValue(args.scope)]!);

  watch([() => toValue(args.scope), () => options.value.groupBy], ([scope, groupBy], [previousScope, previousGroupBy]) => {
    if (scope === previousScope && groupBy !== previousGroupBy) options.value.expanded = {};
  }, { flush: "sync" });

  const hiddenColumns = computed<Set<AccountsHideColumn>>(() => (
    new Set(toValue(args.hideColumns) ?? [])
  ));

  const groupByItems = computed<SelectItem[]>(() => {
    const out: SelectItem[] = [
      { label: "None", value: "none" }
    ];

    if (!hiddenColumns.value.has("institution")) {
      out.push({ label: "Institution", value: "institution" });
    }

    out.push({ label: "Type", value: "type" });
    return out;
  });

  const activityPeriodItems = ref<SelectItem[]>([
    { label: "1W", value: "1W" },
    { label: "1M", value: "1M" },
    { label: "3M", value: "3M" },
    { label: "6M", value: "6M" }
  ]);

  watchEffect(() => {
    if (hiddenColumns.value.has("institution") && options.value.groupBy === "institution") {
      options.value.groupBy = "none";
    }
  });

  return {
    options,
    groupByItems,
    activityPeriodItems
  };
}
