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
  <div class="q-mx-md q-my-md">
    <div class="row items-center no-wrap">
      <div class="col">
        <div class="flex">
          <q-btn no-caps @click="goBack()" padding="xs" outline icon="arrow_back_ios_new" />
          <div class="text-h6 q-ml-md">
            Import Dashboard
          </div>
        </div>
      </div>
    </div>
    <q-separator class="q-my-sm" />
    <div style="width: 400px;">
      <q-form @submit="onSubmit">
        <div class="q-my-md">
          Import Dashboard from exported JSON file
        </div>
        <div style="width: 400px;">
          <q-file filled bottom-slots v-model="jsonFiles" label="Drop your file here" accept=".json" multiple
            :disable="!!isLoading">
            <template v-slot:prepend>
              <q-icon name="cloud_upload" @click.stop.prevent />
            </template>
            <template v-slot:append>
              <q-icon name="close" @click.stop.prevent="jsonFiles = null" class="cursor-pointer" />
            </template>
            <template v-slot:hint>
              .json files only
            </template>
          </q-file>
          <div>
            <div v-if="filesImportResults.length" class="q-py-sm">
              <div v-for="importResult in filesImportResults">
                <span v-if="importResult.status == 'rejected'" class="text-red">
                  <code style="background-color: #f2f1f1; padding: 3px;">{{ importResult?.reason?.file }}</code> : {{ importResult?.reason?.error }}
                </span>
              </div>
            </div>
            <q-btn :disable="!!isLoading" :loading="isLoading == ImportType.FILES" :label="t('dashboard.import')"
              class="q-my-md text-bold no-border" color="secondary" padding="sm xl" type="submit" no-caps
              @click="importFiles()" />
          </div>
        </div>
      </q-form>
      <q-separator class="q-my-sm" />

      <q-form @submit="onSubmit">
        <div class="q-my-md">
          Import Dashboard from URL
        </div>
        <div class="flex" style="width: 400px;">
          <q-input v-model="url" style="width:275px;" label="Add your url" color="input-border" bg-color="input-bg"
            stack-label filled dense label-slot :disable="!!isLoading" />
          <div>
            <q-btn :disable="!!isLoading" :loading="isLoading == ImportType.URL" class="text-bold no-border q-ml-md"
              :label="t('dashboard.import')" color="secondary" type="submit" no-caps @click="importFromUrl()"
              padding="sm xl" />
          </div>
        </div>
      </q-form>
      <q-separator class="q-my-sm" />
      <q-form @submit="onSubmit">
        <div class="q-my-md">
          Import Dashboard from JSON
        </div>
        <div style="width: 400px;" class="flex">
          <q-input :disable="!!isLoading" v-model="jsonStr" style="width: 400px;" label="JSON Object" color="input-border"
            dense filled type="textarea" />
        </div>
        <div class="q-my-md">
          <q-btn :disable="!!isLoading" :loading="isLoading == ImportType.JSON_STRING" class="text-bold no-border q-mr-md"
            :label="t('dashboard.import')" color="secondary" type="submit" padding="sm xl" no-caps
            @click="importFromJsonStr()" />
          <q-btn v-close-popup class="text-bold" :label="t('function.cancel')" text-color="light-text" padding="sm xl"
            no-caps @click="goBack()" />
        </div>
      </q-form>
    </div>
    <div>

    </div>
  </div>
</template>
<script lang="ts">
// @ts-nocheck
import { defineComponent, ref, onMounted } from "vue";
import { useI18n } from "vue-i18n";
import { getAllDashboards } from "../../utils/commons.ts";
import { useStore } from "vuex";
import { useRouter } from "vue-router";
import { useQuasar } from "quasar";
import dashboardService from "../../services/dashboards";
import axios from 'axios';

export default defineComponent({
  name: "Import Dashboard",
  props: ["dashboardId"],
  setup(props, { emit }) {
    const { t } = useI18n();
    const store = useStore()
    const router = useRouter()
    const $q = useQuasar();

    // hold the values of 3 supported import types
    const jsonFiles = ref<any>()
    const url = ref('')
    const jsonStr = ref('')

    // holds the value of the loading for any of the import type
    const isLoading = ref(false)

    // import type values
    const ImportType = {
      FILES: 'files',
      URL: 'url',
      JSON_STRING: 'json_string',
    }

    // store the results of the import (for files)
    const filesImportResults = ref([])

    onMounted(() => {
      filesImportResults.value = []
    });

    //import dashboard from the json
    const importDashboardFromJSON = async (jsonObj: any) => {
      const data = typeof jsonObj == 'string' ? JSON.parse(jsonObj) : typeof jsonObj == 'object' ? jsonObj : jsonObj

      //set owner name and creator name to import dashboard
      data.owner = store.state.userInfo.name
      data.created = new Date().toISOString()

      return dashboardService.create(
        store.state.selectedOrganization.identifier,
        data
      )
    }

    // import multiple files
    const importFiles = async () => {
      isLoading.value = ImportType.FILES

      const data = jsonFiles?.value?.map((it: any, index: number) => {
        return new Promise((resolve, reject) => {
          let reader = new FileReader();
          reader.onload = function (readerResult) {
            try {
              importDashboardFromJSON(readerResult.target.result)
                .then((res) => {
                  jsonFiles.value = null
                  resolve({ file: it.name, result: res })
                }).catch((e) => {
                  reject({ file: it.name, error: e })
                })
            } catch (e) {
              reject({ file: it.name, error: 'Error reading file' })

            }
          };
          reader.readAsText(it)
        })
      })

      Promise.allSettled(data)
        .then(async (results) => {
          filesImportResults.value = results

          const allFulfilledValues = results
            .filter(r => r.status === 'fulfilled')?.length

          if (results.length == allFulfilledValues) {
            await resetAndRefresh(ImportType.FILES);
          }

          if(allFulfilledValues){
            
            $q.notify({
              type: "positive",
              message: `${allFulfilledValues} Dashboard(s) Imported Successfully.`,
            });
          }

          if(results.length-allFulfilledValues){
            
            $q.notify({
              type: "negative",
              message: `${results.length-allFulfilledValues} Dashboard(s) Failed to Import.`,
            });
          }
          

          isLoading.value = false
        });

    }

    // reset and refresh the value based on selected type 
    const resetAndRefresh = async (type) => {
      switch (type) {
        case ImportType.FILES:
          jsonFiles.value = null
          isLoading.value = false
          break
        case ImportType.URL:
          url.value = ''
          isLoading.value = false
          break
        case ImportType.JSON_STRING:
          jsonStr.value = ''
          isLoading.value = false
          break
        default:
          break
      }

      return getAllDashboards(store).then(() => {
        return getAllDashboards(store)
      }).then(() => {
        router.push('/dashboards')
      })
    }

    //import dashboard from url
    const importFromUrl = async () => {
      isLoading.value = ImportType.URL
      try {
        // get the dashboard
        const urlData = url.value ? url.value : ''

        const res = await axios.get(urlData);
        await importDashboardFromJSON(res.data)
          .then((res) => {
            resetAndRefresh(ImportType.URL);
            filesImportResults.value = []
            $q.notify({
              type: "positive",
              message: `Dashboard Imported Successfully`,
            });
          })
      } catch (error) {
        $q.notify({
          type: "negative",
          message: 'Invalid JSON format',
        });
        
      } finally {
        isLoading.value = false
      }
    }

    // import dashboard from json string
    const importFromJsonStr = async () => {
      isLoading.value = ImportType.JSON_STRING
      try {
        // get the dashboard
        await importDashboardFromJSON(jsonStr.value)
          .then((res) => {
            resetAndRefresh(ImportType.JSON_STRING);
            filesImportResults.value = []
            $q.notify({
                type: "positive",
                message: `Dashboard Imported Successfully`,
            });
          })
      } catch (error) {
        $q.notify({
          type: "negative",
          message: 'Invalid JSON format',
        });
       
      } finally {
        isLoading.value = false
      }
    }

    // back button to render dashboard List page
    const goBack = () => {
      jsonFiles.value = ''
      url.value = ''
      jsonStr.value = ''
      filesImportResults.value = []
      return router.push("/dashboards");
    };

    const onSubmit = () => {
      // do nothing here
    }

    return {
      t,
      goBack,
      onSubmit,
      importFiles,
      jsonFiles,
      importFromUrl,
      url,
      jsonStr,
      importFromJsonStr,
      isLoading,
      ImportType,
      filesImportResults
    }
  }
})
</script>
