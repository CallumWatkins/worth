import type { Component } from "vue";
import type {
  CsvSnapshotImportInspectionDto,
  CurrencyCode,
  SnapshotImportOptionsInput,
  SnapshotImportPreviewDto,
  SnapshotImportSourceInput
} from "~/generated/bindings";
import type { SnapshotImportFlowDefinition } from "~/utils/snapshot-import-flows";
import SnapshotImportCsvFileStep from "~/components/Accounts/Snapshots/Import/Csv/FileStep.vue";
import SnapshotImportCsvOptionsStep from "~/components/Accounts/Snapshots/Import/Csv/OptionsStep.vue";
import SnapshotImportReviewStep from "~/components/Accounts/Snapshots/Import/ReviewStep.vue";
import { ApiCommandError } from "~/composables/useApi";
import { reportHandledError } from "~/utils/error-reporting";

const csvFileStepComponent = markRaw(SnapshotImportCsvFileStep) as Component;
const csvOptionsStepComponent = markRaw(SnapshotImportCsvOptionsStep) as Component;
const reviewStepComponent = markRaw(SnapshotImportReviewStep) as Component;

interface UseCsvSnapshotImportFlowParams {
  accountId: Ref<number | null>
  currencyCode: Ref<CurrencyCode>
  setErrorMessage: (message: string | null) => void
  onComplete: () => void
}

export function useCsvSnapshotImportFlow(params: UseCsvSnapshotImportFlowParams): SnapshotImportFlowDefinition {
  const api = useApi();
  const { importSnapshots } = useAccountSnapshotMutations();

  const selectedFile = ref<File | null>(null);
  const sourceFileName = ref("");
  const sourceFileContent = ref("");
  const sourceHasHeaderRow = ref(true);
  const hasHeaderRow = ref(true);
  const inspection = ref<CsvSnapshotImportInspectionDto | null>(null);
  const preview = ref<SnapshotImportPreviewDto | null>(null);
  const inspectPending = ref(false);
  const previewPending = ref(false);
  let inspectRequestId = 0;

  const options = ref<SnapshotImportOptionsInput>({
    source: {
      kind: "csv",
      date_column: "",
      amount_column: "",
      date_format: "yyyy_mm_dd",
      timestamp_date_policy: "date_as_written",
      timestamp_missing_timezone_policy: "local",
      timestamp_missing_timezone: getLocalTimezone(),
      balance_format: "thousands_comma_decimal_dot",
      blank_amount_policy: "error"
    },
    existing_date_policy: "overwrite",
    unchanged_value_policy: "exclude",
    duplicate_date_policy: "keep_last",
    overwrite_existing_confirmed: false
  });

  const sourceInput = computed<SnapshotImportSourceInput | null>(() => {
    if (!sourceFileName.value || !sourceFileContent.value) return null;

    return {
      kind: "csv",
      file_name: sourceFileName.value,
      content: sourceFileContent.value,
      has_header_row: sourceHasHeaderRow.value
    };
  });

  const busy = computed(() => inspectPending.value || previewPending.value || importSnapshots.isPending);

  const hasPendingOverwrites = computed(() => (preview.value?.summary.overwrite_count ?? 0) > 0);

  const columnItems = computed(() => {
    return (inspection.value?.columns ?? []).map((column) => ({
      label: column.name,
      value: column.name
    }));
  });

  watch(selectedFile, async (file) => {
    if (!file) {
      clearFile();
      return;
    }

    await inspectFile(file);
  });

  watch(hasHeaderRow, async () => {
    if (selectedFile.value == null) return;
    await inspectFile(selectedFile.value);
  });

  async function inspectFile(file: File) {
    const requestId = ++inspectRequestId;
    const fileName = file.name;
    const fileHasHeaderRow = hasHeaderRow.value;

    inspectPending.value = true;
    params.setErrorMessage(null);
    sourceFileName.value = "";
    sourceFileContent.value = "";
    inspection.value = null;
    preview.value = null;

    try {
      const fileContent = await file.text();
      if (!isCurrentInspectRequest(requestId, file, fileHasHeaderRow)) return;

      const input: SnapshotImportSourceInput = {
        kind: "csv",
        file_name: fileName,
        content: fileContent,
        has_header_row: fileHasHeaderRow
      };

      const inspected = await api.accountSnapshotImportInspect(input);
      if (!isCurrentInspectRequest(requestId, file, fileHasHeaderRow)) return;

      if (inspected.kind !== "csv") {
        throw new Error("Import source returned an unexpected inspection result");
      }

      sourceFileName.value = fileName;
      sourceFileContent.value = fileContent;
      sourceHasHeaderRow.value = fileHasHeaderRow;
      inspection.value = inspected;
      options.value = {
        source: {
          kind: "csv",
          date_column: inspection.value.guesses.date_column ?? inspection.value.columns[0]?.name ?? "",
          amount_column: inspection.value.guesses.amount_column ?? inspection.value.columns[1]?.name ?? "",
          date_format: inspection.value.guesses.date_format ?? "yyyy_mm_dd",
          timestamp_date_policy: "date_as_written",
          timestamp_missing_timezone_policy: inspection.value.guesses.timestamp_missing_timezone_policy ?? "local",
          timestamp_missing_timezone: getLocalTimezone(),
          balance_format: inspection.value.guesses.balance_format ?? "thousands_comma_decimal_dot",
          blank_amount_policy: "error"
        },
        existing_date_policy: "overwrite",
        unchanged_value_policy: "exclude",
        duplicate_date_policy: inspection.value.guesses.duplicate_date_policy ?? "keep_last",
        overwrite_existing_confirmed: false
      };
    } catch (error) {
      if (!isCurrentInspectRequest(requestId, file, fileHasHeaderRow)) return;

      if (!(error instanceof ApiCommandError)) {
        reportHandledError(error, { source: "csv_import_inspect" });
      }

      inspection.value = null;
      params.setErrorMessage(error instanceof Error ? error.message : "Failed to inspect import source");
    } finally {
      if (isCurrentInspectRequest(requestId, file, fileHasHeaderRow)) {
        inspectPending.value = false;
      }
    }
  }

  function isCurrentInspectRequest(requestId: number, file: File, fileHasHeaderRow: boolean) {
    return requestId === inspectRequestId && selectedFile.value === file && hasHeaderRow.value === fileHasHeaderRow;
  }

  async function loadPreview() {
    if (params.accountId.value == null || sourceInput.value == null) return false;
    previewPending.value = true;
    params.setErrorMessage(null);
    preview.value = null;
    options.value = {
      ...options.value,
      overwrite_existing_confirmed: false
    };

    try {
      preview.value = await api.accountSnapshotImportPreview(params.accountId.value, sourceInput.value, options.value);
      return true;
    } catch (error) {
      if (!(error instanceof ApiCommandError)) {
        reportHandledError(error, { source: "csv_import_preview" });
      }

      params.setErrorMessage(error instanceof Error ? error.message : "Failed to preview import");
      return false;
    } finally {
      previewPending.value = false;
    }
  }

  async function complete() {
    if (params.accountId.value == null || sourceInput.value == null || preview.value == null) return false;
    if (hasPendingOverwrites.value && !options.value.overwrite_existing_confirmed) {
      params.setErrorMessage("Confirm overwrite to continue");
      return false;
    }

    params.setErrorMessage(null);

    try {
      await importSnapshots.mutateAsync({
        accountId: params.accountId.value,
        input: sourceInput.value,
        options: options.value
      });
      params.onComplete();
      return true;
    } catch (error) {
      if (!(error instanceof ApiCommandError)) {
        reportHandledError(error, { source: "csv_import_commit" });
      }

      params.setErrorMessage(error instanceof Error ? error.message : "Failed to import snapshots");
      return false;
    }
  }

  function clearFile() {
    inspectRequestId += 1;
    selectedFile.value = null;
    sourceFileName.value = "";
    sourceFileContent.value = "";
    sourceHasHeaderRow.value = hasHeaderRow.value;
    inspection.value = null;
    preview.value = null;
    inspectPending.value = false;
  }

  function reset() {
    inspectRequestId += 1;
    selectedFile.value = null;
    sourceFileName.value = "";
    sourceFileContent.value = "";
    sourceHasHeaderRow.value = true;
    hasHeaderRow.value = true;
    inspection.value = null;
    preview.value = null;
    inspectPending.value = false;
    previewPending.value = false;
    options.value = {
      source: {
        kind: "csv",
        date_column: "",
        amount_column: "",
        date_format: "yyyy_mm_dd",
        timestamp_date_policy: "date_as_written",
        timestamp_missing_timezone_policy: "local",
        timestamp_missing_timezone: getLocalTimezone(),
        balance_format: "thousands_comma_decimal_dot",
        blank_amount_policy: "error"
      },
      existing_date_policy: "overwrite",
      unchanged_value_policy: "exclude",
      duplicate_date_policy: "keep_last",
      overwrite_existing_confirmed: false
    };
  }

  return {
    id: "csv",
    label: "CSV file",
    description: "Import dates and balances from a local CSV export.",
    icon: "i-lucide-file-spreadsheet",
    completeLabel: "Import snapshots",
    isBusy: () => busy.value,
    reset,
    canComplete: () => {
      return params.accountId.value != null
        && sourceInput.value != null
        && preview.value != null
        && preview.value.summary.invalid_count === 0
        && (preview.value.summary.create_count + preview.value.summary.overwrite_count) > 0
        && (!hasPendingOverwrites.value || options.value.overwrite_existing_confirmed);
    },
    complete,
    steps: () => [
      {
        id: "csv-file",
        title: "File",
        icon: "i-lucide-file-up",
        component: csvFileStepComponent,
        canContinue: () => (inspection.value?.sample_rows.length ?? 0) > 0,
        onBack: clearFile,
        props: () => ({
          selectedFile: selectedFile.value,
          hasHeaderRow: hasHeaderRow.value,
          inspection: inspection.value,
          busy: busy.value,
          "onUpdate:selectedFile": (file: File | null) => {
            selectedFile.value = file;
          },
          "onUpdate:hasHeaderRow": (value: boolean) => {
            hasHeaderRow.value = value;
          }
        })
      },
      {
        id: "csv-options",
        title: "Options",
        icon: "i-lucide-sliders-horizontal",
        component: csvOptionsStepComponent,
        canContinue: () => {
          return params.accountId.value != null
            && sourceInput.value != null
            && options.value.source.date_column !== ""
            && options.value.source.amount_column !== "";
        },
        beforeNext: loadPreview,
        props: () => ({
          modelValue: options.value,
          columnItems: columnItems.value,
          busy: busy.value,
          "onUpdate:modelValue": (value: SnapshotImportOptionsInput) => {
            options.value = value;
          }
        })
      },
      {
        id: "csv-review",
        title: "Review",
        icon: "i-lucide-list-checks",
        component: reviewStepComponent,
        canContinue: () => false,
        props: () => ({
          preview: preview.value,
          currencyCode: params.currencyCode.value,
          overwriteExistingConfirmed: options.value.overwrite_existing_confirmed,
          invalidPreviewDescription: "Go back and adjust the options or fix the source file before importing.",
          emptyPreviewDescription: "Preview will appear here after the import options are checked.",
          "onUpdate:overwriteExistingConfirmed": (value: boolean) => {
            options.value = {
              ...options.value,
              overwrite_existing_confirmed: value
            };
          }
        })
      }
    ]
  };
}

function getLocalTimezone() {
  return Intl.DateTimeFormat().resolvedOptions().timeZone || "UTC";
}
