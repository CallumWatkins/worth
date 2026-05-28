<template>
  <UModal
    v-model:open="open"
    title="Licenses"
    description="License notices for Worth and the third-party software included in this app."
    :ui="{
      content: 'max-w-3xl h-[calc(100dvh-2rem)] max-h-[calc(100dvh-2rem)]',
      body: 'flex min-h-0 sm:p-0 p-0'
    }"
  >
    <template #body>
      <div v-if="licensesQuery.isPending" class="flex min-h-48 w-full items-center justify-center px-4 py-4 text-sm text-muted sm:px-6 sm:py-6">
        Loading license notices...
      </div>

      <div v-else-if="licensesQuery.isError" class="px-4 py-4 sm:px-6 sm:py-6 w-full">
        <UAlert
          color="warning"
          variant="subtle"
          title="License notices unavailable"
          :description="licensesQuery.error.message"
        />
      </div>

      <UScrollArea
        v-else-if="licensesQuery.isSuccess"
        :items="licenseRows"
        :virtualize="{
          estimateSize: estimateLicenseRowSize,
          getItemKey: (index: number) => licenseRows[index]?.id ?? index,
          overscan: 8,
          gap: 8
        }"
        class="min-h-0 flex-1 w-full px-4 py-4 sm:px-6 sm:py-6"
      >
        <template #default="{ item }">
          <section
            v-if="item.kind === 'app' && appLicenseNotice"
            class="space-y-4 rounded-lg ring ring-default p-3"
          >
            <dl class="grid gap-2 text-sm sm:grid-cols-3">
              <div>
                <dt class="text-muted">
                  Name
                </dt>
                <dd class="font-medium text-highlighted wrap-break-word">
                  {{ appLicenseNotice.name }}
                </dd>
              </div>
              <div>
                <dt class="text-muted">
                  Version
                </dt>
                <dd class="font-medium text-highlighted wrap-break-word">
                  {{ appLicenseNotice.version }}
                </dd>
              </div>
              <div>
                <dt class="text-muted">
                  License
                </dt>
                <dd class="font-medium text-highlighted wrap-break-word">
                  {{ appLicenseNotice.license }}
                </dd>
              </div>
            </dl>

            <div class="space-y-2">
              <div class="text-sm text-muted">
                License Text
              </div>
              <pre tabindex="0" aria-label="Worth license text" class="max-h-96 overflow-auto whitespace-pre-wrap rounded-lg bg-elevated/50 p-3 text-xs leading-relaxed text-toned select-text">{{ appLicenseNotice.licenseText }}</pre>
            </div>

            <div class="space-y-2">
              <div class="text-sm text-muted">
                Brand Notice
              </div>
              <pre tabindex="0" aria-label="Worth brand notice" class="whitespace-pre-wrap rounded-lg bg-elevated/50 p-3 text-xs leading-relaxed text-toned select-text">{{ appLicenseNotice.brandNotice }}</pre>
            </div>
          </section>

          <div v-else-if="item.kind === 'summary'" class="pt-2">
            <h3 class="text-base font-semibold text-highlighted">
              Third-party packages
            </h3>
            <p class="mt-1 text-sm text-muted">
              {{ thirdPartyPackages.length }} package{{ thirdPartyPackages.length === 1 ? '' : 's' }}
            </p>
          </div>

          <UCollapsible
            v-else-if="item.kind === 'package'"
            :open="isPackageExpanded(item.packageItem.value)"
            class="overflow-hidden rounded-lg ring ring-default"
            @update:open="setPackageExpanded(item.packageItem.value, $event)"
          >
            <UButton
              type="button"
              color="neutral"
              variant="ghost"
              trailing-icon="i-lucide-chevron-down"
              class="group w-full justify-between gap-3 rounded-lg px-3 py-3 text-left hover:bg-transparent active:bg-transparent data-[state=open]:bg-transparent data-[state=open]:hover:bg-transparent"
              :class="isPackageExpanded(item.packageItem.value) ? 'rounded-b-none' : undefined"
              :ui="{
                base: 'focus-visible:ring-inset',
                label: 'min-w-0',
                trailingIcon: 'size-5 shrink-0 text-muted transition-transform duration-200 group-data-[state=open]:rotate-180'
              }"
            >
              <span class="min-w-0">
                <span class="block truncate text-sm font-medium text-highlighted">
                  {{ item.packageItem.name }}
                </span>
                <span class="mt-0.5 block truncate text-xs text-muted">
                  Version {{ item.packageItem.version }}
                </span>
              </span>
            </UButton>

            <template #content>
              <div class="space-y-4 px-3 pb-4">
                <dl class="grid gap-2 text-sm sm:grid-cols-3">
                  <div>
                    <dt class="text-muted">
                      Name
                    </dt>
                    <dd class="font-medium text-highlighted wrap-break-word">
                      {{ item.packageItem.name }}
                    </dd>
                  </div>
                  <div>
                    <dt class="text-muted">
                      Version
                    </dt>
                    <dd class="font-medium text-highlighted wrap-break-word">
                      {{ item.packageItem.version }}
                    </dd>
                  </div>
                  <div>
                    <dt class="text-muted">
                      License
                    </dt>
                    <dd class="font-medium text-highlighted wrap-break-word">
                      {{ item.packageItem.license }}
                    </dd>
                  </div>
                </dl>

                <div
                  v-for="(text, textIndex) in item.packageItem.texts"
                  :key="`${item.packageItem.value}:${text.title}`"
                  class="space-y-2"
                >
                  <div class="text-sm text-muted">
                    {{ text.kind === 'notice' ? 'Notice Text' : 'License Text' }}{{ duplicateTextLabel(item.packageItem.texts, text, textIndex) }}
                  </div>
                  <pre tabindex="0" :aria-label="`${item.packageItem.name} ${text.kind} text`" class="max-h-80 overflow-auto whitespace-pre-wrap rounded-lg bg-elevated/50 p-3 text-xs leading-relaxed text-toned select-text">{{ text.text }}</pre>
                </div>
              </div>
            </template>
          </UCollapsible>
        </template>
      </UScrollArea>
    </template>
  </UModal>
</template>

<script setup lang="ts">
import { useQuery } from "@tanstack/vue-query";

interface LicenseText {
  title: string
  kind: "license" | "notice"
  text: string
}

interface LicenseNoticePackage {
  ecosystem: string
  name: string
  version: string
  license: string
  texts: LicenseText[]
}

interface LicenseNotices {
  app: {
    name: string
    version: string
    license: string
    brandNotice: string
    licenseText: string
  }
  thirdParty: LicenseNoticePackage[]
}

interface LicensePackageItem extends LicenseNoticePackage {
  value: string
}

type LicenseRow
  = | { id: "worth", kind: "app" }
    | { id: "third-party-summary", kind: "summary" }
    | { id: string, kind: "package", packageItem: LicensePackageItem };

const open = defineModel<boolean>("open", { required: true });
const expandedPackageValues = ref(new Set<string>());

const licensesQuery = proxyRefs(useQuery<LicenseNotices, Error>({
  queryKey: ["license-notices"],
  queryFn: fetchLicenseNotices,
  enabled: computed(() => open.value)
}));

const appLicenseNotice = computed(() => licensesQuery.data?.app);

const thirdPartyPackages = computed<LicensePackageItem[]>(() => licensesQuery.data?.thirdParty
  .map((noticePackage) => ({
    ...noticePackage,
    value: `${noticePackage.ecosystem}:${noticePackage.name}@${noticePackage.version}`
  })) ?? []);

const licenseRows = computed<LicenseRow[]>(() => {
  if (!licensesQuery.data) return [];

  return [
    { id: "worth", kind: "app" },
    { id: "third-party-summary", kind: "summary" },
    ...thirdPartyPackages.value.map((packageItem) => ({
      id: packageItem.value,
      kind: "package" as const,
      packageItem
    }))
  ];
});

watch(open, (isOpen) => {
  if (!isOpen) {
    expandedPackageValues.value = new Set();
    return;
  }

  if (licensesQuery.isError) {
    void licensesQuery.refetch();
  }
});

function estimateLicenseRowSize(index: number) {
  const row = licenseRows.value[index];
  if (row?.kind === "app") return 612;
  if (row?.kind === "summary") return 48;
  if (row?.kind === "package") {
    if (!isPackageExpanded(row.packageItem.value)) return 62;

    return 482 + Math.max(0, row.packageItem.texts.length - 1) * 364;
  }

  return 62;
}

function isPackageExpanded(value: string) {
  return expandedPackageValues.value.has(value);
}

function setPackageExpanded(value: string, isExpanded: boolean) {
  if (isExpanded) {
    expandedPackageValues.value.add(value);
  } else {
    expandedPackageValues.value.delete(value);
  }
}

function duplicateTextLabel(texts: LicenseText[], text: LicenseText, textIndex: number) {
  const sameKindTexts = texts.filter((entry) => entry.kind === text.kind);
  if (sameKindTexts.length === 1) return "";

  const sameKindIndex = texts.slice(0, textIndex + 1).filter((entry) => entry.kind === text.kind).length;
  return ` ${sameKindIndex}`;
}

async function fetchLicenseNotices() {
  const noticesPath = "/licenses/notices.json.gz";

  let response: Response;
  try {
    response = await fetch(noticesPath, { cache: "no-store" });
  } catch {
    throw new Error("Failed to fetch license notices.");
  }

  if (response.status === 404 || (response.ok && response.headers.get("content-type")?.includes("text/html"))) {
    throw new Error("License notices have not been generated for this build.");
  }

  if (!response.ok) {
    throw new Error(`Failed to load ${noticesPath}: ${response.status} ${response.statusText}`);
  }

  if (response.headers.get("content-encoding")?.includes("gzip")) {
    return response.json() as Promise<LicenseNotices>;
  }

  if (!response.body) throw new Error("License notice response body is empty.");
  return new Response(response.body.pipeThrough(new DecompressionStream("gzip"))).json() as Promise<LicenseNotices>;
}
</script>
