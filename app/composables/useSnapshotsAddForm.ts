import type { AccountBalanceSnapshotDto, AccountSnapshotsCreateInput } from "~/generated/bindings";

type NumericField = "amount" | "change";

interface SnapshotDraftRow {
  key: number
  date: string
  amount: number | undefined
  liveAmount: number | undefined
  editingField: NumericField | null
  amountTouched: boolean
}

interface SnapshotContext {
  date: string
  balance_minor: number
}

interface SnapshotRowState {
  amountMinor: number | null
  dateError: string | null
  futureDateWarning: string | null
  previous: SnapshotContext | null
  conflictExisting: AccountBalanceSnapshotDto | null
  changeMinor: number | null
  changeValue: number | undefined
  sameBalanceWarning: boolean
  showAmountErrorState: boolean
  showChangeErrorState: boolean
}

interface UseSnapshotsAddFormParams {
  open: Ref<boolean>
  accountId: Ref<number | null>
  snapshots: Ref<AccountBalanceSnapshotDto[]>
}

export function useSnapshotsAddForm(params: UseSnapshotsAddFormParams) {
  const { createSnapshots } = useAccountSnapshotMutations();

  const submitError = ref<string | null>(null);
  const overwriteExistingConfirmed = ref(false);
  const overwriteConfirmationTouched = ref(false);
  const baselineSnapshots = ref<AccountBalanceSnapshotDto[]>([]);
  const rows = ref<SnapshotDraftRow[]>([]);
  const amountFieldRefs = shallowReactive<Record<number, HTMLElement | null>>({});
  const changeFieldRefs = shallowReactive<Record<number, HTMLElement | null>>({});

  let nextRowKey = 0;

  const today = ref(getTodayCalendarDateIsoString());

  const snapshotsAsc = computed(() => {
    return [...baselineSnapshots.value].sort((left, right) => left.date.localeCompare(right.date));
  });

  const existingByDate = computed(() => {
    return new Map(baselineSnapshots.value.map((snapshot) => [snapshot.date, snapshot]));
  });

  const activeRowCount = computed(() => {
    let count = rows.value.length;
    while (count > 0) {
      const row = rows.value[count - 1]!;
      if (row.date === "" && effectiveAmount(row) == null && !row.amountTouched) {
        count -= 1;
        continue;
      }
      break;
    }
    return count;
  });

  const rowStates = computed<SnapshotRowState[]>(() => {
    const duplicateCounts = new Map<string, number>();
    const activeRows = rows.value.slice(0, activeRowCount.value);
    const stagedDates = new Set(
      activeRows
        .filter((row) => row.date !== "" && convertCurrencyMajorAmountToMinorUnits(effectiveAmount(row)) != null)
        .map((row) => row.date)
    );
    const existingSnapshots = snapshotsAsc.value.filter((snapshot) => !stagedDates.has(snapshot.date));
    let existingIndex = 0;
    let previousSnapshot: SnapshotContext | null = null;

    for (const row of activeRows) {
      if (row.date === "") continue;
      duplicateCounts.set(row.date, (duplicateCounts.get(row.date) ?? 0) + 1);
    }

    return rows.value.map((row, index) => {
      const isActive = index < activeRowCount.value;
      let dateError: string | null = null;

      if (isActive) {
        if (row.date === "") {
          dateError = "Enter a date";
        } else if ((duplicateCounts.get(row.date) ?? 0) > 1) {
          dateError = "Duplicate date";
        } else if (index > 0) {
          const previousRowDate = rows.value[index - 1]!.date;
          if (previousRowDate !== "" && row.date <= previousRowDate) {
            dateError = "Snapshots must stay in ascending date order";
          }
        }
      }

      const amountMinor = convertCurrencyMajorAmountToMinorUnits(effectiveAmount(row));
      const conflictExisting = row.date === "" ? null : (existingByDate.value.get(row.date) || null);

      while (
        row.date !== ""
        && existingIndex < existingSnapshots.length
        && existingSnapshots[existingIndex]!.date < row.date
      ) {
        const existingSnapshot = existingSnapshots[existingIndex]!;
        previousSnapshot = {
          date: existingSnapshot.date,
          balance_minor: existingSnapshot.balance_minor
        };
        existingIndex += 1;
      }

      const previous = row.date === "" ? null : previousSnapshot;
      const changeMinor = previous == null || amountMinor == null
        ? null
        : amountMinor - previous.balance_minor;
      const showAmountErrorState = row.amountTouched && row.amount == null;

      if (row.date !== "" && amountMinor != null) {
        previousSnapshot = {
          date: row.date,
          balance_minor: amountMinor
        };
      }

      return {
        amountMinor,
        dateError,
        futureDateWarning: futureDateWarning(row, index),
        previous,
        conflictExisting,
        changeMinor,
        changeValue: changeMinor == null ? undefined : convertCurrencyMinorUnitsToMajorAmount(changeMinor),
        sameBalanceWarning: previous != null && amountMinor != null && amountMinor === previous.balance_minor,
        showAmountErrorState,
        showChangeErrorState: previous != null && showAmountErrorState
      };
    });
  });

  const hasOverwriteConflicts = computed(() => {
    return rowStates.value.slice(0, activeRowCount.value).some((row) => row.conflictExisting != null);
  });

  const overwriteConfirmationError = computed(() => {
    if (!hasOverwriteConflicts.value || overwriteExistingConfirmed.value || !overwriteConfirmationTouched.value) return null;
    return "Confirm overwrite to continue";
  });

  watch(params.open, (isOpen) => {
    if (!isOpen) return;

    today.value = getTodayCalendarDateIsoString();
    baselineSnapshots.value = params.snapshots.value.map((snapshot) => ({ ...snapshot }));
    submitError.value = null;
    overwriteExistingConfirmed.value = false;
    overwriteConfirmationTouched.value = false;
    rows.value = [createRow(nextStartDate())];
  });

  async function onSubmit() {
    if (params.accountId.value == null) return;

    today.value = getTodayCalendarDateIsoString();
    for (const row of rows.value.slice(0, activeRowCount.value)) {
      row.amountTouched = true;
    }
    overwriteConfirmationTouched.value = true;

    if (
      activeRowCount.value === 0
      || rowStates.value.slice(0, activeRowCount.value).some((row) => row.dateError != null || row.amountMinor == null)
      || overwriteConfirmationError.value != null
    ) {
      return;
    }

    submitError.value = null;

    const input: AccountSnapshotsCreateInput = {
      snapshots: rows.value.slice(0, activeRowCount.value).map((row, index) => ({
        date: row.date,
        balance_minor: rowStates.value[index]!.amountMinor!,
        overwrite_existing: rowStates.value[index]!.conflictExisting != null && overwriteExistingConfirmed.value
      }))
    };

    try {
      await createSnapshots.mutateAsync({ accountId: params.accountId.value, input });
      params.open.value = false;
    } catch (error) {
      submitError.value = error instanceof Error ? error.message : "Failed to create snapshots";
    }
  }

  function onRowDateUpdate(index: number, value: unknown) {
    const row = rows.value[index];
    if (row == null) return;
    row.date = getCalendarDateIsoStringFromInputValue(value);
  }

  function removeRow(index: number) {
    if (rows.value.length === 1) {
      rows.value = [createRow(nextStartDate())];
      return;
    }

    rows.value.splice(index, 1);
  }

  function onValueKeydown(event: KeyboardEvent, index: number, field: NumericField) {
    if (event.key !== "Enter") return;

    const row = rows.value[index];
    if (row == null) return;

    event.preventDefault();

    if (event.shiftKey) {
      focusNumericField(index - 1, field);
      return;
    }

    if (index === rows.value.length - 1 && (row.date !== "" || effectiveAmount(row) != null || row.amountTouched)) {
      if (effectiveAmount(row) == null) {
        row.date = nextDateFrom(index);
        return;
      }

      addRow(index, field);
      return;
    }

    focusNumericField(index + 1, field);
  }

  function addRow(index: number, field: NumericField = "amount") {
    rows.value.splice(index + 1, 0, createRow(nextDateFrom(index)));
    void nextTick(() => focusNumericField(index + 1, field));
  }

  function setNumericFieldRef(rowKey: number, field: NumericField) {
    return (element: unknown) => {
      const resolved = resolveHtmlElement(element);

      if (field === "amount") {
        amountFieldRefs[rowKey] = resolved;
        return;
      }

      changeFieldRefs[rowKey] = resolved;
    };
  }

  function onRowAmountInput(index: number, event: Event) {
    const row = rows.value[index];
    if (row == null) return;

    row.editingField = "amount";
    row.liveAmount = normalizeOptionalCurrencyMajorAmount(parseCurrencyInputNumberEventValue(event));
  }

  function onRowChangeInput(index: number, event: Event) {
    const row = rows.value[index];
    const previous = rowStates.value[index]!.previous;
    if (row == null) return;

    row.editingField = "change";
    const changeValue = normalizeOptionalCurrencyMajorAmount(parseCurrencyInputNumberEventValue(event));

    if (changeValue == null) {
      row.liveAmount = undefined;
      return;
    }

    if (previous != null) {
      row.liveAmount = normalizeCurrencyMajorAmount(convertCurrencyMinorUnitsToMajorAmount(previous.balance_minor) + changeValue);
    }
  }

  function onRowAmountUpdate(index: number, value: number | null | undefined) {
    const row = rows.value[index];
    if (row == null) return;

    const normalized = normalizeOptionalCurrencyMajorAmount(value);
    row.amount = normalized;
    row.liveAmount = normalized;
  }

  function onRowChangeUpdate(index: number, value: number | null | undefined) {
    const row = rows.value[index];
    const previous = rowStates.value[index]!.previous;
    if (row == null) return;

    if (value == null) {
      row.amount = undefined;
      row.liveAmount = undefined;
      return;
    }

    if (previous != null) {
      const normalized = normalizeCurrencyMajorAmount(convertCurrencyMinorUnitsToMajorAmount(previous.balance_minor) + value);
      row.amount = normalized;
      row.liveAmount = normalized;
    }
  }

  function onNumericBlur(index: number) {
    const row = rows.value[index];
    if (row == null) return;

    row.amountTouched = true;
    row.amount = row.liveAmount;
    row.editingField = null;
    row.liveAmount = row.amount;
  }

  function amountModelValue(row: SnapshotDraftRow) {
    return row.editingField === "amount" ? row.amount : effectiveAmount(row);
  }

  function changeModelValue(row: SnapshotDraftRow, index: number) {
    if (row.editingField === "change") {
      const previous = rowStates.value[index]!.previous;
      const amountMinor = convertCurrencyMajorAmountToMinorUnits(row.amount);
      if (previous == null || amountMinor == null) return undefined;
      return convertCurrencyMinorUnitsToMajorAmount(amountMinor - previous.balance_minor);
    }

    return rowStates.value[index]!.changeValue;
  }

  function futureDateWarning(row: SnapshotDraftRow, index: number) {
    if (row.date === "" || row.date <= today.value) return null;

    const laterStagedSnapshot = rows.value.some((otherRow, otherIndex) => (
      otherIndex !== index
      && otherIndex < activeRowCount.value
      && otherRow.date > row.date
    ));
    const laterExistingSnapshot = baselineSnapshots.value.some((snapshot) => snapshot.date > row.date);
    if (laterStagedSnapshot || laterExistingSnapshot) {
      return "Future-dated snapshot. Balance-over-time charts only show data through today.";
    }

    return "Future-dated snapshot. This snapshot will count as the latest balance, but balance-over-time charts only show data through today.";
  }

  function effectiveAmount(row: SnapshotDraftRow) {
    return row.editingField == null ? row.amount : row.liveAmount;
  }

  function nextStartDate() {
    const lastDate = snapshotsAsc.value.at(-1)?.date;
    return lastDate != null && lastDate !== "" ? addDaysToCalendarDateIsoString(lastDate, 1) : today.value;
  }

  function nextDateFrom(index: number) {
    const currentDate = rows.value[index]!.date;
    if (currentDate !== "") return addDaysToCalendarDateIsoString(currentDate, 1);

    const previousDate = rows.value[index - 1]?.date;
    return previousDate != null && previousDate !== "" ? addDaysToCalendarDateIsoString(previousDate, 1) : nextStartDate();
  }

  function createRow(date: string): SnapshotDraftRow {
    const row = {
      key: nextRowKey,
      date,
      amount: undefined,
      liveAmount: undefined,
      editingField: null,
      amountTouched: false
    };
    nextRowKey += 1;
    return row;
  }

  function focusNumericField(index: number, field: NumericField) {
    const row = rows.value[index];
    if (row == null) return;

    const input = (field === "amount"
      ? amountFieldRefs[row.key]
      : changeFieldRefs[row.key])?.querySelector("input");
    if (!(input instanceof HTMLInputElement)) return;

    input.focus();
    requestAnimationFrame(() => {
      input.select();
    });
  }

  function resolveHtmlElement(element: unknown) {
    if (element instanceof HTMLElement) return element;
    if (element == null || typeof element !== "object" || !("$el" in element)) return null;
    return element.$el instanceof HTMLElement ? element.$el : null;
  }

  return {
    createSnapshots,
    submitError,
    overwriteExistingConfirmed,
    rows,
    getCalendarDateModelValueFromIsoString,
    rowStates,
    hasOverwriteConflicts,
    overwriteConfirmationError,
    onSubmit,
    onRowDateUpdate,
    removeRow,
    onValueKeydown,
    addRow,
    setNumericFieldRef,
    onRowAmountInput,
    onRowAmountUpdate,
    onRowChangeInput,
    onRowChangeUpdate,
    onNumericBlur,
    amountModelValue,
    changeModelValue
  };
}
