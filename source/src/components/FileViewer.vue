<template>
  <div class="file-preview-overlay">
    <div v-if="showZoomControls" class="zoom-bar">
      <el-tooltip content="缩小" placement="bottom">
        <el-button size="small" :icon="ZoomOut" text bg :disabled="zoom <= 25" @click="zoom = Math.max(25, zoom - 10)" />
      </el-tooltip>
      <span class="zoom-label">{{ zoom }}%</span>
      <el-tooltip content="放大" placement="bottom">
        <el-button size="small" :icon="ZoomIn" text bg :disabled="zoom >= 200" @click="zoom = Math.min(200, zoom + 10)" />
      </el-tooltip>
      <el-tooltip content="1:1" placement="bottom">
        <el-button size="small" text bg :disabled="zoom === 100" @click="zoom = 100">1:1</el-button>
      </el-tooltip>
    </div>
    <div v-loading="loading" :element-loading-text="loadingText" class="preview-container">
      <div v-if="!isSupported" class="unsupported-tip">
        <el-icon :size="64" color="#909399"><Warning /></el-icon>
        <p>该格式暂不支持预览</p>
      </div>

      <iframe v-if="fileType === 'pdf' && pdfUrl" :src="pdfUrl" class="pdf-iframe" />

      <VueOfficeExcel v-else-if="fileType === 'excel'"
        :src="officeSrc" class="excel-preview" @rendered="handleRendered" />

      <div v-else-if="fileType === 'docx'" ref="officeViewportRef" class="office-viewport" @wheel.prevent="handleCtrlWheel">
        <div class="office-scroll-area">
          <div class="office-zoom-layer" :style="{ zoom: `${zoom}%` }">
            <VueOfficeDocx :src="officeSrc" :options="{ breakPages: true }"
              style="min-width: 600px;" @rendered="handleRendered" />
          </div>
        </div>
      </div>

      <pre v-else-if="fileType === 'text'" class="text-preview" v-text="textContent" />

      <Codemirror v-else-if="fileType === 'code'"
        :model-value="textContent" :extensions="codeExtensions"
        :disabled="true" :style="{ height: '100%' }" :autofocus="false" />
    </div>
  </div>
</template>

<script setup>
import { ref, computed, watch, nextTick, onMounted, onBeforeUnmount } from 'vue'
import { Warning, Close, ZoomIn, ZoomOut } from '@element-plus/icons-vue'
import { ElMessage } from 'element-plus'
import { invoke } from '@tauri-apps/api/core'

import VueOfficeExcel from '@vue-office/excel'
import '@vue-office/excel/lib/index.css'

import VueOfficeDocx from '@vue-office/docx'
import '@vue-office/docx/lib/index.css'

import { Codemirror } from 'vue-codemirror'
import { oneDark } from '@codemirror/theme-one-dark'
import { syntaxHighlighting, HighlightStyle } from '@codemirror/language'
import { tags } from '@lezer/highlight'
import { Prec } from '@codemirror/state'
import { EditorView } from 'codemirror'
import { javascript } from '@codemirror/lang-javascript'
import { python } from '@codemirror/lang-python'
import { html } from '@codemirror/lang-html'
import { css } from '@codemirror/lang-css'
import { json } from '@codemirror/lang-json'
import { xml } from '@codemirror/lang-xml'
import { markdown } from '@codemirror/lang-markdown'
import { sql } from '@codemirror/lang-sql'

const customHighlight = HighlightStyle.define([
  { tag: tags.keyword, color: '#6cb6ff' },
  { tag: tags.definitionKeyword, color: '#6cb6ff', fontWeight: 'bold' },
  { tag: tags.moduleKeyword, color: '#6cb6ff' },
  { tag: tags.controlKeyword, color: '#6cb6ff' },
  { tag: tags.modifier, color: '#6cb6ff' },
  { tag: tags.string, color: '#98c379' },
  { tag: tags.special(tags.string), color: '#98c379' },
  { tag: tags.number, color: '#d19a66' },
  { tag: tags.bool, color: '#e0e0e0', fontWeight: 'bold' },
  { tag: tags.null, color: '#5c6370' },
  { tag: tags.name, color: '#dcdfe4' },
  { tag: tags.propertyName, color: '#dcdfe4' },
  { tag: tags.attributeName, color: '#dcdfe4' },
  { tag: tags.variableName, color: '#dcdfe4' },
  { tag: tags.comment, color: '#5c6370', fontStyle: 'italic' },
  { tag: tags.typeName, color: '#e5c07b' },
  { tag: tags.function, color: '#61afef' },
  { tag: tags.className, color: '#e5c07b' },
  { tag: tags.operator, color: '#56b6c2' },
  { tag: tags.arithmeticOperator, color: '#56b6c2' },
  { tag: tags.compareOperator, color: '#56b6c2' },
  { tag: tags.logicOperator, color: '#56b6c2' },
  { tag: tags.definitionOperator, color: '#56b6c2' },
  { tag: tags.updateOperator, color: '#56b6c2' },
  { tag: tags.bracket, color: '#abb2bf' },
  { tag: tags.punctuation, color: '#abb2bf' },
  { tag: tags.separator, color: '#abb2bf' },
  { tag: tags.regexp, color: '#98c379' },
  { tag: tags.labelName, color: '#e5c07b' },
])

const props = defineProps({
  filePath: { type: String, default: '' },
  fileExt: { type: String, default: '' },
  fileName: { type: String, default: '' },
  fileSize: { type: Number, default: 0 },
})
const emit = defineEmits(['close'])
const close = () => emit('close')

const loading = ref(false)
const loadingText = ref('加载中...')
const pdfUrl = ref(null)
const officeSrc = ref('')
const textContent = ref('')
const officeViewportRef = ref(null)
const zoom = ref(100)
let officeBlobUrl = null

const ext = computed(() => (props.fileExt || '').toLowerCase().replace(/^\./, ''))

const plainExts = ['txt', 'log', 'ini', 'cfg', 'env', 'conf']

const codeExts = {
  js: javascript(), ts: javascript({ typescript: true }), jsx: javascript({ jsx: true }), tsx: javascript({ jsx: true, typescript: true }),
  py: python(), html: html(), htm: html(), css: css(), scss: css(), less: css(),
  json: json(), xml: xml(), xhtml: xml(), svg: xml(), md: markdown(), sql: sql(),
  yaml: [], yml: [], java: [], cpp: [], c: [], cs: [], php: [], rb: [], go: [], rs: [],
  swift: [], kt: [], sh: [], bash: [], bat: [], ps1: [], zsh: [], pl: [], lua: [], r: [], dart: [],
  vue: [html(), javascript()], svelte: [html(), javascript()],
}

const fileType = computed(() => {
  const e = ext.value
  if (e === 'pdf') return 'pdf'
  if (e === 'xlsx' || e === 'xls') return 'excel'
  if (e === 'docx' || e === 'doc') return 'docx'
  if (plainExts.includes(e)) return 'text'
  if (e in codeExts) return 'code'
  return 'unsupported'
})

const codeExtensions = computed(() => {
  const e = ext.value; const exts = codeExts[e]
  const base = [oneDark, Prec.high(syntaxHighlighting(customHighlight)), EditorView.editable.of(false), EditorView.lineWrapping]
  if (!exts) return base
  if (Array.isArray(exts)) return exts.length === 0 ? base : [...exts, ...base]
  return [exts, ...base]
})

const isSupported = computed(() => fileType.value !== 'unsupported')
const showZoomControls = computed(() => fileType.value === 'docx')

const handleCtrlWheel = (e) => {
  if (!e.ctrlKey) return
  zoom.value = Math.max(25, Math.min(200, zoom.value + (e.deltaY > 0 ? -10 : 10)))
}

watch(showZoomControls, (v) => { if (!v) zoom.value = 100 })

async function loadFile() {
  loading.value = true
  loadingText.value = '加载中...'
  const type = fileType.value

  if (type === 'unsupported') {
    loading.value = false
    return
  }

  try {
    if (type === 'text' || type === 'code') {
      textContent.value = await invoke('read_file_text', { path: props.filePath })
      loading.value = false
    } else if (type === 'pdf') {
      loadingText.value = '加载 PDF 中...'
      const data = await invoke('read_file_binary', { path: props.filePath })
      const blob = new Blob([new Uint8Array(data)], { type: 'application/pdf' })
      pdfUrl.value = URL.createObjectURL(blob)
      loading.value = false
    } else if (type === 'excel' || type === 'docx') {
      const data = await invoke('read_file_binary', { path: props.filePath })
      const mime = type === 'excel' ? 'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet' : 'application/vnd.openxmlformats-officedocument.wordprocessingml.document'
      const blob = new Blob([new Uint8Array(data)], { type: mime })
      officeBlobUrl = URL.createObjectURL(blob)
      officeSrc.value = officeBlobUrl
      loading.value = true
    }
  } catch (e) {
    ElMessage.error('加载失败: ' + e)
    loading.value = false
  }
}

const handleRendered = () => { loading.value = false }
const handleKeydown = (e) => { if (e.key === 'Escape') { e.preventDefault(); close() } }

function cleanup() {
  if (pdfUrl.value) { URL.revokeObjectURL(pdfUrl.value); pdfUrl.value = null }
  if (officeBlobUrl) { URL.revokeObjectURL(officeBlobUrl); officeBlobUrl = null }
  officeSrc.value = ''
}

onMounted(() => {
  if (props.filePath) loadFile()
  document.addEventListener('keydown', handleKeydown)
})

onBeforeUnmount(() => {
  cleanup()
  document.removeEventListener('keydown', handleKeydown)
})
</script>

<style scoped>
.file-preview-overlay { position: fixed; inset: 0; z-index: 9999; display: flex; flex-direction: column; background: #fff; }
.zoom-bar {
  flex-shrink: 0; display: flex; align-items: center; justify-content: center;
  padding: 4px 12px; background: #d9d9d9; gap: 4px;
}
.zoom-bar .el-button { --el-button-bg-color: transparent; --el-button-border-color: transparent; }
.zoom-bar .el-button:hover { --el-button-hover-bg-color: rgba(0,0,0,0.06); --el-button-hover-border-color: transparent; }
.zoom-label { font-size: 12px; color: #606266; min-width: 36px; text-align: center; user-select: none; }
.preview-container { flex: 1; overflow: hidden; display: flex; flex-direction: column; }
.unsupported-tip { display: flex; flex-direction: column; align-items: center; justify-content: center; flex: 1; color: #909399; }
.unsupported-tip p { margin: 16px 0 24px 0; font-size: 16px; }
.pdf-iframe { width: 100%; height: 100%; flex: 1; border: none; }
.excel-preview { width: 100%; height: 100%; flex: 1; }
.office-viewport { flex: 1; overflow: hidden; position: relative; display: flex; }
.office-scroll-area { flex: 1; overflow: auto; }
.office-zoom-layer { min-width: 100%; min-height: 100%; overflow: visible; }
.text-preview {
  flex: 1; overflow: auto; padding: 16px 24px; margin: 0; font-size: 15px; line-height: 1.6;
  background: #f8f9fa; white-space: pre-wrap; word-break: break-all;
  font-family: 'Cascadia Code', 'Fira Code', 'JetBrains Mono', 'Consolas', monospace;
}
</style>
