<template>
  <div class="q-py-xs flex justify-start q-px-md copy-log-btn">
    <q-btn
      label="Copy to clipboard"
      dense
      size="sm"
      no-caps
      class="q-px-sm"
      icon="content_copy"
      @click="copyLogToClipboard"
    />
  </div>
  <div class="q-pl-md">
    {
    <div
      class="log_json_content"
      v-for="(key, index) in Object.keys(value)"
      :key="key"
    >
      <q-btn-dropdown
        data-test="log-details-include-exclude-field-btn"
        size="0.5rem"
        flat
        outlined
        filled
        dense
        class="q-ml-sm pointer"
        :name="'img:' + getImageURL('images/common/add_icon.svg')"
      >
        <q-list>
          <q-item clickable v-close-popup>
            <q-item-section>
              <q-item-label
                data-test="log-details-include-field-btn"
                @click.stop="addSearchTerm(`${key}='${value[key]}'`)"
                v-close-popup
                ><q-btn
                  title="Add to search query"
                  size="6px"
                  round
                  class="q-mr-sm pointer"
                >
                  <q-icon color="currentColor">
                    <EqualIcon></EqualIcon>
                  </q-icon> </q-btn
                >Include Search Term</q-item-label
              >
            </q-item-section>
          </q-item>

          <q-item clickable v-close-popup>
            <q-item-section>
              <q-item-label
                data-test="log-details-exclude-field-btn"
                @click.stop="addSearchTerm(`${key}!='${value[key]}'`)"
                v-close-popup
                ><q-btn
                  title="Add to search query"
                  size="6px"
                  round
                  class="q-mr-sm pointer"
                >
                  <q-icon color="currentColor">
                    <NotEqualIcon></NotEqualIcon>
                  </q-icon> </q-btn
                >Exclude Search Term</q-item-label
              >
            </q-item-section>
          </q-item>
          <q-item clickable v-close-popup>
            <q-item-section>
              <q-item-label
                data-test="log-details-exclude-field-btn"
                @click.stop="addFieldToTable(key)"
                v-close-popup
                ><q-btn
                  title="Add field to table"
                  icon="visibility"
                  size="6px"
                  round
                  class="q-mr-sm pointer"
                ></q-btn
                >Add field to table</q-item-label
              >
            </q-item-section>
          </q-item>
        </q-list>
      </q-btn-dropdown>

      <span class="q-pl-xs">
        <span
          :class="store.state.theme === 'dark' ? 'text-red-5' : 'text-red-10'"
          >{{ key }}:</span
        ><span class="q-pl-xs"
          ><template v-if="index < Object.keys(value).length - 1"
            >{{ value[key] }},</template
          >
          <template v-else>
            {{ value[key] }}
          </template>
        </span>
      </span>
    </div>
    }
  </div>
</template>

<script lang="ts">
import { getImageURL } from "@/utils/zincutils";
import { useStore } from "vuex";
import EqualIcon from "@/components/icons/EqualIcon.vue";
import NotEqualIcon from "@/components/icons/NotEqualIcon.vue";

export default {
  name: "JsonPreview",
  props: {
    value: {
      type: Object,
      required: true,
      default: () => ({}),
    },
    showCopyButton: {
      type: Boolean,
      default: true,
    },
  },
  components: { NotEqualIcon, EqualIcon },
  emits: ["copy", "addSearchTerm", "addFieldToTable"],
  setup(props: any, { emit }: any) {
    const store = useStore();
    const copyLogToClipboard = () => {
      emit("copy", props.value);
    };
    const addSearchTerm = (value: string) => {
      emit("addSearchTerm", value);
    };
    const addFieldToTable = (value: string) => {
      emit("addFieldToTable", value);
    };
    return {
      copyLogToClipboard,
      getImageURL,
      addSearchTerm,
      addFieldToTable,
      store,
    };
  },
};
</script>

<style lang="scss" scoped>
.log_json_content {
  white-space: pre-wrap;
  font-family: monospace;
  font-size: 12px;
}
</style>
