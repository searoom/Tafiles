<template>
  <div
    id="title-bar"
    :class="{ maximized: isMaximized }"
    @mousedown="onTitleMouseDown"
    @mousemove="onTitleMouseMove"
    @mouseup="onTitleMouseUp"
    @mouseleave="onTitleMouseUp"
    @dblclick="onTitleDblClick"
  >
    <div id="title-tab-bar">
      <div
        v-for="tab in tabsStore.tabs"
        :key="tab.id"
        class="ct-tab"
        :class="{ active: tab.id === tabsStore.activeId }"
        @click="onTabClick(tab.id)"
        @mousedown="onTabMouseDown($event, tab.id)"
        @contextmenu.prevent="showTabCtxMenu($event, tab.id)"
        draggable="false"
      >
        <span class="ct-tab-icon">
          <img v-if="tabFolderIcon" :src="tabFolderIcon" width="16" height="16" style="display:block" />
          <svg v-else width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>
          </svg>
        </span>
        <span class="ct-tab-title">{{ tab.title }}</span>
        <span class="ct-tab-close" @click.stop="closeTab(tab.id)" title="关闭标签页">✕</span>
      </div>
      <div v-if="tabsStore.tabs.length === 0" class="ct-tab-placeholder">
        <span class="app-title">Tafiles</span>
      </div>
    </div>
    <div class="tb-right">
      <button class="tb-btn" @click="windowMinimize" title="最小化">
        <svg width="12" height="12" viewBox="0 0 12 12"><rect y="5" width="12" height="1.5" fill="currentColor"/></svg>
      </button>
      <button class="tb-btn" @click="windowToggleMaximize" title="最大化/还原">
        <svg v-if="isMaximized" width="12" height="12" viewBox="0 0 12 12">
          <rect x="1.5" y="3" width="7.5" height="7.5" fill="none" stroke="currentColor" stroke-width="1"/>
          <rect x="3" y="1.5" width="7.5" height="7.5" fill="#e8e8ed" stroke="currentColor" stroke-width="1"/>
        </svg>
        <svg v-else width="12" height="12" viewBox="0 0 12 12">
          <rect x="1" y="1" width="10" height="10" fill="none" stroke="currentColor" stroke-width="1.2"/>
        </svg>
      </button>
      <button class="tb-btn tb-close" @click="closeWindow" title="关闭">
        <svg width="12" height="12" viewBox="0 0 12 12">
          <line x1="1" y1="1" x2="11" y2="11" stroke="currentColor" stroke-width="1.2"/>
          <line x1="11" y1="1" x2="1" y2="11" stroke="currentColor" stroke-width="1.2"/>
        </svg>
      </button>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted, onBeforeUnmount } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useTabsStore } from '@/stores/tabs'
import { ElMessageBox } from 'element-plus'

const tabsStore = useTabsStore()

const emit = defineEmits(['tab-activated'])

const isMaximized = ref(false)
const tabFolderIcon = ref(null)

let titleBarDragPos = { x: 0, y: 0 }
let titleBarDragging = false

onMounted(async () => {
  try {
    isMaximized.value = await invoke('window_is_maximized')
  } catch {}
  try {
    const data = await invoke('get_file_icon', { ext: 'folder', size: 16 })
    const blob = new Blob([new Uint8Array(data)], { type: 'image/png' })
    tabFolderIcon.value = URL.createObjectURL(blob)
  } catch {}
})

onBeforeUnmount(() => {
  if (tabFolderIcon.value) URL.revokeObjectURL(tabFolderIcon.value)
})

function onTabClick(tabId) {
  tabsStore.activateTab(tabId)
  emit('tab-activated')
}

function onTabMouseDown(e, tabId) {
  if (e.button === 1) {
    e.preventDefault()
    closeTab(tabId)
  }
}

async function closeTab(id) {
  if (tabsStore.tabs.length <= 1) {
    try {
      await ElMessageBox.confirm('关闭最后一个标签页将退出应用，确定吗？', '提示', {
        confirmButtonText: '确定',
        cancelButtonText: '取消',
        type: 'warning',
      })
    } catch { return }
  }
  tabsStore.closeTab(id)
  emit('tab-activated')
}

async function showTabCtxMenu(e, tabId) {
  try {
    const { confirm } = await ElMessageBox.confirm('', '标签页操作', {
      confirmButtonText: '关闭其他标签页',
      cancelButtonText: '关闭右侧标签页',
      showCancelButton: true,
      distinguishCancelAndClose: true,
    })
    if (confirm) {
      closeOtherTabs(tabId)
    }
  } catch (action) {
    if (action === 'cancel') {
      closeRightTabs(tabId)
    }
  }
}

function closeOtherTabs(tabId) {
  const idx = tabsStore.tabs.findIndex(t => t.id === tabId)
  if (idx === -1) return
  tabsStore.tabs.splice(idx + 1)
  tabsStore.tabs.splice(0, idx)
  tabsStore.activeId = tabId
  emit('tab-activated')
}

function closeRightTabs(tabId) {
  const idx = tabsStore.tabs.findIndex(t => t.id === tabId)
  if (idx === -1) return
  tabsStore.tabs.splice(idx + 1)
  tabsStore.activeId = tabId
  emit('tab-activated')
}

function onTitleMouseDown(e) {
  if (e.target.closest('.tb-btn') || e.target.closest('.ct-tab-close')) return
  titleBarDragPos = { x: e.screenX, y: e.screenY }
  titleBarDragging = false
}

function onTitleMouseMove(e) {
  if (titleBarDragPos.x === 0) return
  if (titleBarDragging) return
  const dx = Math.abs(e.screenX - titleBarDragPos.x)
  const dy = Math.abs(e.screenY - titleBarDragPos.y)
  if (dx > 3 || dy > 3) {
    titleBarDragging = true
    invoke('window_start_drag').catch(() => {})
  }
}

function onTitleMouseUp() {
  titleBarDragPos = { x: 0, y: 0 }
  titleBarDragging = false
}

function onTitleDblClick(e) {
  if (e.target.closest('.tb-btn') || e.target.closest('.ct-tab-close')) return
  windowToggleMaximize()
}

async function windowMinimize() {
  await invoke('window_minimize').catch(() => {})
}

async function windowToggleMaximize() {
  try {
    await invoke('window_toggle_maximize')
    isMaximized.value = await invoke('window_is_maximized')
  } catch {}
}

async function closeWindow() {
  await invoke('window_close').catch(() => {})
}
</script>

<style scoped>
#title-bar {
  display: flex;
  align-items: stretch;
  height: 44px;
  flex-shrink: 0;
  background: rgba(232, 232, 237, 0.55);
  backdrop-filter: blur(12px);
  -webkit-backdrop-filter: blur(12px);
  z-index: 2000;
  position: relative;
  user-select: none;
  -webkit-user-select: none;
}
#title-bar.maximized {
  height: 40px;
}
#title-tab-bar {
  display: flex;
  align-items: stretch;
  flex: 1;
  overflow: hidden;
  min-width: 0;
  padding-top: 10px;
}
#title-bar.maximized #title-tab-bar {
  padding-top: 0;
}
.ct-tab {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 0 6px 0 10px;
  min-width: 72px;
  max-width: 200px;
  cursor: default;
  font-size: 14px;
  color: #555;
  border-right: 1px solid #d0d0d5;
  position: relative;
  background: #dcdce0;
  transition: background 0.1s;
  flex-shrink: 0;
}
.ct-tab:hover {
  background: #d4d4da;
}
.ct-tab.active {
  background: #fff;
  color: #303133;
}
.ct-tab.active .ct-tab-close {
  color: #999;
}
.ct-tab-icon {
  display: flex;
  align-items: center;
  flex-shrink: 0;
  color: #d4a017;
}
.ct-tab-title {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  flex: 1;
  min-width: 0;
}
.ct-tab-close {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 16px;
  height: 16px;
  border-radius: 50%;
  font-size: 10px;
  color: transparent;
  flex-shrink: 0;
  transition: background 0.15s, color 0.15s;
}
.ct-tab:hover .ct-tab-close {
  color: rgba(0,0,0,0.5);
}
.ct-tab-close:hover {
  background: #e81123;
  color: #fff !important;
}
.ct-tab-placeholder {
  display: flex;
  align-items: center;
  padding: 0 14px;
  flex: 1;
}
.app-title {
  font-size: 13px;
  font-weight: 600;
  color: #666;
}
.tb-right {
  display: flex;
  align-items: stretch;
  flex-shrink: 0;
}
.tb-btn {
  width: 44px;
  border: none;
  background: transparent;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  color: #555;
  transition: background 0.1s;
}
.tb-btn:hover {
  background: #d0d0d5;
}
.tb-close:hover {
  background: #e81123;
  color: #fff;
}
</style>
