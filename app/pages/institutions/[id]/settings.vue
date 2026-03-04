<template>
  <UContainer>
    <div v-if="institutionQuery.isSuccess" class="pt-6">
      <UBreadcrumb :items="breadcrumbItems" />
    </div>

    <UPageHeader
      v-if="institutionQuery.isSuccess"
      title="Institution settings"
      :description="institutionQuery.data.name"
      :ui="{
        root: 'pb-0 border-none',
        description: 'mt-1'
      }"
    />

    <UPageBody class="space-y-6">
      <template v-if="institutionQuery.isSuccess">
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

            <UFormField label="Institution name" name="name">
              <UInput
                v-model="state.name"
                class="w-full"
              />
            </UFormField>

            <div class="flex items-center justify-end gap-3">
              <Transition name="save-status-fade">
                <span v-if="didSave && !form?.dirty" class="text-sm text-success">
                  Changes saved
                </span>
              </Transition>
              <UButton
                type="submit"
                color="primary"
                :disabled="form?.loading"
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

const route = useRoute("institutions-id-settings");
const api = useApi();
const { updateInstitution } = useInstitutionMutations();
const form = useTemplateRef("form");
const setBackendValidationErrors = useBackendValidationErrors(form);

const institutionId = useRouteParamInt(route, "id");

const institutionQuery = proxyRefs(useQuery({
  queryKey: computed(() => queryKeys.institutions.get(institutionId.value!)),
  enabled: computed(() => institutionId.value !== null),
  queryFn: () => api.institutionsGet(institutionId.value!)
}));

useResourcePageError({
  resourceName: "Institution",
  resourceId: institutionId,
  resourceIsError: computed(() => institutionQuery.isError),
  resourceError: computed(() => institutionQuery.error),
  fallbackErrorMessage: "Failed to load institution"
});

const submitError = ref<string | null>(null);
const didSave = ref(false);
const hasHydrated = ref(false);
const { state, hydrateFromInstitution } = useInstitutionUpsertForm();

watch(institutionId, () => {
  hasHydrated.value = false;
  didSave.value = false;
});

watch(() => institutionQuery.data, (institution) => {
  if (!institution || hasHydrated.value) return;
  hydrateFromInstitution(institution);
  hasHydrated.value = true;
  form.value?.clear();
}, { immediate: true });

usePreventRouteNavigation({
  isSubmitting: computed(() => form.value?.loading ?? false),
  isDirty: computed(() => form.value?.dirty ?? false),
  title: "Discard institution changes?",
  description: "You have unsaved institution changes that will be lost if you leave this page."
});

const breadcrumbItems = computed<BreadcrumbItem[]>(() => {
  const institution = institutionQuery.data;
  if (!institution) return [];
  return [
    { label: "Institutions", to: { name: "institutions" }, icon: "i-lucide-building-2" },
    { label: institution.name, to: { name: "institutions-id", params: { id: institution.id } } },
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
    throw error;
  }
}
</script>
