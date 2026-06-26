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
              <img v-if="fileIconMap['folder']" class="tree-folder-icon" :src="fileIconMap['folder']" />
              <el-icon v-else :size="16">
                <FolderOpened v-if="node.expanded" />
                <Folder v-else />
              </el-icon>
              <span class="tree-node-label">{{ data.name }}</span>
            </span>
          </template>
        </el-tree>
      </div>
      <div class="sidebar-section">
        网络驱动器
        <el-button size="small" text class="sidebar-manage-btn" @click="showWebdavManager = true">
          管理
        </el-button>
      </div>
      <div class="webdav-list">
        <div
          v-for="conn in webdavStore.connections"
          :key="conn.id"
          class="sidebar-item"
          :class="{ active: currentPath === 'webdav://' + conn.sessionId + '/' }"
          @click="navigateToWebdav(conn.sessionId)"
        >
          <el-icon :size="16"><Connection /></el-icon>
          <span class="sidebar-item-label">{{ conn.name }}</span>
        </div>
        <div v-if="webdavStore.connections.length === 0" class="sidebar-empty">
          暂无连接
        </div>
      </div>
    </aside>

    <WebdavManager v-if="showWebdavManager" v-model:visible="showWebdavManager" />

    <div class="resize-bar" @mousedown="onResizeStart"></div>

    <div class="main-area">
      <div class="toolbar">
        <div v-if="breadcrumbEditing" class="breadcrumb breadcrumb-editing">
          <input
            ref="breadcrumbInputRef"
            v-model="breadcrumbInputValue"
            class="bc-input"
            @keydown.enter="commitBreadcrumbEdit"
            @keydown.escape="cancelBreadcrumbEdit"
            @blur="commitBreadcrumbEdit"
          />
        </div>
        <div v-else class="breadcrumb" @click="startBreadcrumbEdit">
          <span class="bc-root" @click.stop="navigateTo('')">此电脑</span>
          <template v-for="(seg, i) in pathSegments" :key="i">
            <svg class="bc-sep" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <polyline points="9 18 15 12 9 6"/>
            </svg>
            <span
              :class="['bc-item', { active: i === pathSegments.length - 1 }]"
              @click.stop="navigateTo(seg.path)"
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
        </div>
      </div>

      <div class="file-area" ref="fileAreaRef" v-loading="loading" @contextmenu.prevent="onBackgroundContextMenu($event)">
        <div class="file-area-inner" @mousedown="onAreaMouseDown">
          <div
            v-if="selectionBox.active"
            class="selection-box"
            :style="selectionBoxStyle"
          ></div>
          <template v-if="!currentPath">
            <div class="drives-grid">
              <div
                v-for="drive in drives" :key="drive.path"
                class="drive-card"
                @dblclick="navigateTo(drive.path)"
                @contextmenu.prevent="onDriveContextMenu($event, drive)"
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
            <div v-if="viewMode === 'list'" class="file-list" :style="colVars">
              <div class="file-list-header">
                <span class="col-header col-name" @click="toggleSort('name')">
                  名称
                  <span v-if="sortCol === 'name'" class="sort-arrow">{{ sortDir === 'asc' ? '▲' : '▼' }}</span>
                  <span class="col-resize" @mousedown.stop.prevent="startResize($event, 'name')"></span>
                </span>
                <span class="col-header col-size" @click="toggleSort('size')">
                  大小
                  <span v-if="sortCol === 'size'" class="sort-arrow">{{ sortDir === 'asc' ? '▲' : '▼' }}</span>
                  <span class="col-resize" @mousedown.stop.prevent="startResize($event, 'size')"></span>
                </span>
                <span class="col-header col-type" @click="toggleSort('ext')">
                  类型
                  <span v-if="sortCol === 'ext'" class="sort-arrow">{{ sortDir === 'asc' ? '▲' : '▼' }}</span>
                  <span class="col-resize" @mousedown.stop.prevent="startResize($event, 'type')"></span>
                </span>
                <span class="col-header col-date" @click="toggleSort('modified')">
                  修改日期
                  <span v-if="sortCol === 'modified'" class="sort-arrow">{{ sortDir === 'asc' ? '▲' : '▼' }}</span>
                </span>
              </div>
              <div
                v-for="file in sortedFiles" :key="file.path"
                class="file-row"
                :class="{ selected: selectedFiles.has(file.path) }"
                @dblclick="handleDoubleClick(file, $event)"
                @contextmenu.prevent="onFileContextMenu($event, file)"
                @mousedown.left="onFileMouseDown($event, file)"
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
                <span class="col-date">{{ file.modified || '' }}</span>
              </div>
            </div>

            <div v-else :class="['file-grid', `view-${viewMode}`]">
              <div
                v-for="file in files" :key="file.path"
                class="file-item"
                :class="{ selected: selectedFiles.has(file.path) }"
                @dblclick="handleDoubleClick(file, $event)"
                @contextmenu.prevent="onFileContextMenu($event, file)"
                @mousedown.left="onFileMouseDown($event, file)"
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
    <ContextMenu
      :visible="contextMenu.visible"
      :x="contextMenu.x"
      :y="contextMenu.y"
      :items="contextMenuItems"
      @close="closeContextMenu"
    />
  </div>
</template>

<script setup>
import { ref, reactive, computed, watch, onMounted, onBeforeUnmount, nextTick } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { WebviewWindow } from '@tauri-apps/api/webviewWindow'
import { listen } from '@tauri-apps/api/event'
import { Folder, FolderOpened, Document, Picture, Monitor, Download, Connection, Edit, EditPen, CopyDocument, Delete, Plus } from '@element-plus/icons-vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { imageExts } from '@/utils/fileType'
import { useWebdavStore } from '@/stores/webdav'
import { useTabsStore } from '@/stores/tabs'
import WebdavManager from './WebdavManager.vue'
import ContextMenu from './ContextMenu.vue'

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

const webdavStore = useWebdavStore()
const tabsStore = useTabsStore()

const errorMsg = ref('')
const drives = ref([])
const homeDir = ref('')
const currentPath = ref(tabsStore.currentPath)
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
const showWebdavManager = ref(false)
let unlistenSync = null

// ── Context menu state ──
const contextMenu = reactive({ visible: false, x: 0, y: 0, file: null })
const clipboard = ref(null)

// ── Selection state ──
const selectedFiles = ref(new Set())
let selAnchor = null // anchor path for shift-click range selection
const isSelecting = ref(false)
const selectionBox = reactive({ active: false, startX: 0, startY: 0, curX: 0, curY: 0 })
const fileAreaRef = ref(null)

// ── List view columns / sort ──
const colW = reactive({ name: 300, size: 100, type: 90, date: 180 })
const sortCol = ref('name')
const sortDir = ref('asc')
const colResizing = ref(null)

const colVars = computed(() => ({
  '--col-name-w': colW.name + 'px',
  '--col-size-w': colW.size + 'px',
  '--col-type-w': colW.type + 'px',
  '--col-date-w': colW.date + 'px',
}))

const sortedFiles = computed(() => {
  const arr = [...files.value]
  const col = sortCol.value
  const dir = sortDir.value === 'asc' ? 1 : -1
  arr.sort((a, b) => {
    // Directories always first
    if (a.is_dir !== b.is_dir) return a.is_dir ? -1 : 1
    if (col === 'name') return dir * a.name.localeCompare(b.name)
    if (col === 'size') return dir * (a.size - b.size)
    if (col === 'ext') return dir * a.ext.localeCompare(b.ext)
    if (col === 'modified') return dir * a.modified.localeCompare(b.modified)
    return 0
  })
  return arr
})

function toggleSort(col) {
  if (sortCol.value === col) {
    sortDir.value = sortDir.value === 'asc' ? 'desc' : 'asc'
  } else {
    sortCol.value = col
    sortDir.value = 'asc'
  }
}

function startResize(e, col) {
  colResizing.value = { col, startX: e.clientX, startW: colW[col] }
  document.addEventListener('mousemove', onColResizeMove)
  document.addEventListener('mouseup', onColResizeUp)
}

function onColResizeMove(e) {
  if (!colResizing.value) return
  const { col, startX, startW } = colResizing.value
  const delta = e.clientX - startX
  colW[col] = Math.max(60, startW + delta)
}

function onColResizeUp() {
  colResizing.value = null
  document.removeEventListener('mousemove', onColResizeMove)
  document.removeEventListener('mouseup', onColResizeUp)
}

const selectionBoxStyle = computed(() => {
  if (!selectionBox.active) return {}
  const left = Math.min(selectionBox.startX, selectionBox.curX)
  const top = Math.min(selectionBox.startY, selectionBox.curY)
  const w = Math.abs(selectionBox.curX - selectionBox.startX)
  const h = Math.abs(selectionBox.curY - selectionBox.startY)
  return { left: left + 'px', top: top + 'px', width: w + 'px', height: h + 'px' }
})

function clearSelection() {
  selectedFiles.value = new Set()
}

function selectFile(file, clearOthers) {
  const s = new Set(selectedFiles.value)
  if (clearOthers) s.clear()
  if (s.has(file.path)) s.delete(file.path)
  else s.add(file.path)
  selectedFiles.value = s
}

function onFileMouseDown(e, file) {
  if (e.button !== 0) return
  if (e.ctrlKey || e.metaKey) {
    selectFile(file, false)
    selAnchor = file.path
  } else if (e.shiftKey && selAnchor) {
    const paths = files.value.map(f => f.path)
    const anchorIdx = paths.indexOf(selAnchor)
    const curIdx = paths.indexOf(file.path)
    if (anchorIdx !== -1 && curIdx !== -1) {
      const s = new Set()
      const [start, end] = anchorIdx < curIdx ? [anchorIdx, curIdx] : [curIdx, anchorIdx]
      for (let i = start; i <= end; i++) s.add(files.value[i].path)
      selectedFiles.value = s
    }
  } else {
    if (!selectedFiles.value.has(file.path)) {
      selectedFiles.value = new Set([file.path])
    }
    selAnchor = file.path
  }
}

function onAreaMouseDown(e) {
  if (e.button !== 0) return
  const target = e.target
  if (target.closest('.file-item, .file-row, .drive-card, .file-list-header, .empty-dir')) return

  clearSelection()
  selAnchor = null
  const rect = fileAreaRef.value?.getBoundingClientRect()
  if (!rect) return
  selectionBox.active = true
  isSelecting.value = true
  selectionBox.startX = e.clientX
  selectionBox.startY = e.clientY
  selectionBox.curX = e.clientX
  selectionBox.curY = e.clientY

  document.addEventListener('mousemove', onDocMouseMove)
  document.addEventListener('mouseup', onDocMouseUp)
}

function onDocMouseMove(e) {
  if (!isSelecting.value) return
  selectionBox.curX = e.clientX
  selectionBox.curY = e.clientY
}

function onDocMouseUp(e) {
  if (!isSelecting.value) return
  isSelecting.value = false
  document.removeEventListener('mousemove', onDocMouseMove)
  document.removeEventListener('mouseup', onDocMouseUp)

  if (selectionBox.active) {
    const dx = Math.abs(selectionBox.curX - selectionBox.startX)
    const dy = Math.abs(selectionBox.curY - selectionBox.startY)
    if (dx > 4 || dy > 4) {
      const sel = new Set()
      const box = getSelectionRect()
      const items = fileAreaRef.value?.querySelectorAll('.file-row, .file-item')
      if (items) {
        for (let i = 0; i < items.length; i++) {
          const r = items[i].getBoundingClientRect()
          if (rectsOverlap(box, r)) {
            if (files.value[i]) sel.add(files.value[i].path)
          }
        }
      }
      selectedFiles.value = sel
    }
    selectionBox.active = false
  }
}

function getSelectionRect() {
  return {
    left: Math.min(selectionBox.startX, selectionBox.curX),
    top: Math.min(selectionBox.startY, selectionBox.curY),
    right: Math.max(selectionBox.startX, selectionBox.curX),
    bottom: Math.max(selectionBox.startY, selectionBox.curY),
  }
}

function rectsOverlap(a, b) {
  return !(a.right < b.left || a.left > b.right || a.bottom < b.top || a.top > b.bottom)
}

function onFileContextMenu(e, file) {
  // If clicked file is not in selection, select only it
  if (!selectedFiles.value.has(file.path)) {
    selectedFiles.value = new Set([file.path])
  }
  contextMenu.x = e.clientX
  contextMenu.y = e.clientY
  contextMenu.file = file
  contextMenu.visible = true
}

function onBackgroundContextMenu(e) {
  clearSelection()
  contextMenu.x = e.clientX
  contextMenu.y = e.clientY
  contextMenu.file = null
  contextMenu.visible = true
}

function onDriveContextMenu(e, drive) {
  onFileContextMenu(e, { ...drive, is_dir: true })
}

function closeContextMenu() {
  contextMenu.visible = false
  contextMenu.file = null
}

const contextMenuItems = computed(() => {
  if (!contextMenu.visible) return []
  const file = contextMenu.file
  if (!file) return buildBackgroundMenu()
  if (isWebdavPath(file.path)) return buildWebdavMenu(file)
  if (isZipPath(file.path)) return buildZipMenu(file)
  return buildLocalMenu(file)
})

function buildBackgroundMenu() {
  const items = []
  if (currentPath.value) {
    if (isZipPath(currentPath.value)) {
      items.push({ label: '退出 ZIP', onClick: () => {
        const parsed = parseZipPath(currentPath.value)
        if (parsed) navigateTo(getParentPath(parsed.zipPath))
      }})
      items.push({ divider: true })
    } else if (isWebdavPath(currentPath.value)) {
      items.push({ label: '新建文件夹', onClick: handleWebdavBackgroundNewFolder })
      if (clipboard.value) {
        const disabled = clipboard.value.items.some(i => isWebdavPath(i.path))
        if (!disabled) {
          items.push({ divider: true })
          items.push({ label: '粘贴', onClick: () => handlePaste(currentPath.value) })
        }
      }
    } else {
      items.push({ label: '新建文件夹', onClick: handleNewFolder })
      items.push({ label: '新建文件', onClick: handleNewFile })
      if (clipboard.value) {
        items.push({ divider: true })
        const disabled = clipboard.value.items.some(i => isWebdavPath(i.path))
        items.push({ label: '粘贴', disabled, onClick: () => handlePaste(currentPath.value) })
      }
    }
    items.push({ divider: true })
  }
  items.push({ label: '刷新', onClick: refreshList })
  return items
}

function buildLocalMenu(file) {
  const items = []
  const multi = selectedFiles.value.size > 1

  if (multi) {
    // Multi-file context menu
    items.push({ label: `复制 ${selectedFiles.value.size} 项`, onClick: handleMultiCopy })
    items.push({ label: `剪切 ${selectedFiles.value.size} 项`, onClick: handleMultiCut })
    items.push({ divider: true })
    items.push({ label: `删除 ${selectedFiles.value.size} 项`, onClick: handleMultiDelete })
    return items
  }

  if (file.is_dir) {
    items.push({ label: '打开', onClick: () => navigateTo(file.path) })
    items.push({ divider: true })
    items.push({ label: '新建文件夹', onClick: () => handleNewFolderInDir(file.path) })
    items.push({ divider: true })
  } else {
    items.push({ label: '打开', onClick: () => handleDoubleClick(file) })
    if (isPreviewable(file.ext)) {
      items.push({ label: '预览', onClick: () => openPreviewWindow(file) })
    }
    items.push({ divider: true })
  }
  items.push({ label: '重命名', onClick: () => handleRename(file) })
  items.push({ label: '复制', onClick: () => handleCopy(file) })
  items.push({ label: '剪切', onClick: () => handleCut(file) })
  items.push({ divider: true })
  items.push({ label: '删除', onClick: () => handleDelete(file) })
  return items
}

function buildWebdavMenu(file) {
  const items = []
  const multi = selectedFiles.value.size > 1

  if (multi) {
    items.push({ label: `删除 ${selectedFiles.value.size} 项`, onClick: handleMultiWebdavDelete })
    return items
  }

  if (file.is_dir) {
    items.push({ label: '打开', onClick: () => navigateTo(file.path) })
    items.push({ divider: true })
    items.push({ label: '新建文件夹', onClick: () => handleWebdavNewFolder(file) })
    items.push({ divider: true })
  } else {
    items.push({ label: '打开', onClick: () => handleWebdavFileClick(file) })
    if (isPreviewable(file.ext)) {
      items.push({ label: '预览', onClick: () => handleWebdavFileClick(file) })
    }
    items.push({ divider: true })
  }
  items.push({ label: '重命名', onClick: () => handleWebdavRename(file) })
  items.push({ label: '复制', disabled: true })
  items.push({ label: '剪切', disabled: true })
  items.push({ divider: true })
  items.push({ label: '删除', onClick: () => handleWebdavDelete(file) })
  return items
}

function buildZipMenu(file) {
  const items = []
  const multi = selectedFiles.value.size > 1
  if (multi) {
    items.push({ label: `删除 ${selectedFiles.value.size} 项`, disabled: true })
    return items
  }
  if (file.is_dir) {
    items.push({ label: '打开', onClick: () => navigateTo(file.path) })
  } else {
    items.push({ label: '打开', onClick: () => handleDoubleClick(file) })
    if (isPreviewable(file.ext)) {
      items.push({ label: '预览', onClick: () => handleZipFileClick(file) })
    }
  }
  return items
}

function getParentDir(filePath) {
  const idx = Math.max(filePath.lastIndexOf('/'), filePath.lastIndexOf('\\'))
  if (idx <= 0) return filePath.slice(0, 1) + '/'
  if (idx === 2 && filePath[1] === ':') return filePath.slice(0, 3) // C:\
  return filePath.slice(0, idx)
}

function getFileName(filePath) {
  const idx = Math.max(filePath.lastIndexOf('/'), filePath.lastIndexOf('\\'))
  if (idx === -1) return filePath
  return filePath.slice(idx + 1)
}

// ── File operations ──

async function handleNewFolder() {
  try {
    const { value } = await ElMessageBox.prompt('文件夹名称', '新建文件夹', {
      confirmButtonText: '创建', cancelButtonText: '取消',
      inputPattern: /.+/,
      inputErrorMessage: '名称不能为空',
    })
    if (value) {
      const folderPath = currentPath.value + '/' + value
      await invoke('create_dir', { path: folderPath })
      ElMessage.success('文件夹已创建')
      refreshList()
    }
  } catch (e) {
    if (e !== 'cancel') ElMessage.error('创建文件夹失败: ' + String(e))
  }
}

async function handleNewFolderInDir(dirPath) {
  try {
    const { value } = await ElMessageBox.prompt('文件夹名称', '新建文件夹', {
      confirmButtonText: '创建', cancelButtonText: '取消',
      inputPattern: /.+/,
      inputErrorMessage: '名称不能为空',
    })
    if (value) {
      await invoke('create_dir', { path: dirPath + '/' + value })
      ElMessage.success('文件夹已创建')
      refreshList()
    }
  } catch (e) {
    if (e !== 'cancel') ElMessage.error('创建文件夹失败: ' + String(e))
  }
}

async function handleNewFile() {
  try {
    const { value } = await ElMessageBox.prompt('文件名称', '新建文件', {
      confirmButtonText: '创建', cancelButtonText: '取消',
      inputPattern: /.+/,
      inputErrorMessage: '名称不能为空',
    })
    if (value) {
      const filePath = currentPath.value + '/' + value
      await invoke('create_file', { path: filePath })
      ElMessage.success('文件已创建')
      refreshList()
    }
  } catch (e) {
    if (e !== 'cancel') ElMessage.error('创建文件失败: ' + String(e))
  }
}

async function handleRename(file) {
  try {
    const { value } = await ElMessageBox.prompt('新名称', '重命名', {
      confirmButtonText: '确定', cancelButtonText: '取消',
      inputValue: file.name,
      inputPattern: /.+/,
      inputErrorMessage: '名称不能为空',
    })
    if (value && value !== file.name) {
      const parent = getParentDir(file.path)
      const target = parent + '/' + value
      await invoke('rename_item', { from: file.path, to: target })
      ElMessage.success('重命名成功')
      refreshList()
    }
  } catch (e) {
    if (e !== 'cancel') ElMessage.error('重命名失败: ' + String(e))
  }
}

async function handleDelete(file) {
  try {
    await ElMessageBox.confirm(
      `确定要删除 "${file.name}" 吗？${file.is_dir ? '该操作将删除目录及其所有内容。' : ''}`,
      '删除确认',
      { confirmButtonText: '删除', cancelButtonText: '取消', type: 'warning' }
    )
    await invoke('remove', { path: file.path })
    ElMessage.success('已删除')
    refreshList()
  } catch (e) {
    if (e !== 'cancel') ElMessage.error('删除失败: ' + String(e))
  }
}

// ── Multi-file operations ──

function getSelectedPaths() {
  return [...selectedFiles.value]
}

async function handleMultiCopy() {
  const paths = getSelectedPaths()
  if (paths.length === 0) return
  clipboard.value = {
    items: paths.map(p => ({ path: p, name: getFileName(p) })),
    operation: 'copy',
  }
  ElMessage.success(`已复制 ${paths.length} 项`)
}

async function handleMultiCut() {
  const paths = getSelectedPaths()
  if (paths.length === 0) return
  clipboard.value = {
    items: paths.map(p => ({ path: p, name: getFileName(p) })),
    operation: 'cut',
  }
  ElMessage.success(`已剪切 ${paths.length} 项`)
}

async function handleMultiDelete() {
  const paths = getSelectedPaths()
  if (paths.length === 0) return
  try {
    await ElMessageBox.confirm(
      `确定要删除选中的 ${paths.length} 项吗？`,
      '批量删除确认',
      { confirmButtonText: '删除', cancelButtonText: '取消', type: 'warning' }
    )
    for (const p of paths) {
      await invoke('remove', { path: p })
    }
    ElMessage.success(`已删除 ${paths.length} 项`)
    selectedFiles.value = new Set()
    refreshList()
  } catch (e) {
    if (e !== 'cancel') ElMessage.error('删除失败: ' + String(e))
  }
}

async function handleMultiWebdavDelete() {
  const paths = getSelectedPaths()
  if (paths.length === 0) return
  try {
    await ElMessageBox.confirm(
      `确定要删除选中的 ${paths.length} 项吗？`,
      '批量删除确认',
      { confirmButtonText: '删除', cancelButtonText: '取消', type: 'warning' }
    )
    for (const p of paths) {
      const sessionId = getWebdavSessionId(p)
      const remotePath = getWebdavRemotePath(p)
      await invoke('webdav_remove', { sessionId, path: remotePath })
    }
    ElMessage.success(`已删除 ${paths.length} 项`)
    selectedFiles.value = new Set()
    refreshList()
  } catch (e) {
    if (e !== 'cancel') ElMessage.error('删除失败: ' + String(e))
  }
}

function handleCopy(file) {
  clipboard.value = {
    items: [{ path: file.path, name: file.name }],
    operation: 'copy',
  }
  ElMessage.success('已复制')
}

function handleCut(file) {
  clipboard.value = {
    items: [{ path: file.path, name: file.name }],
    operation: 'cut',
  }
  ElMessage.success('已剪切')
}

async function handlePaste(dirPath) {
  if (!clipboard.value) return
  const { items, operation } = clipboard.value
  try {
    const isWebdavDest = isWebdavPath(dirPath)
    for (const item of items) {
      const name = getFileName(item.path)
      if (isWebdavDest) {
        // Upload local file to WebDAV
        const sessionId = getWebdavSessionId(dirPath)
        const remotePath = getWebdavRemotePath(dirPath)
        const target = (remotePath + '/' + name).replace(/\/+/g, '/')
        await invoke('webdav_upload', { sessionId, localPath: item.path, remotePath: target })
        if (operation === 'cut') {
          await invoke('remove', { path: item.path })
        }
      } else {
        const target = dirPath + '/' + name
        if (operation === 'copy') {
          await invoke('copy_item', { from: item.path, to: target })
        } else {
          await invoke('move_item', { from: item.path, to: target })
        }
      }
    }
    ElMessage.success(operation === 'copy' ? '粘贴完成' : '移动完成')
    clipboard.value = null
    refreshList()
  } catch (e) {
    ElMessage.error('粘贴失败: ' + e)
  }
}

// ── WebDAV file operations ──

async function handleWebdavNewFolder(file) {
  const sessionId = getWebdavSessionId(file.path)
  const remotePath = getWebdavRemotePath(file.path)
  try {
    const { value } = await ElMessageBox.prompt('文件夹名称', '新建文件夹', {
      confirmButtonText: '创建', cancelButtonText: '取消',
      inputPattern: /.+/,
      inputErrorMessage: '名称不能为空',
    })
    if (value) {
      const target = (remotePath + '/' + value).replace(/\/+/g, '/')
      await invoke('webdav_create_dir', { sessionId, path: target })
      ElMessage.success('文件夹已创建')
      refreshList()
    }
  } catch (e) {
    if (e !== 'cancel') ElMessage.error('创建文件夹失败: ' + String(e))
  }
}

async function handleWebdavRename(file) {
  const sessionId = getWebdavSessionId(file.path)
  const remotePath = getWebdavRemotePath(file.path)
  const parent = getParentDir(remotePath)
  try {
    const { value } = await ElMessageBox.prompt('新名称', '重命名', {
      confirmButtonText: '确定', cancelButtonText: '取消',
      inputValue: file.name,
      inputPattern: /.+/,
      inputErrorMessage: '名称不能为空',
    })
    if (value && value !== file.name) {
      const target = (parent + '/' + value).replace(/\/+/g, '/')
      await invoke('webdav_rename', { sessionId, from: remotePath, to: target })
      ElMessage.success('重命名成功')
      refreshList()
    }
  } catch (e) {
    if (e !== 'cancel') ElMessage.error('重命名失败: ' + String(e))
  }
}

async function handleWebdavBackgroundNewFolder() {
  const sessionId = getWebdavSessionId(currentPath.value)
  const remotePath = getWebdavRemotePath(currentPath.value)
  try {
    const { value } = await ElMessageBox.prompt('文件夹名称', '新建文件夹', {
      confirmButtonText: '创建', cancelButtonText: '取消',
      inputPattern: /.+/,
      inputErrorMessage: '名称不能为空',
    })
    if (value) {
      const target = (remotePath + '/' + value).replace(/\/+/g, '/')
      await invoke('webdav_create_dir', { sessionId, path: target })
      ElMessage.success('文件夹已创建')
      refreshList()
    }
  } catch (e) {
    if (e !== 'cancel') ElMessage.error('创建文件夹失败: ' + String(e))
  }
}

async function handleWebdavDelete(file) {
  const sessionId = getWebdavSessionId(file.path)
  const remotePath = getWebdavRemotePath(file.path)
  try {
    await ElMessageBox.confirm(
      `确定要删除 "${file.name}" 吗？${file.is_dir ? '该操作将删除目录及其所有内容。' : ''}`,
      '删除确认',
      { confirmButtonText: '删除', cancelButtonText: '取消', type: 'warning' }
    )
    await invoke('webdav_remove', { sessionId, path: remotePath })
    ElMessage.success('已删除')
    refreshList()
  } catch (e) {
    if (e !== 'cancel') ElMessage.error('删除失败: ' + String(e))
  }
}

function isWebdavPath(path) {
  return webdavStore.isWebdavPath(path)
}

function parseWebdavPath(path) {
  return webdavStore.parseWebdavPath(path)
}

function getWebdavSessionId(path) {
  if (!isWebdavPath(path)) return ''
  const rest = path.slice('webdav://'.length)
  const slash = rest.indexOf('/')
  return slash === -1 ? rest : rest.slice(0, slash)
}

function getWebdavRemotePath(path) {
  const parsed = parseWebdavPath(path)
  return parsed ? parsed.path : ''
}

// ── ZIP path helpers ──
function isZipPath(path) {
  return typeof path === 'string' && path.startsWith('zip://')
}

function parseZipPath(path) {
  if (!isZipPath(path)) return null
  const rest = path.slice(6)
  // Match .zip or .cbz (case insensitive) followed by /, \, or end-of-string
  const match = rest.match(/(\.zip|\.cbz)([\/\\]|$)/i)
  if (!match) {
    return { zipPath: rest, innerPath: '' }
  }
  const zipPath = rest.slice(0, match.index + match[1].length)
  const innerPath = rest.slice(match.index + match[1].length).replace(/^[\/\\]+/, '')
  return { zipPath, innerPath }
}

function buildZipPath(zipPath, innerPath) {
  if (!innerPath) return 'zip://' + zipPath
  return 'zip://' + zipPath + '/' + innerPath
}

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
  if (isWebdavPath(currentPath.value)) {
    const parsed = parseWebdavPath(currentPath.value)
    if (!parsed) return []
    const conn = webdavStore.connections.find(c => c.sessionId === getWebdavSessionId(currentPath.value))
    const rootName = conn ? conn.name : 'WebDAV'
    if (!parsed.path || parsed.path === '/') return [{ name: rootName, path: currentPath.value }]
    const parts = parsed.path.split('/').filter(Boolean)
    const result = [{ name: rootName, path: 'webdav://' + getWebdavSessionId(currentPath.value) + '/' }]
    let build = ''
    for (const seg of parts) {
      build += '/' + seg
      result.push({
        name: seg,
        path: 'webdav://' + getWebdavSessionId(currentPath.value) + build,
      })
    }
    return result
  }
  if (isZipPath(currentPath.value)) {
    const parsed = parseZipPath(currentPath.value)
    if (!parsed) return []
    const zipName = parsed.zipPath.split(/[/\\]/).pop() || parsed.zipPath
    if (!parsed.innerPath) return [{ name: zipName, path: currentPath.value }]
    const parts = parsed.innerPath.split('/').filter(Boolean)
    const zipRootPath = 'zip://' + parsed.zipPath
    const result = [{ name: zipName, path: zipRootPath }]
    let build = ''
    for (const seg of parts) {
      build += '/' + seg
      result.push({ name: seg, path: 'zip://' + parsed.zipPath + build })
    }
    return result
  }
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

const breadcrumbEditing = ref(false)
const breadcrumbInputValue = ref('')
const breadcrumbInputRef = ref(null)

function startBreadcrumbEdit() {
  breadcrumbInputValue.value = currentPath.value
  breadcrumbEditing.value = true
  nextTick(() => {
    breadcrumbInputRef.value?.select()
  })
}

function commitBreadcrumbEdit() {
  if (!breadcrumbEditing.value) return
  breadcrumbEditing.value = false
  const val = breadcrumbInputValue.value.trim()
  if (val && val !== currentPath.value) {
    navigateTo(val)
  }
}

function cancelBreadcrumbEdit() {
  breadcrumbEditing.value = false
}

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
  tabsStore.updateCurrentTab(path)
  if (path) {
    await loadDir(path)
    if (!isWebdavPath(path) && !isZipPath(path)) {
      try { treeRef.value?.setCurrentKey(path) } catch {}
    }
  } else {
    files.value = []
  }
}

async function loadDir(path, retried) {
  loading.value = true
  cleanupThumbnails()
  try {
    if (isWebdavPath(path)) {
      const sessionId = getWebdavSessionId(path)
      const remotePath = getWebdavRemotePath(path)
      files.value = await invoke('webdav_list', { sessionId, path: remotePath || '/' })
      loadFileIconsForCurrentDir()
    } else if (isZipPath(path)) {
      const parsed = parseZipPath(path)
      if (parsed) {
        files.value = await invoke('zip_list', { zipPath: parsed.zipPath, innerPath: parsed.innerPath })
        loadFileIconsForCurrentDir()
      } else {
        files.value = []
      }
    } else {
      files.value = await invoke('read_dir', { path })
      if (viewMode.value !== 'list') loadThumbnails()
      loadFileIconsForCurrentDir()
    }
  } catch (e) {
    const msg = String(e)
    console.error('loadDir error:', msg, e)
    if (isWebdavPath(path) && (msg.includes('会话不存在') || msg.includes('已断开')) && !retried) {
      const sessionId = getWebdavSessionId(path)
      const ok = await webdavStore.ensureConnected(sessionId)
      if (ok) {
        const newSessionId = webdavStore.connections.find(c => c.id === sessionId)?.sessionId || sessionId
        if (newSessionId !== sessionId) {
          currentPath.value = path.replace('webdav://' + sessionId, 'webdav://' + newSessionId)
        }
        return loadDir(currentPath.value, true)
      }
    }
    errorMsg.value = msg
    ElMessage.error(msg)
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

function handleDoubleClick(file, event) {
  const isCtrl = event?.ctrlKey || event?.metaKey
  if (file.is_dir) {
    if (isCtrl) {
      tabsStore.addTab(file.path)
    }
    navigateTo(file.path)
  } else if (['zip', 'cbz'].includes((file.ext || '').toLowerCase())) {
    handleZipOrCbzClick(file, isCtrl)
  } else if (isWebdavPath(file.path)) {
    handleWebdavFileClick(file)
  } else if (isZipPath(currentPath.value)) {
    handleZipFileClick(file)
  } else if (isPreviewable(file.ext)) {
    openPreviewWindow(file)
  } else {
    invoke('open_file', { path: file.path }).catch(e => ElMessage.error(String(e)))
  }
}

async function handleZipOrCbzClick(file, isCtrl) {
  if (isWebdavPath(file.path)) {
    try {
      const sessionId = getWebdavSessionId(file.path)
      const remotePath = getWebdavRemotePath(file.path)
      const tempPath = await invoke('webdav_download', { sessionId, path: remotePath })
      const zipPath = 'zip://' + tempPath
      tabsStore.addTab(zipPath)
      navigateTo(zipPath)
    } catch (e) {
      ElMessage.error('下载 ZIP 文件失败: ' + e)
    }
  } else {
    if (isCtrl) tabsStore.addTab('zip://' + file.path)
    navigateTo('zip://' + file.path)
  }
}

async function handleWebdavFileClick(file) {
  const sessionId = getWebdavSessionId(file.path)
  const remotePath = getWebdavRemotePath(file.path)

  try {
    const tempPath = await invoke('webdav_download', { sessionId, path: remotePath })
    if (isPreviewable(file.ext)) {
      openPreviewWindow({ path: tempPath, name: file.name, ext: file.ext, size: file.size })
    } else {
      await invoke('open_file', { path: tempPath })
    }
  } catch (e) {
    ElMessage.error('文件操作失败: ' + e)
  }
}

async function handleZipFileClick(file) {
  const parsed = parseZipPath(file.path)
  if (!parsed) return
  try {
    const data = await invoke('zip_read_binary', { zipPath: parsed.zipPath, innerPath: parsed.innerPath })
    const tempPath = await invoke('save_temp_file', { data, name: file.name })
    if (isPreviewable(file.ext)) {
      openPreviewWindow({ path: tempPath, name: file.name, ext: file.ext, size: file.size })
    } else {
      await invoke('open_file', { path: tempPath }).catch(e => ElMessage.error(String(e)))
    }
  } catch (e) {
    ElMessage.error('文件操作失败: ' + e)
  }
}

async function navigateToWebdav(sessionId) {
  const conn = webdavStore.connections.find(c => c.sessionId === sessionId || c.id === sessionId)
  if (conn) {
    if (!conn.password) {
      try {
        const { value } = await ElMessageBox.prompt(
          `请输入 "${conn.name}" 的密码`,
          'WebDAV 密码',
          { inputType: 'password', confirmButtonText: '确定', cancelButtonText: '取消' }
        )
        if (value !== undefined) {
          // updateConnection calls connect which creates a valid session
          await webdavStore.updateConnection(conn.id, conn.name, conn.url, conn.username, value)
        } else {
          return
        }
      } catch {
        return
      }
    } else {
      // Has saved password, just ensure a fresh session for this launch
      await webdavStore.ensureConnected(conn.id)
    }
    sessionId = conn.sessionId
  }
  const path = 'webdav://' + sessionId + '/'
  await navigateTo(path)
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
  if (isZipPath(path)) {
    const parsed = parseZipPath(path)
    if (!parsed) return ''
    if (!parsed.innerPath) {
      // At root of zip — go back to the directory containing the zip file
      const zipDir = getParentPath(parsed.zipPath)
      return zipDir
    }
    const parts = parsed.innerPath.replace(/\\/g, '/').split('/').filter(Boolean)
    parts.pop()
    const parentInner = parts.join('/')
    return 'zip://' + parsed.zipPath + (parentInner ? '/' + parentInner : '')
  }
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
  console.log('FileBrowser onMounted start')
  await Promise.all([loadDrives(), loadHomeDir()])
  console.log('FileBrowser onMounted homeDir=', homeDir.value, 'drives=', drives.value.length)

  loadFileIcon('folder', 16)
  document.addEventListener('keydown', handleKeydown)

  // ── Ensure at least one tab exists ──
  if (tabsStore.tabs.length === 0) {
    const defaultPath = homeDir.value || (drives.value[0] ? drives.value[0].path : '')
    console.log('FileBrowser creating initial tab with path=', defaultPath)
    tabsStore.addTab(defaultPath)
  }
  console.log('tabsStore state:', JSON.stringify(tabsStore.tabs.map(t => ({ id: t.id, path: t.path, title: t.title }))), 'activeId:', tabsStore.activeId)

  // Sync current path from active tab
  const tabPath = tabsStore.currentPath
  console.log('FileBrowser syncing, tabPath=', tabPath, 'currentPath=', currentPath.value)
  if (tabPath && tabPath !== currentPath.value) {
    currentPath.value = tabPath
    console.log('FileBrowser loading dir from onMounted:', tabPath)
    loadDir(tabPath)
  } else if (!currentPath.value) {
    console.log('FileBrowser showing drives view')
    files.value = []
  }

  unlistenSync = await listen('webdav-sync-status', (event) => {
    webdavStore.updateSyncStatus(event.payload)
  })
  console.log('FileBrowser onMounted done')
})

// React to tab activation from title bar
watch(() => tabsStore.activeId, (newId, oldId) => {
  try {
    if (newId && newId !== oldId) {
      const path = tabsStore.currentPath
      if (path !== currentPath.value) {
        currentPath.value = path || ''
        if (path) loadDir(path)
        else files.value = []
      }
    }
  } catch (e) {
    console.error('Tab watch error:', e)
  }
})

onBeforeUnmount(() => {
  cleanupThumbnails()
  document.removeEventListener('keydown', handleKeydown)
  document.removeEventListener('mousemove', onResizeMove)
  document.removeEventListener('mouseup', onResizeEnd)
  document.removeEventListener('mousemove', onColResizeMove)
  document.removeEventListener('mouseup', onColResizeUp)
  if (unlistenSync) unlistenSync()
})
</script>

<style scoped>
.file-browser {
  display: flex;
  height: 100%;
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

.tree-folder-icon {
  width: 16px;
  height: 16px;
  object-fit: contain;
  flex-shrink: 0;
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
  cursor: pointer;
}

.breadcrumb.breadcrumb-editing {
  cursor: default;
}
.bc-input {
  width: 100%;
  border: 1px solid #409eff;
  outline: none;
  padding: 2px 6px;
  font-size: 13px;
  border-radius: 3px;
  background: #fff;
  color: #303133;
  height: 26px;
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
  user-select: none;
}

.file-area-inner {
  position: relative;
  min-height: 100%;
}

.selection-box {
  position: fixed;
  background: rgba(64, 158, 255, 0.10);
  border: 1px solid rgba(64, 158, 255, 0.40);
  pointer-events: none;
  z-index: 100;
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
  table-layout: fixed;
}

.file-list-header {
  display: flex;
  align-items: stretch;
  position: sticky;
  top: 0;
  z-index: 10;
  font-size: 12px;
  font-weight: 600;
  color: #909399;
  border-bottom: 1px solid #e4e7ed;
  background: #fafafa;
}

.col-header {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 8px 12px;
  cursor: pointer;
  user-select: none;
  white-space: nowrap;
  flex-shrink: 0;
  position: relative;
  transition: background 0.1s;
}

.col-header:hover {
  background: #e8edf3;
}

.sort-arrow {
  font-size: 10px;
  color: #409eff;
}

.col-resize {
  position: absolute;
  right: -3px;
  top: 0;
  bottom: 0;
  width: 6px;
  cursor: col-resize;
  z-index: 5;
}

.col-resize:hover {
  background: rgba(64, 158, 255, 0.3);
}

.file-row {
  display: flex;
  align-items: center;
  font-size: 13px;
  color: #303133;
  cursor: pointer;
  transition: background 0.1s;
  border-bottom: 1px solid #f2f3f5;
}

.file-row:hover {
  background: #f0f2f5;
}

.file-row.context-selected,
.file-item.context-selected,
.file-row.selected,
.file-item.selected {
  background: #ecf5ff;
}

.col-name,
.col-header.col-name {
  width: var(--col-name-w);
  flex-shrink: 0;
  display: flex;
  align-items: center;
  gap: 8px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  padding: 6px 12px;
}

.col-size,
.col-header.col-size {
  width: var(--col-size-w);
  flex-shrink: 0;
  text-align: right;
  padding: 6px 12px;
  color: #909399;
  white-space: nowrap;
}

.col-type,
.col-header.col-type {
  width: var(--col-type-w);
  flex-shrink: 0;
  text-align: right;
  padding: 6px 12px;
  color: #909399;
  white-space: nowrap;
}

.col-date,
.col-header.col-date {
  width: var(--col-date-w);
  flex-shrink: 0;
  padding: 6px 12px;
  color: #909399;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.type-icon-list {
  width: 16px;
  height: 16px;
  object-fit: contain;
  flex-shrink: 0;
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

/* WebDAV sidebar */
.sidebar-manage-btn {
  float: right;
  margin-top: -2px;
  font-size: 12px;
  text-transform: none;
  letter-spacing: 0;
}

.webdav-list {
  flex-shrink: 0;
  max-height: 200px;
  overflow-y: auto;
  margin-bottom: 4px;
}

.sidebar-empty {
  padding: 6px 16px;
  font-size: 12px;
  color: #c0c4cc;
}
</style>