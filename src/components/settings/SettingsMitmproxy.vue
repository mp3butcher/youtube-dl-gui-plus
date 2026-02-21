<template>
  <base-fieldset
      :legend="t('settings.mitmproxy.legend')"
      :label="t('settings.mitmproxy.legendLabel')"
  >
    <label class="font-semibold mt-2" for="mitmPort">
      {{ t('settings.mitmproxy.mitmPort.label') }}
    </label>
    <input
        type="number"
        id="mitmPort"
        class="input mb-2"
        v-model="settings.mitmproxy.mitmPort"
        placeholder="15930"
    />
     <label class="font-semibold mt-2" for="mitmExtraArgs">
      {{ t('settings.mitmproxy.mitmExtraArgs.label') }}
    </label>
    <input
        type="text"
        id="mitmExtraArgs"
        class="input mb-2"
        v-model="settings.mitmproxy.mitmExtraArgs"
        placeholder="--anticache --anticomp --mode socks5"
    />
    <label class="font-semibold mt-2" for="headerFilter">
      {{ t('settings.mitmproxy.headerFilter.label') }}
    </label>
    <textarea
      id="headerFilter"
      class="input mb-2"
      v-model="headerFilterText"
      placeholder="one header per line">
    </textarea>
  </base-fieldset>
</template>

<script setup lang="ts">

import BaseFieldset from '../base/BaseFieldset.vue';
import { Settings } from '../../tauri/types/config.ts';
import { useI18n } from 'vue-i18n';
import { computed } from 'vue';

const { t } = useI18n();
const settings = defineModel<Settings>({ required: true });

const headerFilterText = computed({
  get: (): string => {
    const hf = settings.value.mitmproxy.headerFilter;
    return Array.isArray(hf) ? hf.join('\n') : '';
  },
  set: (val: string) => {
    const arr = val
      .split(/\r?\n/)
      .map(s => s.trim())
      .filter(s => s.length > 0);
    settings.value.mitmproxy.headerFilter = arr;
  },
});

</script>
