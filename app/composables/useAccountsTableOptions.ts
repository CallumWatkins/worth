import type { SelectItem } from "@nuxt/ui";
import type { Ref } from "vue";
import type { ActivityPeriod } from "~/bindings";

import { toValue } from "vue";

export type AccountGroupBy = "none" | "institution" | "type";
export type AccountsHideColumn = "institution";

interface UseAccountsTableOptionsArgs {
  hideColumns?: AccountsHideColumn[] | Ref<AccountsHideColumn[]>
}

export function useAccountsTableOptions(args: UseAccountsTableOptionsArgs = {}) {
  const groupBy = ref<AccountGroupBy>("none");
  const showEmpty = ref(false);
  const activityPeriod = ref<ActivityPeriod>("1M");

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
    if (hiddenColumns.value.has("institution") && groupBy.value === "institution") {
      groupBy.value = "none";
    }
  });

  return {
    groupBy,
    groupByItems,
    showEmpty,
    activityPeriod,
    activityPeriodItems
  };
}
