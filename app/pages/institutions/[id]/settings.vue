<template>
  <UContainer>
    <div v-if="institutionQuery.data" class="pt-6">
      <UBreadcrumb :items="breadcrumbItems" />
    </div>

    <UPageHeader
      v-if="institutionQuery.data"
      title="Institution settings"
      :description="institutionQuery.data.name"
      :ui="{
        root: 'pb-0 border-none',
        description: 'mt-1'
      }"
    />

    <UPageBody class="space-y-6">
      <UAlert
        v-if="invalidId"
        color="error"
        variant="subtle"
        title="Invalid institution id"
      />

      <UAlert
        v-else-if="institutionQuery.isError"
        color="error"
        variant="subtle"
        :title="institutionQuery.error!.message ?? 'Failed to load institution'"
      />

      <template v-else-if="institutionQuery.data">
        <UPageCard title="General" description="Update institution details">
          <UForm
            ref="form"
            :schema="institutionFormSchema"
            :state="state"
            :validate-on="['blur']"
            class="space-y-4"
            @submit="onSubmit"
          >
            <UAlert
              v-if="submitError"
              color="error"
              variant="subtle"
              :title="submitError"
            />

            <UAlert
              v-if="didSave"
              color="success"
              variant="subtle"
              title="Institution updated"
            />

            <UFormField label="Institution name" name="name">
              <UInput
                v-model="state.name"
                class="w-full"
              />
            </UFormField>

            <div class="flex justify-end">
              <UButton
                type="submit"
                color="primary"
                :loading="isSubmitting"
                :disabled="isSubmitting"
              >
                Save changes
              </UButton>
            </div>
          </UForm>
        </UPageCard>

        <UPageCard
          title="Actions"
          description="Additional actions are planned for this page."
        >
          <div class="flex flex-wrap gap-2">
            <UButton
              color="neutral"
              variant="subtle"
              icon="i-lucide-download"
              disabled
            >
              Export (coming soon)
            </UButton>
            <UButton
              color="error"
              variant="subtle"
              icon="i-lucide-trash-2"
              disabled
            >
              Delete (coming soon)
            </UButton>
          </div>
        </UPageCard>
      </template>
    </UPageBody>
  </UContainer>
</template>

<script lang="ts" setup>
import type { BreadcrumbItem, FormSubmitEvent } from "@nuxt/ui";
import { useQuery } from "@tanstack/vue-query";

const route = useRoute();
const api = useApi();
const { updateInstitution } = useInstitutionMutations();
const form = useTemplateRef("form");
const setBackendValidationErrors = useBackendValidationErrors(form);

const rawId = computed(() => {
  const p = (route.params as any)?.id;
  if (Array.isArray(p)) return p[0];
  return p;
});

const institutionId = computed<number | null>(() => {
  const s = String(rawId.value ?? "");
  const n = Number.parseInt(s, 10);
  if (!Number.isFinite(n)) return null;
  return n;
});

const invalidId = computed(() => rawId.value != null && institutionId.value == null);

const institutionQuery = proxyRefs(useQuery({
  queryKey: computed(() => queryKeys.institutions.get(institutionId.value!)),
  enabled: computed(() => institutionId.value !== null),
  queryFn: () => api.institutionsGet(institutionId.value!)
}));

const submitError = ref<string | null>(null);
const didSave = ref(false);
const hasHydrated = ref(false);
const { state, hydrateFromInstitution } = useInstitutionUpsertForm();

watch(institutionId, () => {
  hasHydrated.value = false;
});

watch(() => institutionQuery.data, (institution) => {
  if (!institution || hasHydrated.value) return;
  hydrateFromInstitution(institution.name);
  hasHydrated.value = true;
  form.value?.clear();
}, { immediate: true });

const isSubmitting = computed(() => updateInstitution.isPending.value);

const breadcrumbItems = computed<BreadcrumbItem[]>(() => {
  const institution = institutionQuery.data;
  if (!institution) return [];
  return [
    { label: "Institutions", to: "/institutions", icon: "i-lucide-building-2" },
    { label: institution.name, to: `/institutions/${institution.id}` },
    { label: "Settings" }
  ];
});

async function onSubmit(event: FormSubmitEvent<InstitutionFormValues>) {
  if (!institutionId.value) return;

  submitError.value = null;
  didSave.value = false;

  try {
    await updateInstitution.mutateAsync({
      institutionId: institutionId.value,
      input: {
        name: event.data.name
      }
    });
    didSave.value = true;
  } catch (error) {
    if (!setBackendValidationErrors(error)) {
      submitError.value = error instanceof Error ? error.message : "Failed to update institution";
    }
  }
}
</script>
