<template>
  <CollapsibleRoot v-if="identifiers.length" v-model:open="open" class="w-full max-w-xl">
    <div class="flex flex-wrap items-center justify-between gap-x-4 gap-y-2 pe-2 overflow-hidden scrollbar-gutter-stable">
      <CollapsibleTrigger as-child>
        <UButton
          label="Account details"
          icon="i-lucide-chevron-right"
          :ui="{ leadingIcon: ['transition-transform duration-200 ease-out motion-reduce:transition-none', open && 'rotate-90'] }"
          color="neutral"
          variant="link"
          class="px-0"
        />
      </CollapsibleTrigger>
      <Transition
        enter-active-class="transition-opacity duration-200 ease-out motion-reduce:transition-none"
        leave-active-class="transition-opacity duration-200 ease-out motion-reduce:transition-none"
        enter-from-class="opacity-0"
        leave-to-class="opacity-0"
      >
        <div v-show="open" :inert="!open" class="ms-auto flex shrink-0 items-center gap-1">
          <UTooltip text="Edit account details">
            <UButton icon="i-lucide-pencil" aria-label="Edit account details" color="neutral" variant="ghost" size="sm" :to="settingsLink" />
          </UTooltip>
          <UTooltip v-if="identifiers.length > 1" :text="allRevealed ? 'Hide all' : 'Show all'">
            <UButton
              :icon="allRevealed ? 'i-lucide-eye-off' : 'i-lucide-eye'"
              :aria-label="allRevealed ? 'Hide all identifiers' : 'Show all identifiers'"
              :aria-pressed="allRevealed"
              color="neutral"
              variant="ghost"
              size="sm"
              @click="revealed = new Set(allRevealed ? [] : identifiers.map((_, index) => index))"
            />
          </UTooltip>
          <UTooltip v-if="identifiers.length > 1" text="Copy all">
            <UButton
              :icon="copiedAll ? 'i-lucide-check' : 'i-lucide-copy'"
              aria-label="Copy all identifiers"
              color="neutral"
              variant="ghost"
              size="sm"
              @click="copyAll"
            />
          </UTooltip>
        </div>
      </Transition>
    </div>
    <CollapsibleContent class="overflow-hidden motion-safe:data-[state=open]:animate-[collapsible-down_200ms_var(--ease-out),fade-in_200ms_var(--ease-out)] motion-safe:data-[state=closed]:animate-[collapsible-up_200ms_var(--ease-out),fade-out_200ms_var(--ease-out)]">
      <UScrollArea
        :items="identifiers"
        :virtualize="{ estimateSize: 56, overscan: 8 }"
        class="max-h-[min(24rem,50vh)] overscroll-y-contain pt-3 pe-2 ms-6.5 scrollbar-gutter-stable"
        tabindex="0"
        role="region"
        aria-label="Account identifiers"
      >
        <template #default="{ item: identifier, index }">
          <dl
            class="grid grid-cols-1 min-[480px]:grid-cols-[8rem_minmax(0,1fr)] items-center gap-x-3.5 gap-y-1 py-3 border-default"
            :class="{ 'border-t': index > 0 }"
          >
            <dt class="text-sm text-muted min-w-0 wrap-anywhere">
              {{ identifier.label }}
            </dt>
            <dd class="flex min-w-0 items-center gap-1">
              <span
                class="min-w-0 flex-1 font-mono text-sm wrap-anywhere"
                :class="[
                  revealed.has(index) && identifier.value ? 'select-text' : 'select-none',
                  { 'text-muted': revealed.has(index) && !identifier.value }
                ]"
                dir="auto"
              >{{ revealed.has(index) ? (identifier.value || '(no value)') : '••••••••' }}</span>
              <UButton
                :icon="revealed.has(index) ? 'i-lucide-eye-off' : 'i-lucide-eye'"
                color="neutral"
                variant="ghost"
                size="sm"
                class="shrink-0"
                :aria-label="`${revealed.has(index) ? 'Hide' : 'Reveal'} ${identifier.label}`"
                :aria-pressed="revealed.has(index)"
                @click="revealed.has(index) ? revealed.delete(index) : revealed.add(index)"
              />
              <UButton
                :icon="copiedIndex === index ? 'i-lucide-check' : 'i-lucide-copy'"
                color="neutral"
                variant="ghost"
                size="sm"
                class="shrink-0"
                :aria-label="`Copy ${identifier.label}`"
                @click="copyValue(index, identifier.value)"
              />
            </dd>
          </dl>
        </template>
      </UScrollArea>
      <span class="sr-only" role="status">{{ copiedAll ? 'Account details copied' : copiedIndex == null ? '' : 'Copied' }}</span>
    </CollapsibleContent>
  </CollapsibleRoot>
  <UButton v-else label="Add account details" icon="i-lucide-plus" color="neutral" variant="link" :to="settingsLink" class="px-0" />
</template>

<script setup lang="ts">
import type { AccountIdentifierInput } from "~/generated/bindings";
import { CollapsibleContent, CollapsibleRoot, CollapsibleTrigger } from "reka-ui";

const props = defineProps<{ accountId: number, identifiers: AccountIdentifierInput[] }>();
const settingsLink = computed(() => ({ name: "accounts-id-settings" as const, params: { id: props.accountId }, hash: "#account-details" }));
const open = ref(false);
const revealed = ref(new Set<number>());
const allRevealed = computed(() => props.identifiers.every((_, index) => revealed.value.has(index)));
const copiedIndex = ref<number | null>(null);
const copiedAll = ref(false);
const toast = useToast();
const { start, stop } = useTimeoutFn(() => {
  copiedIndex.value = null;
  copiedAll.value = false;
}, 2000, { immediate: false });

watch([() => props.identifiers, open], () => {
  revealed.value.clear();
  copiedIndex.value = null;
  copiedAll.value = false;
  stop();
});

async function copyValue(index: number, value: string) {
  try {
    await navigator.clipboard.writeText(value);
    copiedIndex.value = index;
    copiedAll.value = false;
    start();
  } catch {
    toast.add({ title: "Could not copy account detail", color: "error" });
  }
}

async function copyAll() {
  const rows = [["Identifier", "Value"], ...props.identifiers.map(({ label, value }) => [label, value])];
  const text = rows.map((row) => row.join("\t")).join("\n");
  const table = document.createElement("table");
  const header = table.createTHead();
  const body = table.createTBody();
  rows.forEach((values, index) => {
    const row = (index === 0 ? header : body).insertRow();
    values.forEach((value) => {
      const cell = document.createElement(index === 0 ? "th" : "td");
      cell.textContent = value;
      row.append(cell);
    });
  });

  try {
    if (typeof ClipboardItem !== "undefined" && typeof navigator.clipboard?.write === "function") {
      try {
        await navigator.clipboard.write([new ClipboardItem({
          "text/html": new Blob([table.outerHTML], { type: "text/html" }),
          "text/plain": new Blob([text], { type: "text/plain" })
        })]);
      } catch {
        await navigator.clipboard.writeText(text);
      }
    } else {
      await navigator.clipboard.writeText(text);
    }
    copiedIndex.value = null;
    copiedAll.value = true;
    start();
  } catch {
    toast.add({ title: "Could not copy account details", color: "error" });
  }
}
</script>
