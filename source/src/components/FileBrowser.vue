<template>
  <div class="file-browser">
    <aside class="sidebar" :style="{ width: sidebarWidth + 'px' }">
      <div class="sidebar-section">快速访问</div>
      <div
        v-for="folder in quickAccessFolders" :key="folder.path"
        class="sidebar-item" :class="{ active: currentPath === folder.path }"
        @click="navigateTo(folder.path)"
      >
        <el-icon :size="16"><component :is="folder.icon" /></el-icon>
        <span class="sidebar-item-label">{{ folder.name }}</span>
      </div>
      <div class="sidebar-section">此电脑</div>
      <div class="sidebar-tree">
        <el-tree
          ref="treeRef"
          :data="treeData"
          :props="{ label: 'name', isLeaf: () => false }"
          lazy
          :load="loadTreeNode"
          node-key="path"
          highlight-current
          @node-click="onTreeNodeClick"
        >
          <template #default="{ node, data }">
            <span class="tree-node">
              <el-icon :size="16">
                <FolderOpened v-if="node.expanded" />
                <Folder v-else />
              </el-icon>
              <span class="tree-node-label">{{ data.name }}</span>
            </span>
          </template>
        </el-tree>
      </div>
    </aside>

    <div class="resize-bar" @mousedown="onResizeStart"></div>

    <div class="main-area">
      <div class="toolbar">
        <div class="breadcrumb">
          <span class="bc-root" @click="navigateTo('')">此电脑</span>
          <template v-for="(seg, i) in pathSegments" :key="i">
            <span class="bc-sep">›</span>
            <span
              :class="['bc-item', { active: i === pathSegments.length - 1 }]"
              @click="navigateTo(seg.path)"
            >{{ seg.name }}</span>
          </template>
        </div>
        <div class="toolbar-actions">
          <el-button-group size="small">
            <el-button
              :type="viewMode === 'large' ? 'primary' : ''"
              @click="setViewMode('large')"
              title="大图标"
            >
              <svg width="14" height="14" viewBox="0 0 14 14"><rect x="1" y="1" width="5" height="5" rx="1" :fill="viewMode === 'large' ? '#fff' : '#606266'"/><rect x="8" y="1" width="5" height="5" rx="1" :fill="viewMode === 'large' ? '#fff' : '#606266'"/><rect x="1" y="8" width="5" height="5" rx="1" :fill="viewMode === 'large' ? '#fff' : '#606266'"/><rect x="8" y="8" width="5" height="5" rx="1" :fill="viewMode === 'large' ? '#fff' : '#606266'"/></svg>
            </el-button>
            <el-button
              :type="viewMode === 'medium' ? 'primary' : ''"
              @click="setViewMode('medium')"
              title="中图标"
            >
              <svg width="14" height="14" viewBox="0 0 14 14"><rect x="0.5" y="0.5" width="3.5" height="3.5" rx="0.5" :fill="viewMode === 'medium' ? '#fff' : '#606266'"/><rect x="5" y="0.5" width="3.5" height="3.5" rx="0.5" :fill="viewMode === 'medium' ? '#fff' : '#606266'"/><rect x="9.5" y="0.5" width="4" height="3.5" rx="0.5" :fill="viewMode === 'medium' ? '#fff' : '#606266'"/><rect x="0.5" y="5" width="3.5" height="3.5" rx="0.5" :fill="viewMode === 'medium' ? '#fff' : '#606266'"/><rect x="5" y="5" width="3.5" height="3.5" rx="0.5" :fill="viewMode === 'medium' ? '#fff' : '#606266'"/><rect x="9.5" y="5" width="4" height="3.5" rx="0.5" :fill="viewMode === 'medium' ? '#fff' : '#606266'"/></svg>
            </el-button>
            <el-button
              :type="viewMode === 'small' ? 'primary' : ''"
              @click="setViewMode('small')"
              title="小图标"
            >
              <svg width="14" height="14" viewBox="0 0 14 14"><rect x="0.5" y="0.5" width="2.5" height="2.5" rx="0.5" :fill="viewMode === 'small' ? '#fff' : '#606266'"/><rect x="4" y="0.5" width="2.5" height="2.5" rx="0.5" :fill="viewMode === 'small' ? '#fff' : '#606266'"/><rect x="7.5" y="0.5" width="2.5" height="2.5" rx="0.5" :fill="viewMode === 'small' ? '#fff' : '#606266'"/><rect x="11" y="0.5" width="2.5" height="2.5" rx="0.5" :fill="viewMode === 'small' ? '#fff' : '#606266'"/><rect x="0.5" y="4" width="2.5" height="2.5" rx="0.5" :fill="viewMode === 'small' ? '#fff' : '#606266'"/><rect x="4" y="4" width="2.5" height="2.5" rx="0.5" :fill="viewMode === 'small' ? '#fff' : '#606266'"/><rect x="7.5" y="4" width="2.5" height="2.5" rx="0.5" :fill="viewMode === 'small' ? '#fff' : '#606266'"/><rect x="11" y="4" width="2.5" height="2.5" rx="0.5" :fill="viewMode === 'small' ? '#fff' : '#606266'"/></svg>
            </el-button>
            <el-button
              :type="viewMode === 'list' ? 'primary' : ''"
              @click="setViewMode('list')"
              title="列表"
            >
              <svg width="14" height="14" viewBox="0 0 14 14"><rect x="1" y="1.5" width="12" height="1.5" rx="0.5" :fill="viewMode === 'list' ? '#fff' : '#606266'"/><rect x="1" y="5.5" width="12" height="1.5" rx="0.5" :fill="viewMode === 'list' ? '#fff' : '#606266'"/><rect x="1" y="9.5" width="12" height="1.5" rx="0.5" :fill="viewMode === 'list' ? '#fff' : '#606266'"/></svg>
            </el-button>
          </el-button-group>
          <el-button size="small" text @click="refreshList" title="刷新">
            <el-icon><Refresh /></el-icon>
          </el-button>
          <el-button size="small" text @click="pickFolder" title="打开文件夹">
            <el-icon><FolderOpened /></el-icon>
          </el-button>
        </div>
      </div>

      <div class="file-area" v-loading="loading">
        <template v-if="!currentPath">
          <div class="drives-grid">
            <div
              v-for="drive in drives" :key="drive.path"
              class="drive-card"
              @dblclick="navigateTo(drive.path)"
            >
              <el-icon :size="40" color="#909399"><Monitor /></el-icon>
              <span class="drive-name">{{ drive.name }}</span>
            </div>
          </div>
        </template>

        <template v-else-if="!loading && files.length === 0">
          <div class="empty-dir">该文件夹为空</div>
        </template>

        <template v-else>
          <div v-if="viewMode === 'list'" class="file-list">
            <div class="file-list-header">
              <span class="col-name">名称</span>
              <span class="col-size">大小</span>
              <span class="col-type">类型</span>
            </div>
            <div
              v-for="file in files" :key="file.path"
              class="file-row"
              @dblclick="handleDoubleClick(file)"
            >
              <span class="col-name">
                <img v-if="file.is_dir && fileIconMap['folder']" class="type-icon-list" :src="fileIconMap['folder']" />
                <img v-else-if="pathIconMap[file.path]" class="type-icon-list" :src="pathIconMap[file.path]" />
                <img v-else-if="fileIconMap[file.ext]" class="type-icon-list" :src="fileIconMap[file.ext]" />
                <el-icon v-else :size="16" color="#909399"><Document /></el-icon>
                {{ file.name }}
              </span>
              <span class="col-size">{{ file.is_dir ? '' : formatSize(file.size) }}</span>
              <span class="col-type">{{ file.is_dir ? '文件夹' : file.ext.toUpperCase() || '文件' }}</span>
            </div>
          </div>

          <div v-else :class="['file-grid', `view-${viewMode}`]">
            <div
              v-for="file in files" :key="file.path"
              class="file-item"
              @dblclick="handleDoubleClick(file)"
            >
              <div class="file-thumb">
                <template v-if="file.is_dir">
                  <img class="folder-icon" :src="fileIconMap['folder'] || ''" v-if="fileIconMap['folder']" />
                  <el-icon v-else :size="thumbIconSize" color="#409eff"><Folder /></el-icon>
                </template>
                <template v-else-if="isImage(file.ext) && thumbnailMap[file.path]">
                  <img :src="thumbnailMap[file.path]" />
                </template>
                <template v-else-if="pathIconMap[file.path]">
                  <img :src="pathIconMap[file.path]" :class="viewMode === 'large' ? 'type-icon-large' : (viewMode === 'medium' ? 'type-icon-medium' : 'type-icon-small')" />
                </template>
                <template v-else-if="fileIconMap[file.ext]">
                  <img :src="fileIconMap[file.ext]" :class="viewMode === 'large' ? 'type-icon-large' : (viewMode === 'medium' ? 'type-icon-medium' : 'type-icon-small')" />
                </template>
                <template v-else>
                  <el-icon :size="thumbIconSize" color="#909399"><Document /></el-icon>
                </template>
              </div>
              <div class="file-name" :title="file.name">{{ file.name }}</div>
            </div>
          </div>
        </template>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, reactive, computed, onMounted, onBeforeUnmount } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { WebviewWindow } from '@tauri-apps/api/webviewWindow'
import { Folder, FolderOpened, Document, Picture, Monitor, Download, Refresh } from '@element-plus/icons-vue'
import { ElMessage } from 'element-plus'
import { imageExts } from '@/utils/fileType'

let previewableCache = null
function isPreviewable(ext) {
  if (!previewableCache) previewableCache = new Set([
    ...imageExts, 'pdf', 'xlsx', 'xls', 'docx', 'doc',
    'txt', 'log', 'ini', 'cfg', 'env', 'conf', 'yaml', 'yml', 'toml',
    'js', 'ts', 'jsx', 'tsx', 'py', 'html', 'htm', 'css', 'scss', 'less',
    'json', 'xml', 'xhtml', 'md', 'sql', 'java', 'cpp', 'c', 'cs', 'php',
    'rb', 'go', 'rs', 'swift', 'kt', 'sh', 'bash', 'bat', 'ps1', 'zsh',
    'pl', 'lua', 'r', 'dart', 'vue', 'svelte',
  ])
  return previewableCache.has((ext || '').toLowerCase())
}

const drives = ref([])
const homeDir = ref('')
const currentPath = ref('')
const files = ref([])
const loading = ref(false)
const viewMode = ref(localStorage.getItem('tafiles-view-mode') || 'medium')
const treeData = ref([])
const treeRef = ref(null)
const thumbnailMap = reactive({})
const fileIconMap = reactive({})
const pathIconMap = reactive({})
const iconSizes = reactive({})
const iconLoadingSet = new Set()
const sidebarWidth = ref(Number(localStorage.getItem('tafiles-sidebar-width')) || 220)

let resizing = false
let resizeStartX = 0
let resizeStartWidth = 0

const thumbIconSize = computed(() => {
  if (viewMode.value === 'large') return 128
  if (viewMode.value === 'medium') return 64
  return 48
})

const quickAccessFolders = computed(() => {
  if (!homeDir.value) return []
  const h = homeDir.value
  return [
    { name: '桌面', path: h + '\\Desktop', icon: 'Monitor' },
    { name: '下载', path: h + '\\Downloads', icon: 'Download' },
    { name: '文档', path: h + '\\Documents', icon: 'Document' },
    { name: '图片', path: h + '\\Pictures', icon: 'Picture' },
  ]
})

const pathSegments = computed(() => {
  if (!currentPath.value) return []
  const normalized = currentPath.value.replace(/\/+/g, '\\').replace(/\\+$/, '')
  const segments = normalized.split('\\').filter(Boolean)
  const result = []
  let buildPath = ''
  for (const seg of segments) {
    if (buildPath && !buildPath.endsWith('\\')) {
      buildPath += '\\' + seg
    } else {
      buildPath += seg
    }
    if (buildPath.match(/^[A-Z]:$/i)) buildPath += '\\'
    result.push({ name: seg, path: buildPath })
  }
  return result
})

function setViewMode(mode) {
  viewMode.value = mode
  localStorage.setItem('tafiles-view-mode', mode)
  if (currentPath.value && files.value.length) loadFileIconsForCurrentDir()
}

function onResizeStart(e) {
  resizing = true
  resizeStartX = e.clientX
  resizeStartWidth = sidebarWidth.value
  document.addEventListener('mousemove', onResizeMove)
  document.addEventListener('mouseup', onResizeEnd)
  document.body.style.cursor = 'col-resize'
  document.body.style.userSelect = 'none'
}

function onResizeMove(e) {
  if (!resizing) return
  const delta = e.clientX - resizeStartX
  const newWidth = Math.max(160, Math.min(500, resizeStartWidth + delta))
  sidebarWidth.value = newWidth
}

function onResizeEnd() {
  if (!resizing) return
  resizing = false
  document.removeEventListener('mousemove', onResizeMove)
  document.removeEventListener('mouseup', onResizeEnd)
  document.body.style.cursor = ''
  document.body.style.userSelect = ''
  localStorage.setItem('tafiles-sidebar-width', String(sidebarWidth.value))
}

async function navigateTo(path) {
  currentPath.value = path
  if (path) {
    await loadDir(path)
    try { treeRef.value?.setCurrentKey(path) } catch {}
  } else {
    files.value = []
  }
}

async function loadDir(path) {
  loading.value = true
  cleanupThumbnails()
  try {
    files.value = await invoke('read_dir', { path })
    if (viewMode.value !== 'list') loadThumbnails()
    loadFileIconsForCurrentDir()
  } catch (e) {
    ElMessage.error(String(e))
    files.value = []
  } finally {
    loading.value = false
  }
}

async function loadDrives() {
  try {
    drives.value = await invoke('list_drives')
    treeData.value = drives.value.map(d => ({ ...d }))
  } catch (e) {
    console.error('读取驱动器失败:', e)
  }
}

async function loadHomeDir() {
  try {
    homeDir.value = await invoke('get_home_dir')
  } catch (e) {
    console.error('获取主目录失败:', e)
  }
}

async function loadTreeNode(node, resolve) {
  try {
    const dirs = await invoke('list_dirs', { path: node.data.path })
    resolve(dirs)
  } catch {
    resolve([])
  }
}

function onTreeNodeClick(data) {
  navigateTo(data.path)
}

async function pickFolder() {
  try {
    const path = await invoke('pick_folder')
    if (path) {
      await navigateTo(path)
    }
  } catch (e) {
    ElMessage.error(String(e))
  }
}

function refreshList() {
  if (currentPath.value) loadDir(currentPath.value)
}

function handleDoubleClick(file) {
  if (file.is_dir) {
    navigateTo(file.path)
  } else if (isPreviewable(file.ext)) {
    openPreviewWindow(file)
  } else {
    invoke('open_file', { path: file.path }).catch(e => ElMessage.error(String(e)))
  }
}

const previewWindows = new Map()

async function openPreviewWindow(file) {
  // Clean up closed windows first
  for (const [path, label] of previewWindows) {
    if (!WebviewWindow.getByLabel(label)) previewWindows.delete(path)
  }

  // Check if already open
  const existingLabel = previewWindows.get(file.path)
  if (existingLabel) {
    const existing = WebviewWindow.getByLabel(existingLabel)
    if (existing) {
      try {
        await existing.setFocus()
        return
      } catch {}
    }
  }

  const label = 'preview-' + Date.now()
  previewWindows.set(file.path, label)

  const params = new URLSearchParams({
    path: file.path,
    name: file.name,
    ext: file.ext || '',
    size: String(file.size),
  })

  const baseUrl = `${window.location.origin}/preview.html?${params}`
  const isImage = imageExts.includes((file.ext || '').toLowerCase())

  if (isImage) {
    try {
      new WebviewWindow(label, {
        url: baseUrl,
        title: file.name,
        decorations: false,
        maximized: true,
        transparent: true,
      })
    } catch (e) {
      ElMessage.error('打开预览窗口失败: ' + e)
    }
    return
  }

  const savedRect = localStorage.getItem('tafiles-preview-rect')
  let options = {
    url: baseUrl,
    title: `${file.name} - Tafiles`,
  }
  if (savedRect) {
    try {
      const r = JSON.parse(savedRect)
      options.x = r.x
      options.y = r.y
      options.width = r.width
      options.height = r.height
    } catch {}
  } else {
    options.width = 1200
    options.height = 900
    options.center = true
  }

  try {
    new WebviewWindow(label, options)
  } catch (e) {
    ElMessage.error('打开预览窗口失败: ' + e)
  }
}

async function loadThumbnails() {
  const imageFiles = files.value.filter(f => !f.is_dir && imageExts.includes(f.ext.toLowerCase()))
  const batchSize = 6
  const maxSize = viewMode.value === 'large' ? 384 : 192
  for (let i = 0; i < imageFiles.length; i += batchSize) {
    const batch = imageFiles.slice(i, i + batchSize)
    await Promise.allSettled(batch.map(async (file) => {
      try {
        const data = await invoke('get_thumbnail', { path: file.path, maxSize })
        const blob = new Blob([new Uint8Array(data)], { type: 'image/jpeg' })
        thumbnailMap[file.path] = URL.createObjectURL(blob)
      } catch {}
    }))
  }
}

function loadFileIconsForCurrentDir() {
  const exts = new Set()
  for (const f of files.value) {
    if (f.is_dir) { exts.add('folder'); continue }
    const e = f.ext.toLowerCase()
    if (e === 'exe') {
      loadFileIconForPath(f.path, iconSizeFromViewMode())
      continue
    }
    if (e && !fileIconMap[e]) {
      exts.add(e)
    }
  }
  const iconSize = iconSizeFromViewMode()
  for (const ext of exts) {
    loadFileIcon(ext, iconSize)
  }
}

function iconSizeFromViewMode() {
  if (viewMode.value === 'list') return 16
  if (viewMode.value === 'small') return 48
  return 256
}

async function loadFileIcon(ext, size) {
  const key = ext.toLowerCase()
  if (iconLoadingSet.has(key)) return
  if (fileIconMap[key] && (iconSizes[key] || 0) >= size) return
  iconLoadingSet.add(key)
  try {
    const data = await invoke('get_file_icon', { ext: key, size })
    const blob = new Blob([new Uint8Array(data)], { type: 'image/png' })
    const url = URL.createObjectURL(blob)
    if (fileIconMap[key]) URL.revokeObjectURL(fileIconMap[key])
    fileIconMap[key] = url
    iconSizes[key] = size
  } catch {
    const fallbackUrl = `/type-icons/${key}.png`
    const img = new Image()
    img.onload = () => { fileIconMap[key] = fallbackUrl; iconSizes[key] = size }
    img.onerror = () => { fileIconMap[key] = undefined }
    img.src = fallbackUrl
  } finally {
    iconLoadingSet.delete(key)
  }
}

async function loadFileIconForPath(filePath, size) {
  if (iconLoadingSet.has(filePath)) return
  if (pathIconMap[filePath] && (iconSizes['_path_' + filePath] || 0) >= size) return
  iconLoadingSet.add(filePath)
  try {
    const ext = filePath.split('.').pop() || ''
    const data = await invoke('get_file_icon', { ext, size, filePath })
    const blob = new Blob([new Uint8Array(data)], { type: 'image/png' })
    const url = URL.createObjectURL(blob)
    if (pathIconMap[filePath]) URL.revokeObjectURL(pathIconMap[filePath])
    pathIconMap[filePath] = url
    iconSizes['_path_' + filePath] = size
  } catch {} finally {
    iconLoadingSet.delete(filePath)
  }
}

function cleanupThumbnails() {
  for (const key of Object.keys(thumbnailMap)) {
    URL.revokeObjectURL(thumbnailMap[key])
    delete thumbnailMap[key]
  }
}

function isImage(ext) {
  return imageExts.includes((ext || '').toLowerCase())
}

function formatSize(bytes) {
  if (bytes < 1024) return bytes + ' B'
  if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + ' KB'
  return (bytes / (1024 * 1024)).toFixed(1) + ' MB'
}

function handleKeydown(e) {
  if (e.key === 'Backspace' && currentPath.value && !e.target.closest('input, textarea')) {
    e.preventDefault()
    const parent = getParentPath(currentPath.value)
    if (parent !== null) navigateTo(parent)
  }
}

function getParentPath(path) {
  const normalized = path.replace(/\/+/g, '\\').replace(/\\+$/, '')
  const lastSep = normalized.lastIndexOf('\\')
  if (lastSep <= 0) {
    if (normalized.match(/^[A-Z]:$/i)) return ''
    return ''
  }
  const parent = normalized.substring(0, lastSep)
  if (parent.match(/^[A-Z]:$/i)) return parent + '\\'
  if (!parent) return ''
  return parent
}

onMounted(async () => {
  await Promise.all([loadDrives(), loadHomeDir()])
  document.addEventListener('keydown', handleKeydown)
})

onBeforeUnmount(() => {
  cleanupThumbnails()
  document.removeEventListener('keydown', handleKeydown)
  document.removeEventListener('mousemove', onResizeMove)
  document.removeEventListener('mouseup', onResizeEnd)
})
</script>

<style scoped>
.file-browser {
  display: flex;
  height: 100vh;
  background: #fff;
}

.sidebar {
  flex-shrink: 0;
  background: #f7f8fa;
  overflow: hidden;
  user-select: none;
  display: flex;
  flex-direction: column;
}

.resize-bar {
  width: 4px;
  flex-shrink: 0;
  cursor: col-resize;
  background: transparent;
  transition: background 0.15s;
  position: relative;
  z-index: 10;
}

.resize-bar:hover {
  background: #c0c4cc;
}

.resize-bar:active {
  background: #409eff;
}

.sidebar-section {
  padding: 10px 16px 4px;
  font-size: 12px;
  font-weight: 600;
  color: #909399;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.sidebar-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 16px;
  cursor: pointer;
  font-size: 13px;
  color: #303133;
  transition: background 0.15s;
}

.sidebar-item:hover {
  background: #e8edf3;
}

.sidebar-item.active {
  background: #ecf5ff;
  color: #409eff;
}

.sidebar-item-label {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.sidebar-tree {
  flex: 1;
  overflow-y: auto;
}

.sidebar-tree :deep(.el-tree) {
  background: transparent;
  font-size: 13px;
}

.sidebar-tree :deep(.el-tree-node__content) {
  height: 30px;
}

.sidebar-tree :deep(.el-tree-node__content:hover) {
  background: #e8edf3;
}

.sidebar-tree :deep(.el-tree-node.is-current > .el-tree-node__content) {
  background: #ecf5ff;
  color: #409eff;
}

.tree-node {
  display: flex;
  align-items: center;
  gap: 6px;
  overflow: hidden;
}

.tree-node-label {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.main-area {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  min-width: 0;
}

.toolbar {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 6px 16px;
  background: #fff;
  border-bottom: 1px solid #e4e7ed;
  flex-shrink: 0;
}

.breadcrumb {
  flex: 1;
  display: flex;
  align-items: center;
  gap: 2px;
  font-size: 13px;
  color: #606266;
  overflow: hidden;
  min-width: 0;
}

.bc-root {
  cursor: pointer;
  color: #409eff;
  white-space: nowrap;
}

.bc-root:hover {
  text-decoration: underline;
}

.bc-sep {
  color: #c0c4cc;
  margin: 0 2px;
  flex-shrink: 0;
}

.bc-item {
  cursor: pointer;
  color: #606266;
  white-space: nowrap;
}

.bc-item:hover {
  color: #409eff;
  text-decoration: underline;
}

.bc-item.active {
  color: #303133;
  font-weight: 500;
  cursor: default;
}

.bc-item.active:hover {
  text-decoration: none;
}

.toolbar-actions {
  display: flex;
  align-items: center;
  gap: 4px;
  flex-shrink: 0;
}

.toolbar-actions :deep(.el-button-group .el-button) {
  padding: 5px 8px;
}

.file-area {
  flex: 1;
  overflow-y: auto;
  padding: 12px;
}

.drives-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(160px, 1fr));
  gap: 12px;
}

.drive-card {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  padding: 24px 16px;
  border-radius: 8px;
  cursor: pointer;
  transition: background 0.15s;
}

.drive-card:hover {
  background: #f0f2f5;
}

.drive-name {
  font-size: 13px;
  color: #303133;
}

.empty-dir {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 200px;
  color: #909399;
  font-size: 14px;
}

/* List view */
.file-list {
  width: 100%;
}

.file-list-header {
  display: flex;
  align-items: center;
  padding: 8px 12px;
  font-size: 12px;
  font-weight: 600;
  color: #909399;
  border-bottom: 1px solid #e4e7ed;
  background: #fafafa;
}

.file-row {
  display: flex;
  align-items: center;
  padding: 6px 12px;
  font-size: 13px;
  color: #303133;
  cursor: pointer;
  transition: background 0.1s;
  border-bottom: 1px solid #f2f3f5;
}

.file-row:hover {
  background: #f0f2f5;
}

.col-name {
  flex: 1;
  display: flex;
  align-items: center;
  gap: 8px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  min-width: 0;
}

.type-icon-list {
  width: 16px;
  height: 16px;
  object-fit: contain;
  flex-shrink: 0;
}

.col-size {
  width: 90px;
  text-align: right;
  flex-shrink: 0;
  color: #909399;
}

.col-type {
  width: 80px;
  text-align: right;
  flex-shrink: 0;
  color: #909399;
}

/* Grid views */
.file-grid {
  display: grid;
  gap: 4px;
}

.file-grid.view-large {
  grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
}

.file-grid.view-medium {
  grid-template-columns: repeat(auto-fill, minmax(120px, 1fr));
}

.file-grid.view-small {
  grid-template-columns: repeat(auto-fill, minmax(80px, 1fr));
}

.file-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 8px 4px;
  border-radius: 6px;
  cursor: pointer;
  transition: background 0.1s;
}

.file-item:hover {
  background: #f0f2f5;
}

.file-thumb {
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
  border-radius: 4px;
}

.view-large .file-thumb {
  width: 192px;
  height: 192px;
}

.view-medium .file-thumb {
  width: 96px;
  height: 96px;
}

.view-small .file-thumb {
  width: 48px;
  height: 48px;
}

.file-thumb img {
  width: 100%;
  height: 100%;
  object-fit: contain;
  border-radius: 4px;
}

.folder-icon {
  width: 75%;
  height: 75%;
  object-fit: contain;
}

.type-icon-large {
  width: 128px;
  height: 128px;
  object-fit: contain;
}

.type-icon-medium {
  width: 64px;
  height: 64px;
  object-fit: contain;
}

.type-icon-small {
  width: 48px;
  height: 48px;
  object-fit: contain;
}

.file-name {
  width: 100%;
  text-align: center;
  font-size: 13px;
  color: #303133;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  line-height: 1.4;
  margin-top: 4px;
}

.view-small .file-name {
  font-size: 12px;
}
</style>