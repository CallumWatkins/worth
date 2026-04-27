<template>
  <UModal
    v-model:open="open"
    title="Edit snapshot"
    :dismissible="false"
    :close="false"
  >
    <template #body>
      <div class="space-y-4">
        <UAlert
          v-if="!currentSnapshot"
          color="error"
          variant="subtle"
          title="Snapshot not found"
        />

        <template v-else>
          <div class="text-sm text-muted">
            Editing {{ formatShortDate(currentSnapshot.date) }} at
            {{ formatCurrencyMinor(currentSnapshot.balance_minor, currencyCode) }}
          </div>

          <UAlert
            v-if="submitError"
            color="error"
            variant="subtle"
            :title="submitError"
          />

          <div class="grid grid-cols-2 gap-3">
            <div class="rounded-lg border border-default p-3">
              <div class="text-xs uppercase tracking-wide text-muted mb-1">
                Previous snapshot
              </div>
              <div v-if="previousSnapshot" class="space-y-0.5">
                <div class="font-medium text-highlighted">
                  {{ formatShortDate(previousSnapshot.date) }}
                </div>
                <div>{{ formatCurrencyMinor(previousSnapshot.balance_minor, currencyCode) }}</div>
              </div>
              <div v-else class="text-muted">
                No previous snapshot
              </div>
            </div>

            <div class="rounded-lg border border-default p-3">
              <div class="text-xs uppercase tracking-wide text-muted mb-1">
                Next snapshot
              </div>
              <div v-if="nextSnapshot" class="space-y-0.5">
                <div class="font-medium text-highlighted">
                  {{ formatShortDate(nextSnapshot.date) }}
                </div>
                <div>{{ formatCurrencyMinor(nextSnapshot.balance_minor, currencyCode) }}</div>
              </div>
              <div v-else class="text-muted">
                No next snapshot
              </div>
            </div>
          </div>

          <UFormField label="Date" :error="dateError || undefined">
            <UInputDate
              :model-value="getCalendarDateModelValueFromIsoString(state.date)"
              :disabled="updateSnapshot.isPending"
              :color="dateError != null ? 'error' : 'neutral'"
              :highlight="dateError != null"
              :trailing="false"
              class="w-full"
              @update:model-value="state.date = getCalendarDateIsoStringFromInputValue($event)"
            />
          </UFormField>

          <UAlert
            v-if="state.date !== '' && state.date > today"
            color="warning"
            variant="subtle"
            title="Future-dated snapshot"
            :description="nextSnapshot != null
              ? 'Balance-over-time charts only show data through today.'
              : 'This snapshot will count as the latest balance, but balance-over-time charts only show data through today.'"
          />

          <UAlert
            v-if="state.date !== '' && props.openedDate != null && state.date < props.openedDate"
            color="warning"
            variant="subtle"
            title="Before account opened date"
            :description="`This snapshot is before the account opened date of ${formatShortDate(props.openedDate)}.`"
          />

          <UAlert
            v-if="state.date !== '' && props.closedDate != null && state.date > props.closedDate"
            color="warning"
            variant="subtle"
            title="After account closed date"
            :description="`This snapshot is after the account closed date of ${formatShortDate(props.closedDate)}.`"
          />

          <UFormField label="Balance" :error="showAmountErrorState ? true : undefined">
            <UInputNumber
              v-model="state.amount"
              :step="0.01"
              :increment="false"
              :decrement="false"
              :disabled="updateSnapshot.isPending"
              :color="showAmountErrorState ? 'error' : 'neutral'"
              :highlight="showAmountErrorState"
              :format-options="{ style: 'currency', currency: props.currencyCode, currencySign: 'standard' }"
              class="w-full"
              @blur="amountTouched = true"
            />
          </UFormField>

          <div v-if="changeMinor != null" class="text-sm">
            Change from previous snapshot:
            <span :class="changeMinor >= 0 ? 'text-success' : 'text-error'">
              {{ formatCurrencyMinor(changeMinor, currencyCode, { signDisplay: 'always' }) }}
            </span>
          </div>

          <div v-if="conflictingSnapshot" class="space-y-2 rounded-lg border border-warning/40 bg-warning/10 p-3">
            <div class="text-warning">
              Saving this date will overwrite the existing snapshot from {{ formatShortDate(conflictingSnapshot.date) }}
              at {{ formatCurrencyMinor(conflictingSnapshot.balance_minor, currencyCode) }}.
            </div>
            <UCheckbox
              v-model="state.overwriteExisting"
              :disabled="updateSnapshot.isPending"
              label="I understand this will overwrite the existing snapshot"
            />
            <div v-if="overwriteError" class="text-xs text-error">
              {{ overwriteError }}
            </div>
          </div>

          <div class="flex justify-end gap-2">
            <UButton
              color="neutral"
              variant="ghost"
              :disabled="updateSnapshot.isPending"
              @click="open = false"
            >
              Cancel
            </UButton>
            <UButton
              :disabled="updateSnapshot.isPending || props.accountId == null || props.snapshotId == null || currentSnapshot == null"
              loading-auto
              @click="onSubmit"
            >
              Save changes
            </UButton>
          </div>
        </template>
      </div>
    </template>
  </UModal>
</template>

<script lang="ts" setup>
import type { AccountBalanceSnapshotDto, AccountSnapshotUpdateInput, CurrencyCode } from "~/generated/bindings";

const props = defineProps<{
  accountId: number | null
  snapshotId: number | null
  openedDate: string | null | undefined
  closedDate: string | null | undefined
  currencyCode: CurrencyCode
  snapshots: AccountBalanceSnapshotDto[]
}>();

const open = defineModel<boolean>("open", { required: true });

const { formatCurrencyMinor, formatShortDate } = useLocaleFormatters();
const { updateSnapshot } = useAccountSnapshotMutations();

const submitError = ref<string | null>(null);
const amountTouched = ref(false);
const baselineSnapshots = ref<AccountBalanceSnapshotDto[]>([]);
const state = reactive({
  date: "",
  amount: undefined as number | undefined,
  overwriteExisting: false
});

const today = ref(getTodayCalendarDateIsoString());

const currentSnapshot = computed(() => {
  if (props.snapshotId == null) return null;
  return baselineSnapshots.value.find((snapshot) => snapshot.id === props.snapshotId) ?? null;
});

const siblingSnapshotsAsc = computed(() => {
  return baselineSnapshots.value
    .filter((snapshot) => snapshot.id !== props.snapshotId)
    .sort((left, right) => left.date.localeCompare(right.date));
});

const previousSnapshot = computed(() => {
  if (state.date === "") return null;
  return siblingSnapshotsAsc.value.filter((snapshot) => snapshot.date < state.date).at(-1) ?? null;
});

const nextSnapshot = computed(() => {
  if (state.date === "") return null;
  return siblingSnapshotsAsc.value.find((snapshot) => snapshot.date > state.date) ?? null;
});

const conflictingSnapshot = computed(() => {
  if (state.date === "") return null;
  return baselineSnapshots.value.find((snapshot) => snapshot.id !== props.snapshotId && snapshot.date === state.date) ?? null;
});

const amountMinor = computed(() => convertCurrencyMajorAmountToMinorUnits(state.amount));

const dateError = computed(() => {
  if (state.date === "") return "Enter a date";
  return null;
});

const showAmountErrorState = computed(() => amountTouched.value && state.amount == null);

const overwriteError = computed(() => {
  if (conflictingSnapshot.value == null || state.overwriteExisting) return null;
  return "Confirm overwrite to continue";
});

const changeMinor = computed(() => {
  if (amountMinor.value == null || previousSnapshot.value == null) return null;
  return amountMinor.value - previousSnapshot.value.balance_minor;
});

watch(open, (isOpen) => {
  if (!isOpen) return;
  today.value = getTodayCalendarDateIsoString();
  baselineSnapshots.value = props.snapshots.map((snapshot) => ({ ...snapshot }));
  hydrateFromSnapshot();
});

watch(() => props.snapshotId, () => {
  if (!open.value) return;
  baselineSnapshots.value = props.snapshots.map((snapshot) => ({ ...snapshot }));
  hydrateFromSnapshot();
});

async function onSubmit() {
  today.value = getTodayCalendarDateIsoString();
  amountTouched.value = true;

  if (
    props.accountId == null
    || props.snapshotId == null
    || dateError.value != null
    || overwriteError.value != null
    || amountMinor.value == null
  ) {
    return;
  }

  submitError.value = null;

  const input: AccountSnapshotUpdateInput = {
    date: state.date,
    balance_minor: amountMinor.value,
    overwrite_existing: state.overwriteExisting
  };

  try {
    await updateSnapshot.mutateAsync({
      accountId: props.accountId,
      snapshotId: props.snapshotId,
      input
    });
    open.value = false;
  } catch (error) {
    submitError.value = error instanceof Error ? error.message : "Failed to update snapshot";
  }
}

function hydrateFromSnapshot() {
  submitError.value = null;
  amountTouched.value = false;
  state.overwriteExisting = false;
  state.date = currentSnapshot.value == null ? "" : currentSnapshot.value.date;
  state.amount = currentSnapshot.value == null ? undefined : convertCurrencyMinorUnitsToMajorAmount(currentSnapshot.value.balance_minor);
}
</script>
