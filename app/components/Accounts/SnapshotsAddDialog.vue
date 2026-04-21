<template>
  <UModal
    v-model:open="open"
    title="Add snapshots"
    :dismissible="false"
    :close="false"
    :ui="{ content: 'max-w-5xl' }"
  >
    <template #body>
      <div class="space-y-4">
        <UAlert
          v-if="submitError"
          color="error"
          variant="subtle"
          :title="submitError"
        />

        <div class="overflow-auto rounded-lg border border-default">
          <table class="min-w-full divide-y divide-default text-sm">
            <thead class="bg-elevated/50 text-left text-muted">
              <tr class="whitespace-nowrap font-medium">
                <th class="px-3 py-2">
                  Date
                </th>
                <th class="px-3 py-2">
                  Balance
                </th>
                <th class="px-3 py-2">
                  Previous balance
                </th>
                <th class="px-3 py-2">
                  Change
                </th>
                <th class="min-w-60" />
                <th />
              </tr>
            </thead>

            <tbody class="divide-y divide-default align-top">
              <tr v-for="(row, index) in rows" :key="row.key">
                <td class="p-3">
                  <UFormField :error="rowState(index).dateError || undefined">
                    <UInputDate
                      :model-value="getCalendarDateModelValueFromIsoString(row.date)"
                      :max-value="todayDate"
                      :disabled="createSnapshots.isPending"
                      :color="rowState(index).dateError != null ? 'error' : 'neutral'"
                      :highlight="rowState(index).dateError != null"
                      :trailing="false"
                      class="w-full"
                      @update:model-value="onRowDateUpdate(index, $event)"
                    />
                  </UFormField>
                </td>

                <td class="p-3">
                  <UFormField :error="rowState(index).showAmountErrorState ? true : undefined">
                    <div :ref="setNumericFieldRef(row.key, 'amount')">
                      <UInputNumber
                        :model-value="amountModelValue(row)"
                        :step="0.01"
                        :increment="false"
                        :decrement="false"
                        :disabled="createSnapshots.isPending"
                        :color="rowState(index).showAmountErrorState ? 'error' : 'neutral'"
                        :highlight="rowState(index).showAmountErrorState"
                        :format-options="{ style: 'currency', currency: props.currencyCode, currencySign: 'standard' }"
                        class="w-full"
                        @input="onRowAmountInput(index, $event)"
                        @update:model-value="onRowAmountUpdate(index, $event)"
                        @blur="onNumericBlur(index)"
                        @keydown="onValueKeydown($event, index, 'amount')"
                      />
                    </div>
                  </UFormField>
                </td>

                <td class="py-2.5 px-3">
                  <div v-if="rowState(index).previous">
                    <div class="font-medium text-highlighted">
                      {{ formatCurrencyMinor(rowState(index).previous!.balance_minor, props.currencyCode) }}
                    </div>
                    <div class="text-xs text-muted">
                      {{ formatShortDate(rowState(index).previous!.date) }}
                    </div>
                  </div>
                  <div v-else class="text-muted">
                    No previous snapshot
                  </div>
                </td>

                <td class="p-3">
                  <UFormField :error="rowState(index).showChangeErrorState ? true : undefined">
                    <div :ref="setNumericFieldRef(row.key, 'change')">
                      <UInputNumber
                        :model-value="changeModelValue(row, index)"
                        :step="0.01"
                        :increment="false"
                        :decrement="false"
                        :disabled="createSnapshots.isPending || rowState(index).previous == null"
                        :color="rowState(index).showChangeErrorState ? 'error' : 'neutral'"
                        :highlight="rowState(index).showChangeErrorState"
                        :format-options="{ style: 'currency', currency: props.currencyCode, currencySign: 'standard', signDisplay: 'exceptZero' }"
                        class="w-full"
                        @input="onRowChangeInput(index, $event)"
                        @update:model-value="onRowChangeUpdate(index, $event)"
                        @blur="onNumericBlur(index)"
                        @keydown="onValueKeydown($event, index, 'change')"
                      />
                    </div>
                  </UFormField>
                </td>

                <td class="p-3">
                  <div class="space-y-2">
                    <div v-if="rowState(index).conflictExisting" class="text-warning">
                      Overwrites the existing snapshot from {{ formatShortDate(row.date) }}
                      at {{ formatCurrencyMinor(rowState(index).conflictExisting!.balance_minor, props.currencyCode) }}.
                    </div>
                    <div v-if="rowState(index).sameBalanceWarning" class="text-warning">
                      Balance is the same as the previous snapshot.
                    </div>
                  </div>
                </td>

                <td class="p-3">
                  <UButton
                    color="neutral"
                    variant="ghost"
                    icon="i-lucide-trash-2"
                    :disabled="createSnapshots.isPending"
                    :aria-label="`Remove row ${index + 1}`"
                    @click="removeRow(index)"
                  />
                </td>
              </tr>
            </tbody>
          </table>
        </div>

        <UButton
          color="neutral"
          variant="subtle"
          icon="i-lucide-plus"
          :disabled="createSnapshots.isPending"
          @click="addRow(rows.length - 1)"
        >
          Add snapshot
        </UButton>

        <div v-if="hasOverwriteConflicts" class="space-y-2 rounded-lg border border-warning/40 bg-warning/10 p-3">
          <UCheckbox
            v-model="overwriteExistingConfirmed"
            :disabled="createSnapshots.isPending"
            label="I understand conflicting dates will overwrite existing snapshots"
          />
          <div v-if="overwriteConfirmationError" class="text-xs text-error">
            {{ overwriteConfirmationError }}
          </div>
        </div>

        <div class="flex justify-end gap-2">
          <UButton
            color="neutral"
            variant="ghost"
            :disabled="createSnapshots.isPending"
            @click="open = false"
          >
            Cancel
          </UButton>
          <UButton
            :disabled="createSnapshots.isPending || props.accountId == null"
            loading-auto
            @click="onSubmit"
          >
            Create snapshots
          </UButton>
        </div>
      </div>
    </template>
  </UModal>
</template>

<script lang="ts" setup>
import type { AccountBalanceSnapshotDto, CurrencyCode } from "~/generated/bindings";
import { toRef } from "vue";

const props = defineProps<{
  accountId: number | null
  currencyCode: CurrencyCode
  snapshots: AccountBalanceSnapshotDto[]
}>();

const open = defineModel<boolean>("open", { required: true });

const { formatCurrencyMinor, formatShortDate } = useLocaleFormatters();
const {
  createSnapshots,
  submitError,
  overwriteExistingConfirmed,
  rows,
  todayDate,
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
} = useSnapshotsAddForm({
  open,
  accountId: toRef(props, "accountId"),
  snapshots: toRef(props, "snapshots")
});

function rowState(index: number) {
  return rowStates.value[index]!;
}
</script>
