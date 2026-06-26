import { defineStore } from 'pinia'
import { ref, computed } from 'vue'

export const useTabsStore = defineStore('tabs', () => {
  const tabs = ref([])
  const activeId = ref('')

  function getTitle(path) {
    if (!path) return '新标签页'
    if (path.startsWith('zip://')) {
      return path.replace(/\\/g, '/').split('/').pop() || 'ZIP'
    }
    if (path.startsWith('webdav://')) return 'WebDAV'
    const segments = path.replace(/\\+$/, '').split(/[\\/]/).filter(Boolean)
    const last = segments[segments.length - 1]
    if (last && last.match(/^[A-Z]:$/i)) return '本地磁盘 (' + last + ')'
    return last || '此电脑'
  }

  function addTab(path) {
    const existing = tabs.value.find(t => t.path === path)
    if (existing) {
      activeId.value = existing.id
      return existing.id
    }
    const id = Date.now().toString(36) + Math.random().toString(36).slice(2, 6)
    tabs.value.push({ id, path, title: getTitle(path) })
    if (tabs.value.length > 10) {
      tabs.value.splice(0, tabs.value.length - 10)
    }
    activeId.value = id
    return id
  }

  function closeTab(id) {
    const idx = tabs.value.findIndex(t => t.id === id)
    if (idx === -1) return
    tabs.value.splice(idx, 1)
    if (tabs.value.length === 0) {
      activeId.value = ''
    } else if (activeId.value === id) {
      const nextIdx = Math.min(idx, tabs.value.length - 1)
      activeId.value = tabs.value[nextIdx].id
    }
  }

  function updateCurrentTab(path) {
    const tab = tabs.value.find(t => t.id === activeId.value)
    if (tab) {
      tab.path = path
      tab.title = getTitle(path)
    }
  }

  function activateTab(id) {
    activeId.value = id
  }

  const currentPath = computed(() => {
    const tab = tabs.value.find(t => t.id === activeId.value)
    return tab ? tab.path : ''
  })

  return { tabs, activeId, currentPath, addTab, closeTab, updateCurrentTab, activateTab, getTitle }
})
