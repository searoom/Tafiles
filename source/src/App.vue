<template>
  <div id="app-root">
    <TitleBar @tab-activated="onTabActivated" />
    <div id="app-content">
      <router-view />
    </div>
    <span v-if="versionText" class="version-badge">{{ versionText }}</span>
  </div>
</template>

<script setup>
import { ref, onMounted, provide } from 'vue'
import TitleBar from '@/components/TitleBar.vue'
import { useTabsStore } from '@/stores/tabs'
import { useRouter } from 'vue-router'

const tabsStore = useTabsStore()
const router = useRouter()
const versionText = ref('')

// Provide a function the FileBrowser can call to notify tab switching
provide('onTabActivated', () => {})

onMounted(async () => {
  // Load build info for version badge
  try {
    const res = await fetch('/build_info.json')
    const info = await res.json()
    if (info.buildNumber > 0) {
      versionText.value = `v${info.version}.${info.buildNumber}`
    }
  } catch {}
})

function onTabActivated() {
  // FileBrowser watches the store and reacts
}
</script>

<style>
html, body, #app {
  margin: 0;
  padding: 0;
  height: 100%;
  overflow: hidden;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
}
#app-root {
  display: flex;
  flex-direction: column;
  height: 100vh;
}
#app-content {
  flex: 1;
  overflow: hidden;
}
router-view {
  display: block;
  height: 100%;
}
.version-badge {
  position: fixed;
  right: 10px;
  bottom: 6px;
  font-size: 11px;
  color: #c0c4cc;
  pointer-events: none;
  z-index: 999;
}
</style>
