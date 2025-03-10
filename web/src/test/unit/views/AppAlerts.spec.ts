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

import { describe, expect, it, beforeEach, vi, afterEach } from "vitest";
import { mount } from "@vue/test-utils";
import { installQuasar } from "../helpers/install-quasar-plugin";
import { Dialog, Notify } from "quasar";
import AppAlerts from "../../../views/AppAlerts.vue";
import i18n from "../../../locales";
import store from "../helpers/store";
import router from "../helpers/router";

installQuasar({
  plugins: [Dialog, Notify],
});

describe("Streams", async () => {
  let wrapper: any;
  beforeEach(() => {
    vi.useFakeTimers();
    wrapper = mount(AppAlerts, {
      global: {
        provide: {
          store: store,
        },
        plugins: [i18n, router],
      },
    });
  });

  afterEach(() => {
    wrapper.unmount();
  });

  it("Should render tabs", () => {
    expect(wrapper.find('[data-test="alert-tabs"]').exists()).toBeTruthy();
  });
  it("Should render alerts tab", () => {
    expect(wrapper.find('[data-test="alert-alerts-tab"]').text()).toBe(
      "Alerts"
    );
  });
  it("Should render destinations tabs", () => {
    expect(wrapper.find('[data-test="alert-destinations-tab"]').text()).toBe(
      "Destinations"
    );
  });
  it("Should render templates tabs", () => {
    expect(wrapper.find('[data-test="alert-templates-tab"]').text()).toBe(
      "Templates"
    );
  });
});
