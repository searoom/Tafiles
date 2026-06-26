import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'

const STORAGE_KEY = 'tafiles-webdav-connections'

function loadConnections() {
  try {
    return JSON.parse(localStorage.getItem(STORAGE_KEY) || '[]')
  } catch {
    return []
  }
}

function saveConnections(connections) {
  const toSave = connections.map(c => ({
    id: c.id,
    name: c.name,
    url: c.url,
    username: c.username,
    password: c.password ? btoa(c.password) : '',
    sessionId: c.sessionId || '',
  }))
  localStorage.setItem(STORAGE_KEY, JSON.stringify(toSave))
}

export const useWebdavStore = defineStore('webdav', () => {
  const raw = loadConnections()
  const connections = ref(raw.map(c => ({
    ...c,
    password: c.password ? atob(c.password) : '',
  })))
  const activeSessionId = ref('')
  const syncStatuses = ref({})

  const activeConnection = computed(() =>
    connections.value.find(c => c.sessionId === activeSessionId.value) || null
  )

  function save() {
    saveConnections(connections.value)
  }

  async function connect(url, name, username, password) {
    const info = await invoke('webdav_connect', { url, name, username, password })
    return info
  }

  async function addConnection(name, url, username, password) {
    const info = await connect(url, name, username, password)
    const existing = connections.value.find(c => c.id === info.id)
    if (existing) {
      existing.name = info.name
      existing.url = info.url
      existing.username = info.username
      existing.password = password
      existing.sessionId = info.id
    } else {
      connections.value.push({
        id: info.id,
        name: info.name,
        url: info.url,
        username: info.username,
        password,
        sessionId: info.id,
      })
    }
    save()
    return info
  }

  async function updateConnection(id, name, url, username, password) {
    const old = connections.value.find(c => c.id === id)
    if (old && old.sessionId) {
      try {
        await invoke('webdav_disconnect', { sessionId: old.sessionId })
      } catch {}
    }

    const info = await connect(url, name, username, password)
    if (old) {
      old.name = info.name
      old.url = info.url
      old.username = info.username
      old.password = password
      old.sessionId = info.id
    }
    save()
    return info
  }

  async function removeConnection(id) {
    const conn = connections.value.find(c => c.id === id)
    if (conn && conn.sessionId) {
      try {
        await invoke('webdav_disconnect', { sessionId: conn.sessionId })
      } catch {}
    }
    connections.value = connections.value.filter(c => c.id !== id)
    if (activeSessionId.value === id) {
      activeSessionId.value = ''
    }
    save()
  }

  async function ensureConnected(sessionId) {
    const conn = connections.value.find(c => c.sessionId === sessionId || c.id === sessionId)
    if (!conn) return false

    try {
      const info = await connect(conn.url, conn.name, conn.username, conn.password || '')
      conn.sessionId = info.id
      save()
      return true
    } catch {
      return false
    }
  }

  function setActive(sessionId) {
    activeSessionId.value = sessionId
  }

  function isWebdavPath(path) {
    return typeof path === 'string' && path.startsWith('webdav://')
  }

  function parseWebdavPath(path) {
    if (!isWebdavPath(path)) return null
    const rest = path.slice('webdav://'.length)
    const firstSlash = rest.indexOf('/')
    if (firstSlash === -1) return { href: rest, path: '' }
    return {
      href: rest.slice(firstSlash + 1),
      path: '/' + decodeURIComponent(rest.slice(firstSlash + 1)),
    }
  }

  function updateSyncStatus({ path, sessionId, remotePath, status, error }) {
    syncStatuses.value[path] = { sessionId, remotePath, status, error }
  }

  return {
    connections,
    activeSessionId,
    activeConnection,
    syncStatuses,
    addConnection,
    updateConnection,
    removeConnection,
    ensureConnected,
    setActive,
    isWebdavPath,
    parseWebdavPath,
    updateSyncStatus,
  }
})
