// Copyright 2023 Zinc Labs Inc.

//  Licensed under the Apache License, Version 2.0 (the "License");
//  you may not use this file except in compliance with the License.
//  You may obtain a copy of the License at

//      http:www.apache.org/licenses/LICENSE-2.0

//  Unless required by applicable law or agreed to in writing, software
//  distributed under the License is distributed on an "AS IS" BASIS,
//  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//  See the License for the specific language governing permissions and
//  limitations under the License.

import { ref } from "vue";
import organizationsService from "../services/organizations";
import { useLocalOrganization, getPath } from "./zincutils";

const selectedOrg = ref("");
const orgOptions = ref([{ label: Number, value: String }]);

export const getDefaultOrganization = async (
  userInfo: any,
  org_identifier: any
) => {
  await organizationsService
    .os_list(0, 1000, "id", false, "", org_identifier)
    .then((res: any) => {
      const localOrg: any = useLocalOrganization();
      if (
        localOrg.value != null &&
        localOrg.value.user_email !== userInfo.email
      ) {
        localOrg.value = null;
        useLocalOrganization("");
      }

      orgOptions.value = res.data.data.map(
        (data: {
          id: any;
          name: any;
          org_type: any;
          identifier: any;
          user_obj: any;
          ingest_threshold: number;
          search_threshold: number;
          note: string;
        }) => {
          const optiondata: any = {
            label: data.name,
            id: data.id,
            type: data.org_type,
            identifier: data.identifier,
            user_email: userInfo.email,
            ingest_threshold: data.ingest_threshold,
            search_threshold: data.search_threshold,
            note: data.note,
          };

          if (
            ((selectedOrg.value == "" || selectedOrg.value == undefined) &&
              data.org_type == "default" &&
              userInfo.email == data.user_obj.email) ||
            res.data.data.length == 1
          ) {
            selectedOrg.value = localOrg.value ? localOrg.value : optiondata;
            useLocalOrganization(selectedOrg.value);
            //   $store.dispatch("setSelectedOrganization", selectedOrg.value);
          }
          return optiondata;
        }
      );
      return res.data.data;
    });
};

export const redirectUser = (redirectURI: string | null) => {
  const path = getPath();
  if (redirectURI != null && redirectURI != "") {
    // $router.push({ path: redirectURI });
    window.location.replace(path);
  } else {
    // $router.push({ path: "/" });
    window.location.replace(path);
  }
};

export const logsErrorMessage = (code: number) => {
  const messages: any = {
    10001: "ServerInternalError",
    20001: "SearchSQLNotValid",
    20002: "SearchStreamNotFound",
    20003: "FullTextSearchFieldNotFound",
    20004: "SearchFieldNotFound",
    20005: "SearchFunctionNotDefined",
    20006: "SearchParquetFileNotFound",
    20007: "SearchFieldHasNoCompatibleDataType",
    20008: "SearchSQLExecuteError",
  };

  if (messages[code] != undefined) {
    return "message." + messages[code];
  } else {
    return "";
  }
};
