<!-- Copyright 2023 Zinc Labs Inc.

 Licensed under the Apache License, Version 2.0 (the "License");
 you may not use this file except in compliance with the License.
 You may obtain a copy of the License at

     http:www.apache.org/licenses/LICENSE-2.0

 Unless required by applicable law or agreed to in writing, software
 distributed under the License is distributed on an "AS IS" BASIS,
 WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 See the License for the specific language governing permissions and
 limitations under the License. 
-->

<template>
  <q-card class="column full-height">
    <q-card-section class="q-px-md q-py-md">
      <div class="row items-center no-wrap">
        <div class="col">
          <div
            v-if="beingUpdated"
            class="text-body1 text-bold"
            data-test="update-org"
          >
            {{ t("organization.updateOrganization") }}
          </div>
          <div
            v-else
            class="text-body1 text-bold"
            data-test="create-org"
          >
            {{ t("organization.createOrganization") }}
          </div>
        </div>
        <div class="col-auto">
          <q-btn
            v-close-popup
            round
            flat
            icon="img:/src/assets/images/common/close_icon.svg"
            @click="router.replace({ name: 'organizations' })"
          />
        </div>
      </div>
    </q-card-section>
    <q-separator />
    <q-card-section class="q-w-md q-mx-lg">
      <q-form ref="addOrganizationForm" @submit="onSubmit">
        <q-input
          v-if="beingUpdated"
          v-model="organizationData.id"
          :readonly="beingUpdated"
          :disabled="beingUpdated"
          :label="t('organization.id')"
        />

        <q-input
          v-model="organizationData.name"
          :placeholder="t('organization.nameHolder')"
          :label="t('organization.name') + '*'"
          color="input-border"
          bg-color="input-bg"
          class="q-py-md showLabelOnTop"
          stack-label
          outlined
          filled
          dense
          :rules="[(val) => !!val || t('organization.nameRequired')]"
          data-test="org-name"
        />

        <div class="flex justify-center q-mt-lg">
          <q-btn
            v-close-popup
            class="q-mb-md text-bold"
            :label="t('organization.cancel')"
            text-color="light-text"
            padding="sm md"
            no-caps
            @click="router.replace({ name: 'organizations' })"
          />
          <q-btn
            :disable="organizationData.name === '' && !proPlanRequired"
            :label="t('organization.save')"
            class="q-mb-md text-bold no-border q-ml-md"
            color="secondary"
            padding="sm xl"
            type="submit"
            no-caps
            data-test="add-org"
          />
        </div>

        <div class="flex justify-center q-mt-lg" v-if="proPlanRequired">
          <q-btn
            class="q-mb-md text-bold no-border q-ml-md"
            :label="t('organization.proceed_subscription')"
            text-color="light-text"
            padding="sm xl"
            color="secondary"
            no-caps
            @click="completeSubscriptionProcess"
          />
        </div>
      </q-form>
    </q-card-section>
  </q-card>
</template>

<script lang="ts">
import { defineComponent, ref } from "vue";
import organizationService from "@/services/organizations";
import { useI18n } from "vue-i18n";
import { useStore } from "vuex";
import { useRouter } from "vue-router";
import config from "@/aws-exports";

const defaultValue = () => {
  return {
    id: "",
    name: "",
  };
};

let callOrganization: Promise<{ data: any }>;

export default defineComponent({
  name: "ComponentAddUpdateUser",
  props: {
    modelValue: {
      type: Object,
      default: () => defaultValue(),
    },
  },
  data() {
    return {
      proPlanRequired: false,
      proPlanMsg: "",
      newOrgIdentifier: "",
    };
  },
  emits: ["update:modelValue", "updated", "finish"],
  setup() {
    const store: any = useStore();
    const router: any = useRouter();
    const beingUpdated: any = ref(false);
    const addOrganizationForm: any = ref(null);
    const disableColor: any = ref("");
    const organizationData: any = ref(defaultValue());
    const isValidIdentifier: any = ref(true);
    const { t } = useI18n();

    return {
      t,
      router,
      disableColor,
      isPwd: ref(true),
      beingUpdated,
      organizationData,
      addOrganizationForm,
      store,
      isValidIdentifier,
    };
  },
  created() {
    if (this.modelValue && this.modelValue.id) {
      this.beingUpdated = true;
      this.disableColor = "grey-5";
      this.organizationData = {
        id: this.modelValue.id,
        name: this.modelValue.name,
      };
    }

    // this.store.state.organizations.forEach((organization: any) => {
    //   if (
    //     (organization.hasOwnProperty("CustomerBillingObj") &&
    //       organization.CustomerBillingObj.subscription_type ==
    //         config.freePlan) ||
    //     !organization.hasOwnProperty("CustomerBillingObj")
    //   ) {
    //     this.proPlanRequired = true;
    //   }
    // });
  },
  methods: {
    onRejected(rejectedEntries: string | any[]) {
      this.$q.notify({
        type: "negative",
        message: `${rejectedEntries.length} file(s) did not pass validation constraints`,
      });
    },
    completeSubscriptionProcess() {
      console.log(this.store.state);
      // this.store.state.dispatch("setSelectedOrganization",)
      this.router.push(
        `/billings/plans?org_identifier=${this.newOrgIdentifier}`
      );
    },
    onSubmit() {
      const dismiss = this.$q.notify({
        spinner: true,
        message: "Please wait...",
        timeout: 2000,
      });
      this.addOrganizationForm.validate().then((valid: any) => {
        if (!valid) {
          return false;
        }

        const organizationId = this.organizationData.id;
        delete this.organizationData.id;

        if (organizationId == "") {
          callOrganization = organizationService.create(this.organizationData);
        }
        // else {
        //   callOrganization = organizationService.update(
        //     organizationId,
        //     this.organizationData
        //   );
        // }

        callOrganization
          .then((res: { data: any }) => {
            const data = res.data;
            if (res.data.data.status == "active") {
              this.organizationData = {
                id: "",
                name: "",
              };

              this.$emit("update:modelValue", data);
              this.$emit("updated");
              this.addOrganizationForm.resetValidation();
              dismiss();
            } else {
              this.proPlanRequired = true;
              this.proPlanMsg = res.data.message;
              this.newOrgIdentifier = res.data.identifier;
              // this.store.state.dispatch("setSelectedOrganization", {
              //   identifier: data.identifier,
              //   name: data.name,
              //   id: data.id,
              //   ingest_threshold: data.ingest_threshold,
              //   search_threshold: data.search_threshold,
              //   label: data.name,
              //   user_email: this.store.state.userInfo.email,
              //   subscription_type: "Free-Plan-USD-Monthly",
              // });
              // window.location.href = `/organizations?org_identifier=${data.data.identifier}&action=subscribe`;
              this.router.push({
                name: "organizations",
                query: {
                  org_identifier: data.data.identifier,
                  action: "subscribe",
                  update_org: Date.now(),
                },
              })
            }
          })
          .catch((err: any) => {
            this.$q.notify({
              type: "negative",
              message: JSON.stringify(
                err.response.data["error"] || "Organization creation failed."
              ),
            });
            dismiss();
          });
      });
    },
  },
});
</script>
