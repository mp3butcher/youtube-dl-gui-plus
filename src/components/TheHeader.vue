<template>
  <header class="p-4 bg-base-300 flex gap-4 justify-center w-full shadow-lg">
    <form @submit.prevent="submitUrl" class="join w-full max-w-155 grow">
      <input
          v-model="url"
          id="url-input"
          name="URL input for video or playlist"
          class="input join-item w-full"
          :placeholder="inputPlaceholder"
          type="text"
          ref="input"
      />
      <button
          class="btn btn-primary join-item"
          type="submit"
          :disabled="isInputDisabled"
      >
        {{ t('common.add') }}
      </button>
    </form>
    <router-link class="btn btn-subtle" :title="t('layout.header.nav.settings')" :to="{ name: 'settings' }">
      <span class="sr-only">{{ t('layout.header.nav.settings') }}</span>
      <cog8-tooth-icon class="w-6 h-6"/>
    </router-link>
  <div> <button type="button" @click="startStopScanner" class="btn btn-subtle{{ text }}"  :title="t('layout.header.nav.networkscan')"> {{ text }}  
     <span class="sr-only">{{ t('layout.header.nav.networkscan') }}</span>
    <signal-icon class="w-6 h-6"/></button> </div>
  </header>
</template>

<script setup lang="ts">

import { Cog8ToothIcon } from '@heroicons/vue/24/outline';
import { SignalIcon } from '@heroicons/vue/24/outline';
import { useMediaStore } from '../stores/media/media';
import { ref, computed, onMounted, onUnmounted } from 'vue';
import { useClipboard } from '../composables/useClipboard';
import { useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';
import { useSettingsStore } from '../stores/settings';
import { isValidUrl } from '../helpers/url.ts';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { useToastStore } from '../stores/toast';
const toastStore = useToastStore();

const state = ref(false), text = computed(() => state.value ? 'ON' : 'OFF');
let unlistenScanner: (() => void) | null = null;

async function startStopScanner() {
  if (!state.value)
  {
    console.log('Starting network scan...');
    toastStore.showToast('Starting network scan...', { style: 'success' });
    try {
      const result = await invoke<string>('start_tcp_scanner');
      toastStore.showToast('Started '+ result, { style: 'success' });
      state.value = true
    } catch (error) {
      console.error('Failed to start scanner:', error);
      toastStore.showToast('Failed start network scan...', { style: 'error' });
      state.value = false;
    }
  } else {
    console.log('Stopping network scan...');
    toastStore.showToast('Stopping network scan...', { style: 'success' });
    try {
      const result = await invoke<string>('stop_tcp_scanner');
      console.log(result);
      toastStore.showToast(result, { style: 'success' });
      state.value = false;
    } catch (error) {
      console.error('Failed to stop scanner:', error);
      toastStore.showToast('Failed to stop network scan...', { style: 'error' });
      state.value = false;
    }
   }
}

const { t } = useI18n();
const router = useRouter();
const mediaStore = useMediaStore();

const settingsStore = useSettingsStore();

const doPolling = computed(() => settingsStore.settings.input.autoFillClipboard);

const { content: clipboardContent } = useClipboard({
  doPolling,
});

const input = ref<HTMLInputElement | null>(null);

const inputPlaceholder = computed(() => {
  const defaultPlaceholder = t('layout.header.placeholder');
  if (clipboardHasValidUrl.value) {
    return clipboardContent.value ?? defaultPlaceholder;
  } else {
    return defaultPlaceholder;
  }
});

const clipboardHasValidUrl = computed(() => isValidUrl(clipboardContent));

const isInputDisabled = computed(() => {
  return url.value.length === 0 && !clipboardHasValidUrl.value;
});

const url = ref('');

const submitUrl = () => {
  const urlToSubmit = url.value.length > 0 ? url.value : clipboardContent.value;
  if (!urlToSubmit) return;
  void mediaStore.dispatchMediaInfoFetch(urlToSubmit);
  void router.push('/');
};

const regexDash = /(?:<\?.*>\n*)*<MPD/gi;
const regexprange = /bytes\s*=?(\d+)-(\d+)?\/?(\d+)?/g;
const lastwdurl=ref('');
const lastwdheader=ref('');
/*
let reqlicHeaders='';
const regspotpssh=/"pssh"\s*:\s*"([^"]*)/g*/
const regexwd=/[\s.]*<ContentProtection[^>]*VINE">[\s.]*<cenc:pssh[^>]*>(.*)<\/cenc:pssh[^>]*>[\s.]*<\/ContentProtection[^>]*>/g
const regexcenc=/[\s.]*<ContentProtection[^>]*>[\s.]*<cenc:pssh[^>]*>(.*)<\/cenc:pssh[^>]*>[\s.]*<\/ContentProtection[^>]*>/g
const currentkeys=ref('');//, lastposturl=''//eslint-disable-line no-unused-vars
const pssh=ref('');

onMounted(async () => {
  input.value?.focus();
  // Setup listener for TCP scanner messages
  unlistenScanner = await listen('tcp_scanner', (event: any) => {
    console.error('Received TCP message:', event.payload);
    let msg = event.payload.message;
    let headerFilter = settingsStore.settings.mitmproxy.headerFilter;
    //if (scannerIsOn)
    {
      let data;
      try {
          data = JSON.parse(msg);
      } catch (e) {
          console.error(e); //Error in the above string (in this case, yes)!
          toastStore.showToast(msg, { style: 'error' });
          return;
      }

      //Filter unwanted headers
      let idx = 0, removed = false;
      while(idx<data.headers.length){
          for(let idx2=0; idx2 <  headerFilter.length; idx2++) {
              if(data.headers[idx].k.toLowerCase() == headerFilter[idx2]) {
                  data.headers.splice(idx,1);
                  removed = true;
                  break;
              }
          }
          if(!removed) idx++;
          removed = false;
      }

      //Basic scanner
      let sizeok = false;   //Check if range is not prohibitively small
      let contentype = "";
      let contentlength = 0;
      let headerstr = '';
      let lh = data.headers.length;
      for(let i=0; i<lh; i++) {
          let h = data.headers[i];
          headerstr = headerstr + h.k + ": " + h.v + '$';
          if (h.k.toLowerCase() == "range") {
              const ranges = [...h.v.matchAll(regexprange)];
              console.log(ranges[0]);
              if (typeof (ranges[0][2]) == "undefined") contentlength = 20000000;
              else contentlength = parseInt(ranges[0][2], 10) - parseInt(ranges[0][1], 10);
              if (contentlength > 1500000) sizeok = true;
          }
      }
      console.log(" scan url " + data.url + " headers:" + headerstr);
      if (data.method == 'POST') {
          // scanPostRequest(data)
      }

      lh = data.rheaders.length;
      for(let i=0; i<lh; i++) {
          let h = data.rheaders[i];
          if (h.k.toLowerCase() == "content-type") contentype = h.v;
          if (h.k.toLowerCase() == "content-length") {
              contentlength = parseInt(h.v, 10);
          }
          if (h.k.toLowerCase() == "content-range") {

              const ranges = [...h.v.matchAll(regexprange)];
              if (typeof (ranges[0][2]) == "undefined") contentlength = 20000000;
              else contentlength = parseInt(ranges[0][2], 10) - parseInt(ranges[0][1], 10);
              if (contentlength > 1500000) sizeok = true;
          }
      };
      let toscandeeply = false;
      //Large Content-type video
      if (contentype.startsWith('video') && contentlength > 1500000) {
          toscandeeply = true;
          console.warn(" [x] contentype video!!!!!!!!!!!" + data);
      }
      //Large Content-Range
      if(!contentype.startsWith('image') && (sizeok || contentlength >  1500000)) {
          toscandeeply = true;
      }
      let res = atob(data.response)
      console.log(" [x] response  " + res.substring(0, 100));
      if (res.length > 1) {
          //Spotify pssh            https://seektables.scdn.co/seektable/file_id.json "pssh":"cap"
          if(data.url.startsWith('https://seektables.scdn.co/seektable/')) {
              const wd = [""];//...res.matchAll(regspotpssh)];
              console.log(wd[0]);
              if (typeof (wd[0]) != "undefined") pssh.value = wd[0][1];

          }
          //Spotify stream            https://audio-ak.spotifycdn.com/audio
          if(data.url.indexOf('.spotifycdn.com/audio')>0 || data.url.indexOf('scdn.co/audio')>0) {
              idx = 0;
              removed = false;
              while(idx<data.headers.length) {
                      if(data.headers[idx].k.toLowerCase() == 'range') {
                          data.headers.splice(idx,1);
                          removed = true;
                      }
                  if(!removed) idx++;
                  removed = false;
              }
              if(currentkeys.value!=''){
                  //Delay to be sure to have key returned lastwdurl = data.url; lastwdheader = data.headers
                  toscandeeply = true;
                  //let ckey = currentkeys;
                  //setVideoKey(data.url, data.headers, ckey, 2000);
                  currentkeys.value = ''
              }
          }

          if (res[0] == "#") { //HLS?
              console.log(res);
              toscandeeply = true;
          } else if (res.match(regexDash)) { //DASH?
              console.log(res);
              pssh.value = '';
              ///seek pssh
              const wd = [...res.matchAll(regexwd)];
              console.log(wd[0]);
              if (typeof (wd[0]) != "undefined") {
                  let bestqkey = 0; //Have to find a way to get desired stream
                  console.log(wd[bestqkey]);
                  pssh.value = wd[bestqkey][1];
                  lastwdurl.value = data.url;
                  lastwdheader.value = data.headers
              }else{
                  const cenc = [...res.matchAll(regexcenc)];
                  console.log(cenc[0]);
                  if (typeof (cenc[0]) != "undefined") {
                      console.log(cenc[0]);
                      pssh.value = cenc[0][1];
                      lastwdurl.value = data.url;
                      lastwdheader.value = data.headers
                  } else toscandeeply = true;
              }
          }
      }
      if (toscandeeply) {
        let argheaders: Record<string, string> = {};
        lh = data.headers.length;
        for(let i=0; i<lh; i++) {
            let h = data.headers[i];
            argheaders[h.k] = h.v;
        }
        toastStore.showToast(data.url+" "+JSON.stringify(argheaders), { style: 'success' });
        mediaStore.dispatchMediaInfoFetch(data.url, false, argheaders);
      }
    }
  });
});

onUnmounted(() => {
  if (unlistenScanner) {
    unlistenScanner();
  }
});

</script>
