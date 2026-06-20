<template>
  <div class="ip-overlay" @click.self="onOverlayClick">
    <div ref="ipCanvasWrap" class="ip-canvas">
      <canvas ref="canvasEl"
        @mousedown="onCanvasMouseDown"
        @mousemove="onCanvasMouseMove"
        @mouseup="onCanvasMouseUp"
        @wheel.prevent="onCanvasWheel" />
      <div v-if="showSelectionOverlay" class="ip-selection-overlay" :style="selectionOverlayStyle">
        <div class="ip-selection-fill" />
        <template v-if="selectedAnnot && selectedAnnot.type !== 'arrow'">
          <div v-for="d in resizeDirs" :key="d"
            class="ip-resize-handle" :class="'ip-handle-' + d"
            @mousedown.stop="onResizeStart($event, d)" />
        </template>
        <template v-else-if="selectedAnnot && selectedAnnot.type === 'arrow'">
          <div class="ip-arrow-endpoint" :style="arrowHandleStyle('start')"
            @mousedown.stop="onResizeStart($event, 'start')" />
          <div class="ip-arrow-endpoint" :style="arrowHandleStyle('end')"
            @mousedown.stop="onResizeStart($event, 'end')" />
          <div class="ip-arrow-move" :style="arrowHandleStyle('move')"
            @mousedown.stop="onResizeStart($event, 'move')" />
        </template>
      </div>
      <div v-if="editingText" class="ip-text-group"
        :style="{ left: textInputPos.x + 'px', top: textInputPos.y + 'px' }">
        <input ref="textInputEl" class="ip-text-input" v-model="textInputValue"
          :style="{ fontSize: textInputFS + 'px', color: textInputColor }"
          @keydown="handleTextKeydown" @click.stop @mousedown.stop @mouseup.stop />
        <el-button circle size="small" type="primary" @click.stop="commitText" class="ip-text-confirm-btn">
          <el-icon><Check /></el-icon>
        </el-button>
      </div>
      <div v-if="hasPendingCrop" class="ip-crop-overlay" :style="cropOverlayStyle"
        @mousedown.stop="onCropOverlayMouseDown">
        <div class="ip-crop-fill" />
        <div class="ip-crop-center" />
        <div v-for="d in resizeDirs" :key="d"
          class="ip-resize-handle" :class="'ip-handle-' + d"
          @mousedown.stop="onCropResizeStart($event, d)" />
      </div>
      <div v-if="hasPendingCrop" class="ip-crop-actions" :style="cropActionsStyle">
        <span class="ip-crop-dim-label">宽</span>
        <input class="ip-crop-dim-input" v-model.number="cropW" type="number" min="1"
          @click.stop @mousedown.stop @keydown.stop />
        <span class="ip-crop-dim-label">高</span>
        <input class="ip-crop-dim-input" v-model.number="cropH" type="number" min="1"
          @click.stop @mousedown.stop @keydown.stop />
        <div class="ip-crop-btn ip-crop-btn-confirm" @click.stop="confirmCrop" @mousedown.stop title="确认裁切">✓</div>
        <div class="ip-crop-btn ip-crop-btn-cancel" @click.stop="cancelCrop" @mousedown.stop title="取消">✕</div>
      </div>
    </div>
    <div class="ip-info">
      <span class="ip-info-name">{{ fileName }}</span>
      <span v-if="imageResolution" class="ip-info-divider">|</span>
      <span v-if="imageResolution" class="ip-info-res">{{ imageResolution }}</span>
    </div>
    <div v-if="editing && propertyTool" class="ip-property-bar" @click.stop>
      <template v-if="propertyTool === 'rect'">
        <el-color-picker v-model="styleColor" size="small" :predefine="editorPreColors" popper-class="ip-editor-popper" />
        <span class="ip-prop-label">粗</span>
        <el-select v-model="styleStroke" size="small" style="width:52px" @click.stop popper-class="ip-editor-popper">
          <el-option v-for="s in [1,2,3,4,5]" :key="s" :value="s" :label="String(s)" />
        </el-select>
      </template>
      <template v-if="propertyTool === 'arrow'">
        <el-color-picker v-model="styleColor" size="small" :predefine="editorPreColors" popper-class="ip-editor-popper" />
        <span class="ip-prop-label">粗</span>
        <el-select v-model="styleStroke" size="small" style="width:52px" @click.stop popper-class="ip-editor-popper">
          <el-option v-for="s in [1,2,3,4,5]" :key="s" :value="s" :label="String(s)" />
        </el-select>
        <span class="ip-prop-label" style="margin-left:2px">样式</span>
        <div class="ip-arrow-styles" @click.stop>
          <button v-for="opt in arrowStyleOpts" :key="opt.value"
            :class="['ip-arrow-style-btn', { active: headStyle === opt.value }]"
            @click.stop="headStyle = opt.value" v-html="opt.icon" />
        </div>
      </template>
      <template v-if="propertyTool === 'text'">
        <el-color-picker v-model="styleColor" size="small" :predefine="editorPreColors" popper-class="ip-editor-popper" />
        <span class="ip-prop-label">字号</span>
        <el-select v-model="styleFontSize" size="small" style="width:64px" @click.stop popper-class="ip-editor-popper">
          <el-option v-for="s in [14,16,18,20,24,28,32,36,48]" :key="s" :value="s" :label="`${s}`" />
        </el-select>
      </template>
      <template v-if="propertyTool === 'mosaic'">
        <span class="ip-prop-label">块</span>
        <el-select v-model="mosaicBlockSize" size="small" style="width:64px" @click.stop popper-class="ip-editor-popper">
          <el-option v-for="s in [4,6,8,10,12,16,20,24,32]" :key="s" :value="s" :label="`${s}`" />
        </el-select>
      </template>
      <template v-if="propertyTool === 'crop'">
        <span class="ip-prop-label">裁切模式</span>
        <el-radio-group v-model="cropMode" size="small" @click.stop>
          <el-radio-button value="free">自由</el-radio-button>
          <el-radio-button value="original">原比例</el-radio-button>
          <el-radio-button value="fixed">固定比例</el-radio-button>
        </el-radio-group>
        <template v-if="cropMode === 'fixed'">
          <span class="ip-prop-label">宽</span>
          <el-input-number v-model="ratioW" :min="1" :max="100" size="small" style="width:60px" controls-position="right" @click.stop />
          <span class="ip-prop-label">:</span>
          <el-input-number v-model="ratioH" :min="1" :max="100" size="small" style="width:60px" controls-position="right" @click.stop />
          <span class="ip-prop-label">高</span>
        </template>
      </template>
    </div>
    <div class="ip-toolbar" @click.stop>
      <template v-if="!editing">
        <el-tooltip content="缩小" placement="top"><el-button circle @click.stop="ipZoomIn"><el-icon><ZoomIn /></el-icon></el-button></el-tooltip>
        <span class="ip-scale">{{ Math.round(zoomLevel * 100) }}%</span>
        <el-tooltip content="放大" placement="top"><el-button circle @click.stop="ipZoomOut"><el-icon><ZoomOut /></el-icon></el-button></el-tooltip>
        <el-tooltip content="适应/1:1" placement="top"><el-button circle @click.stop="ipReset">
          <svg v-if="Math.abs(zoomLevel - 1) > 0.001" viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="m14 10 7-7"/><path d="M20 10h-6V4"/><path d="m3 21 7-7"/><path d="M4 14h6v6"/>
          </svg>
          <svg v-else viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M15 3h6v6"/><path d="m21 3-7 7"/><path d="m3 21 7-7"/><path d="M9 21H3v-6"/>
          </svg>
        </el-button></el-tooltip>
        <span class="ip-toolbar-divider" />
        <el-tooltip content="左旋90°" placement="top"><el-button circle @click.stop="rotate(-90)"><el-icon><RefreshLeft /></el-icon></el-button></el-tooltip>
        <el-tooltip content="右旋90°" placement="top"><el-button circle @click.stop="rotate(90)"><el-icon><RefreshRight /></el-icon></el-button></el-tooltip>
        <span class="ip-toolbar-divider" />
        <el-tooltip content="编辑" placement="top"><el-button circle @click.stop="startEdit"><el-icon><Edit /></el-icon></el-button></el-tooltip>
        <span class="ip-toolbar-divider" />
        <el-button circle @click.stop="close" title="关闭"><el-icon><Close /></el-icon></el-button>
      </template>
      <template v-else>
        <el-tooltip content="选择/平移" placement="top">
          <el-button :type="tool === 'select' ? 'primary' : ''" circle @click.stop="setTool('select')"><el-icon><Pointer /></el-icon></el-button>
        </el-tooltip>
        <el-tooltip content="文字" placement="top">
          <el-button :type="tool === 'text' ? 'primary' : ''" circle @click.stop="setTool('text')"><el-icon><EditPen /></el-icon></el-button>
        </el-tooltip>
        <el-tooltip content="箭头" placement="top">
          <el-button :type="tool === 'arrow' ? 'primary' : ''" circle @click.stop="setTool('arrow')"><svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round"><line x1="5" y1="19" x2="19" y2="5"/><polyline points="12 5 19 5 19 12"/></svg></el-button>
        </el-tooltip>
        <el-tooltip content="矩形" placement="top">
          <el-button :type="tool === 'rect' ? 'primary' : ''" circle @click.stop="setTool('rect')"><el-icon><FullScreen /></el-icon></el-button>
        </el-tooltip>
        <el-tooltip content="马赛克" placement="top">
          <el-button :type="tool === 'mosaic' ? 'primary' : ''" circle @click.stop="setTool('mosaic')"><svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="7" height="7"/><rect x="14" y="3" width="7" height="7"/><rect x="3" y="14" width="7" height="7"/><rect x="14" y="14" width="7" height="7"/></svg></el-button>
        </el-tooltip>
        <span class="ip-toolbar-divider" />
        <el-tooltip content="裁切" placement="top">
          <el-button :type="tool === 'crop' ? 'primary' : ''" circle @click.stop="setTool('crop')"><svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="M6 2v14a2 2 0 0 0 2 2h14"/><path d="M2 6h14a2 2 0 0 1 2 2v14"/></svg></el-button>
        </el-tooltip>
        <el-tooltip content="左旋90°" placement="top"><el-button circle @click.stop="rotate(-90)"><el-icon><RefreshLeft /></el-icon></el-button></el-tooltip>
        <el-tooltip content="右旋90°" placement="top"><el-button circle @click.stop="rotate(90)"><el-icon><RefreshRight /></el-icon></el-button></el-tooltip>
        <span class="ip-toolbar-divider" />
        <el-tooltip content="撤销" placement="top"><el-button circle @click.stop="editorUndo" :disabled="!canUndo"><el-icon><Back /></el-icon></el-button></el-tooltip>
        <el-tooltip content="重做" placement="top"><el-button circle @click.stop="editorRedo" :disabled="!canRedo"><el-icon><Right /></el-icon></el-button></el-tooltip>
        <el-tooltip content="删除" placement="top"><el-button circle @click.stop="deleteSelected" :disabled="!hasSelection"><el-icon><Delete /></el-icon></el-button></el-tooltip>
        <span class="ip-toolbar-divider" />
        <el-tooltip content="缩小" placement="top"><el-button circle @click.stop="editorZoomIn" :disabled="zoomLevel <= 0.1">−</el-button></el-tooltip>
        <span class="ip-scale">{{ Math.round(zoomLevel * 100) }}%</span>
        <el-tooltip content="放大" placement="top"><el-button circle @click.stop="editorZoomOut" :disabled="zoomLevel >= 10">+</el-button></el-tooltip>
        <el-tooltip content="适应/1:1" placement="top"><el-button circle @click.stop="ipReset">
          <svg v-if="Math.abs(zoomLevel - 1) > 0.001" viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="m14 10 7-7"/><path d="M20 10h-6V4"/><path d="m3 21 7-7"/><path d="M4 14h6v6"/>
          </svg>
          <svg v-else viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M15 3h6v6"/><path d="m21 3-7 7"/><path d="m3 21 7-7"/><path d="M9 21H3v-6"/>
          </svg>
        </el-button></el-tooltip>
        <span class="ip-toolbar-divider" />
        <el-tooltip content="保存" placement="top"><el-button type="primary" size="small" @click.stop="promptSave" style="border-radius:20px;padding:0 16px">保存</el-button></el-tooltip>
        <el-tooltip content="关闭" placement="top"><el-button circle @click.stop="stopEdit"><el-icon><Close /></el-icon></el-button></el-tooltip>
      </template>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, watch, nextTick, onMounted, onBeforeUnmount } from 'vue'
import { ZoomIn, ZoomOut, RefreshRight, RefreshLeft, FullScreen, Edit, Pointer, EditPen, Delete, Back, Right, Check, Close } from '@element-plus/icons-vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { invoke } from '@tauri-apps/api/core'

const props = defineProps({
  filePath: { type: String, default: '' },
  fileExt: { type: String, default: '' },
  fileName: { type: String, default: '' },
})
const emit = defineEmits(['close'])
const close = () => emit('close')

const ext = computed(() => (props.fileExt || '').toLowerCase().replace(/^\./, ''))

const canvasEl = ref(null)
const ipCanvasWrap = ref(null)
let ctx = null
let imgEl = null
let imageNaturalWidth = 0
let imageNaturalHeight = 0
let canvasResizeObserver = null
let oldWrapW = 0
let oldWrapH = 0

const zoomLevel = ref(1)
let fitZoom = 1
let panX = 0
let panY = 0

const editing = ref(false)
const tool = ref('select')
const styleColor = ref('#ff0000')
const styleStroke = ref(2)
const styleFontSize = ref(24)
const mosaicBlockSize = ref(8)
const headStyle = ref('wedge')
const arrowStyleOpts = [
  { value: 'wedge', icon: '<svg width="14" height="12" viewBox="0 0 20 12"><path d="M1 6h12M13 1l6 5-6 5" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"/><path d="M13 9V3l6 3z" fill="currentColor"/></svg>' },
  { value: 'triangle', icon: '<svg width="14" height="12" viewBox="0 0 20 12"><path d="M1 6h12M13 1l6 5-6 5" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"/><path d="M13 1l6 5-6 5z" fill="currentColor"/></svg>' },
  { value: 'open', icon: '<svg width="14" height="12" viewBox="0 0 20 12"><path d="M1 6h12M13 1l6 5-6 5" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"/></svg>' },
  { value: 'line', icon: '<svg width="14" height="12" viewBox="0 0 20 12"><path d="M1 6h14" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round"/></svg>' },
]
const resizeDirs = ['nw', 'n', 'ne', 'e', 'se', 's', 'sw', 'w']
const maxHistory = 30
const undoStack = ref([])
const redoStack = ref([])
const annotations = ref([])
let savedAnnotations = '[]'
let pendingPixelChanges = false

let drawStart = null
let drawPreviewAnnot = null
let isPanning = false
let panStartMouse = { x: 0, y: 0 }
let panStartPan = { x: 0, y: 0 }
const selectedIndex = ref(-1)
let dragOffset = { x: 0, y: 0 }
let isDragging = false
let isResizing = false
let resizeDir = ''
let resizeStartImage = { x: 0, y: 0 }
let resizeSnapshot = null

const editingText = ref(false)
const textInputValue = ref('')
const textInputPos = ref({ x: 0, y: 0 })
const textInputFS = ref(24)
const textInputColor = ref('#ff0000')
const textInputEl = ref(null)
let textEditAnnotIndex = -1
let pendingTextAnnot = null

let cropStart = null
const cropMode = ref('free')
const ratioW = ref(1)
const ratioH = ref(1)
const hasPendingCrop = ref(false)
const cropTick = ref(0)
const cropW = ref(0)
const cropH = ref(0)
let cropDragging = false
let cropDragStart = { x: 0, y: 0 }
let cropDragOrig = { x: 0, y: 0 }
let cropResizing = false
let cropResizeDir = ''
let cropResizeStart = { x: 0, y: 0 }
let cropRectSnapshot = null

const editorPreColors = ['#ff0000', '#ff8c00', '#ffd700', '#00cc00', '#0099ff', '#0033ff', '#9933ff', '#ffffff', '#cccccc', '#666666']
const canUndo = computed(() => undoStack.value.length > 0)
const canRedo = computed(() => redoStack.value.length > 0)
const hasSelection = computed(() => selectedIndex.value >= 0 && selectedIndex.value < annotations.value.length && tool.value !== 'crop')
const selectedAnnot = computed(() => {
  if (!editing.value || tool.value !== 'select') return null
  if (selectedIndex.value < 0 || selectedIndex.value >= annotations.value.length) return null
  return annotations.value[selectedIndex.value]
})
const showSelectionOverlay = computed(() => selectedAnnot.value !== null)
const imageResolution = computed(() => {
  zoomLevel.value
  if (!imageNaturalWidth || !imageNaturalHeight) return ''
  return `${imageNaturalWidth} × ${imageNaturalHeight}`
})
const propertyTool = computed(() => {
  if (!editing.value) return null
  if (tool.value === 'select') {
    const a = selectedAnnot.value
    if (a && ['rect', 'arrow', 'text', 'mosaic'].includes(a.type)) return a.type
    return null
  }
  if (['rect', 'arrow', 'text', 'mosaic', 'crop'].includes(tool.value)) return tool.value
  return null
})

function getAnnotationBBox(a) {
  if (a.type === 'rect' || a.type === 'mosaic') {
    return { x: a.x, y: a.y, w: a.w, h: a.h }
  } else if (a.type === 'arrow') {
    const x = Math.min(a.x, a.x2); const y = Math.min(a.y, a.y2)
    const w = Math.abs(a.x2 - a.x); const h = Math.abs(a.y2 - a.y)
    return { x, y, w, h }
  } else if (a.type === 'text') {
    let tw = 60
    if (ctx) {
      ctx.save(); ctx.font = `${a.fontSize || 24}px sans-serif`
      tw = Math.max(ctx.measureText(a.text || '').width, 10); ctx.restore()
    }
    return { x: a.x, y: a.y, w: tw, h: (a.fontSize || 24) }
  }
  return { x: 0, y: 0, w: 0, h: 0 }
}

const selectionOverlayStyle = computed(() => {
  const a = selectedAnnot.value
  if (!a) return { display: 'none' }
  const bbox = getAnnotationBBox(a)
  return {
    left: (bbox.x * zoomLevel.value + panX) + 'px',
    top: (bbox.y * zoomLevel.value + panY) + 'px',
    width: (bbox.w * zoomLevel.value) + 'px',
    height: (bbox.h * zoomLevel.value) + 'px',
  }
})

const cropActionsStyle = computed(() => {
  cropTick.value
  if (!drawPreviewAnnot || !drawPreviewAnnot.isCrop) return { display: 'none' }
  const { x, y, w, h } = drawPreviewAnnot
  const sx = x * zoomLevel.value + panX
  const sy = (y + h) * zoomLevel.value + panY + 6
  return {
    position: 'absolute',
    left: sx + (w * zoomLevel.value) / 2 + 'px',
    top: sy + 'px',
    transform: 'translateX(-50%)',
  }
})

const cropOverlayStyle = computed(() => {
  cropTick.value
  if (!hasPendingCrop.value || !drawPreviewAnnot || !drawPreviewAnnot.isCrop) return { display: 'none' }
  const { x, y, w, h } = drawPreviewAnnot
  return {
    position: 'absolute',
    left: (x * zoomLevel.value + panX) + 'px',
    top: (y * zoomLevel.value + panY) + 'px',
    width: (w * zoomLevel.value) + 'px',
    height: (h * zoomLevel.value) + 'px',
  }
})

function arrowHandleStyle(which) {
  const a = selectedAnnot.value
  if (!a || a.type !== 'arrow') return {}
  const bbox = getAnnotationBBox(a)
  let ix, iy
  if (which === 'start') { ix = a.x; iy = a.y }
  else if (which === 'end') { ix = a.x2; iy = a.y2 }
  else { ix = (a.x + a.x2) / 2; iy = (a.y + a.y2) / 2 }
  const rx = ((ix - bbox.x) / bbox.w) * 100
  const ry = ((iy - bbox.y) / bbox.h) * 100
  return { left: `calc(${rx}% - 7px)`, top: `calc(${ry}% - 7px)` }
}

function screenToImage(sx, sy) { return { x: (sx - panX) / zoomLevel.value, y: (sy - panY) / zoomLevel.value } }
function imageToScreen(ix, iy) { return { x: ix * zoomLevel.value + panX, y: iy * zoomLevel.value + panY } }

function render() {
  if (!ctx || !imgEl) return
  const wrap = ipCanvasWrap.value
  if (!wrap) return
  const wr = wrap.getBoundingClientRect()
  ctx.clearRect(0, 0, wr.width, wr.height)
  ctx.save()
  ctx.translate(panX, panY)
  ctx.scale(zoomLevel.value, zoomLevel.value)
  ctx.drawImage(imgEl, 0, 0, imageNaturalWidth, imageNaturalHeight)
  for (let i = 0; i < annotations.value.length; i++) drawAnnotation(annotations.value[i], i)
  if (drawPreviewAnnot) drawAnnotation(drawPreviewAnnot, -1)
  ctx.restore()
}

function drawAnnotation(a, idx) {
  if (a && a.isCrop) {
    ctx.save(); ctx.setLineDash([6 / zoomLevel.value, 3 / zoomLevel.value])
    ctx.strokeStyle = '#409eff'; ctx.fillStyle = 'rgba(64, 158, 255, 0.15)'; ctx.lineWidth = 1.5
    ctx.fillRect(a.x, a.y, a.w, a.h); ctx.strokeRect(a.x, a.y, a.w, a.h)
    const fs = Math.max(11, 12 / zoomLevel.value)
    ctx.setLineDash([]); ctx.font = `${fs}px sans-serif`
    ctx.fillStyle = '#409eff'
    const label = `${Math.round(a.w)} × ${Math.round(a.h)}`
    ctx.fillText(label, a.x + 4, a.y + fs + 2)
    ctx.restore(); return
  }
  ctx.strokeStyle = a.color || '#ff0000'
  ctx.fillStyle = a.color || '#ff0000'
  ctx.lineWidth = (a.strokeWidth || 2)
  ctx.lineCap = 'round'; ctx.lineJoin = 'round'

  if (a.type === 'rect') {
    ctx.strokeRect(a.x, a.y, a.w, a.h)
  } else if (a.type === 'arrow') {
    const dx = a.x2 - a.x; const dy = a.y2 - a.y; const len = Math.sqrt(dx * dx + dy * dy)
    if (len < 1) return; const angle = Math.atan2(dy, dx)
    const hs = a.headStyle || 'wedge'
    if (hs === 'wedge') {
      const s = Math.min(30, len * 0.4)
      const h = s * Math.sqrt(3) / 2
      const wt = s * 2 / 5
      const ws = 1
      const nx = -Math.sin(angle); const ny = Math.cos(angle)
      const bcx = a.x2 - h * Math.cos(angle)
      const bcy = a.y2 - h * Math.sin(angle)
      ctx.beginPath()
      ctx.moveTo(a.x + nx * ws / 2, a.y + ny * ws / 2)
      ctx.lineTo(bcx + nx * wt / 2, bcy + ny * wt / 2)
      ctx.lineTo(bcx + nx * s / 2, bcy + ny * s / 2)
      ctx.lineTo(a.x2, a.y2)
      ctx.lineTo(bcx - nx * s / 2, bcy - ny * s / 2)
      ctx.lineTo(bcx - nx * wt / 2, bcy - ny * wt / 2)
      ctx.lineTo(a.x - nx * ws / 2, a.y - ny * ws / 2)
      ctx.closePath(); ctx.fill()
    } else if (hs === 'triangle') {
      ctx.beginPath(); ctx.moveTo(a.x, a.y); ctx.lineTo(a.x2, a.y2); ctx.stroke()
      const hl = Math.min(14, len * 0.3)
      ctx.beginPath(); ctx.moveTo(a.x2, a.y2)
      ctx.lineTo(a.x2 - hl * Math.cos(angle - Math.PI / 6), a.y2 - hl * Math.sin(angle - Math.PI / 6))
      ctx.lineTo(a.x2 - hl * Math.cos(angle + Math.PI / 6), a.y2 - hl * Math.sin(angle + Math.PI / 6))
      ctx.closePath(); ctx.fill()
    } else if (hs === 'open') {
      ctx.beginPath(); ctx.moveTo(a.x, a.y); ctx.lineTo(a.x2, a.y2); ctx.stroke()
      const hl = Math.min(14, len * 0.3)
      ctx.beginPath(); ctx.moveTo(a.x2, a.y2)
      ctx.lineTo(a.x2 - hl * Math.cos(angle - Math.PI / 6), a.y2 - hl * Math.sin(angle - Math.PI / 6))
      ctx.moveTo(a.x2, a.y2)
      ctx.lineTo(a.x2 - hl * Math.cos(angle + Math.PI / 6), a.y2 - hl * Math.sin(angle + Math.PI / 6))
      ctx.stroke()
    } else if (hs === 'line') {
      ctx.beginPath(); ctx.moveTo(a.x, a.y); ctx.lineTo(a.x2, a.y2); ctx.stroke()
    }
  } else if (a.type === 'text') {
    ctx.font = `${a.fontSize || 24}px sans-serif`; ctx.textBaseline = 'top'
    ctx.fillText(a.text || '', a.x, a.y)
  } else if (a.type === 'mosaic' && a.w > 0 && a.h > 0) {
    const bs = a.bs || 8
    ctx.save(); ctx.beginPath(); ctx.rect(a.x, a.y, a.w, a.h); ctx.clip()
    if (imgEl) {
      const cols = Math.ceil(a.w / bs); const rows = Math.ceil(a.h / bs)
      if (cols > 0 && rows > 0) {
        const oc = document.createElement('canvas'); oc.width = cols; oc.height = rows
        const octx = oc.getContext('2d'); octx.imageSmoothingEnabled = false
        octx.drawImage(imgEl, a.x, a.y, a.w, a.h, 0, 0, cols, rows)
        ctx.imageSmoothingEnabled = false; ctx.drawImage(oc, a.x, a.y, a.w, a.h)
      }
    } else {
      ctx.fillStyle = 'rgba(200,200,200,0.7)'; ctx.fillRect(a.x, a.y, a.w, a.h)
    }
    ctx.restore()
  }
}

function zoomToFit() {
  const wrap = ipCanvasWrap.value; if (!wrap) return
  const wr = wrap.getBoundingClientRect(); const cw = wr.width, ch = wr.height
  if (!cw || !ch || !imageNaturalWidth || !imageNaturalHeight) return
  const z = Math.min(cw / imageNaturalWidth, ch / imageNaturalHeight, 1)
  zoomLevel.value = z; fitZoom = z
  panX = (cw - imageNaturalWidth * z) / 2; panY = (ch - imageNaturalHeight * z) / 2
  render()
}

function zoomToPoint(newZ, cx, cy) {
  const iz = zoomLevel.value; const ox = (cx - panX) / iz, oy = (cy - panY) / iz
  zoomLevel.value = Math.max(0.1, Math.min(20, newZ))
  panX = cx - ox * zoomLevel.value; panY = cy - oy * zoomLevel.value; render()
}

function clampPan() {
  const wrap = ipCanvasWrap.value; if (!wrap || !imageNaturalWidth || !imageNaturalHeight) return
  const wr = wrap.getBoundingClientRect(); const vw = wr.width, vh = wr.height; const z = zoomLevel.value
  const iw = imageNaturalWidth * z, ih = imageNaturalHeight * z
  panX = Math.max(Math.min(0, vw - iw), Math.min(panX, Math.max(0, vw - iw)))
  panY = Math.max(Math.min(0, vh - ih), Math.min(panY, Math.max(0, vh - ih)))
}

function setupCanvas() {
  const wrap = ipCanvasWrap.value; const el = canvasEl.value; if (!wrap || !el) return
  const wr = wrap.getBoundingClientRect(); const dpr = window.devicePixelRatio || 1
  el.width = wr.width * dpr; el.height = wr.height * dpr; el.style.width = wr.width + 'px'; el.style.height = wr.height + 'px'
  ctx = el.getContext('2d'); ctx.scale(dpr, dpr); oldWrapW = wr.width; oldWrapH = wr.height
}

function updateCursor() {
  if (!canvasEl.value) return
  canvasEl.value.style.cursor = editing.value ? (tool.value === 'select' ? 'default' : tool.value === 'text' ? 'text' : 'crosshair') : 'default'
}

async function initCanvas() {
  setupCanvas()
  try {
    const data = await invoke('read_file_binary', { path: props.filePath })
    const blob = new Blob([new Uint8Array(data)])
    const url = URL.createObjectURL(blob)
    const el = await new Promise((resolve, reject) => {
      const img = new Image()
      img.onload = () => resolve(img)
      img.onerror = () => reject(new Error('图片加载失败'))
      img.src = url
    })
    URL.revokeObjectURL(url)
    imgEl = el; imageNaturalWidth = el.naturalWidth; imageNaturalHeight = el.naturalHeight
    zoomToFit()
    await loadAnnotations()
  } catch (err) {
    ElMessage.error('图片加载失败')
    return
  }
  setupCanvasResizeObserver(); updateCursor()
}

function setupCanvasResizeObserver() {
  const wrap = ipCanvasWrap.value; if (!wrap) return
  canvasResizeObserver = new ResizeObserver(([entry]) => {
    if (!ctx) return; const { width, height } = entry.contentRect
    if (width > 0 && height > 0) {
      const dpr = window.devicePixelRatio || 1; const el = canvasEl.value
      el.width = width * dpr; el.height = height * dpr; ctx.setTransform(dpr, 0, 0, dpr, 0, 0)
      if (!editing.value) { zoomToFit() } else { panX *= width / oldWrapW; panY *= height / oldWrapH; oldWrapW = width; oldWrapH = height; render() }
    }
  })
  canvasResizeObserver.observe(wrap)
}

function disposeCanvas() {
  if (canvasResizeObserver) { canvasResizeObserver.disconnect(); canvasResizeObserver = null }
  if (isResizing) { document.removeEventListener('mousemove', onResizeMove); document.removeEventListener('mouseup', onResizeEnd) }
  ctx = null; imgEl = null; pendingPixelChanges = false; editing.value = false; tool.value = 'select'
  undoStack.value = []; redoStack.value = []; annotations.value = []; selectedIndex.value = -1
  zoomLevel.value = 1; fitZoom = 1; panX = 0; panY = 0; drawStart = null; drawPreviewAnnot = null
  cropStart = null; isPanning = false; editingText.value = false; isResizing = false; resizeDir = ''; resizeSnapshot = null
}

function ipZoomIn() { const wrap = ipCanvasWrap.value; if (!wrap) return; const wr = wrap.getBoundingClientRect(); zoomToPoint(zoomLevel.value * 1.25, wr.width / 2, wr.height / 2) }
function ipZoomOut() { const wrap = ipCanvasWrap.value; if (!wrap) return; const wr = wrap.getBoundingClientRect(); zoomToPoint(zoomLevel.value / 1.25, wr.width / 2, wr.height / 2) }
function ipReset() {
  if (Math.abs(zoomLevel.value - 1) < 0.001) { zoomToFit() }
  else {
    const wrap = ipCanvasWrap.value; if (!wrap) return; const wr = wrap.getBoundingClientRect()
    zoomLevel.value = 1; panX = (wr.width - imageNaturalWidth) / 2; panY = (wr.height - imageNaturalHeight) / 2; render(); updateCursor()
  }
}

function startEdit() {
  editing.value = true
  render()
}

async function stopEdit() {
  const current = JSON.stringify(annotations.value)
  if (current !== savedAnnotations || pendingPixelChanges) {
    try {
      await ElMessageBox.confirm('存在未保存的修改，是否放弃？', '提示', { confirmButtonText: '放弃', cancelButtonText: '取消', type: 'warning', zIndex: 10002 })
      doStopEdit()
    } catch {}
    return
  }
  doStopEdit()
}

function doStopEdit() {
  if (pendingPixelChanges) { pendingPixelChanges = false }
  editing.value = false; tool.value = 'select'
  try { annotations.value = JSON.parse(savedAnnotations) } catch { annotations.value = [] }
  selectedIndex.value = -1; drawStart = null; drawPreviewAnnot = null; cropStart = null; undoStack.value = []; redoStack.value = []
  render(); updateCursor()
}

async function loadAnnotations() {
  try {
    const data = await invoke('load_annotations', { path: props.filePath })
    const parsed = JSON.parse(data)
    annotations.value = Array.isArray(parsed) ? parsed : []
    savedAnnotations = JSON.stringify(annotations.value); pushHistory(); render()
  } catch (e) { annotations.value = []; savedAnnotations = '[]'; render() }
}

async function promptSave() {
  try {
    await ElMessageBox.confirm('确定要保存当前修改吗？', '保存确认', { confirmButtonText: '保存', cancelButtonText: '取消', type: 'info', zIndex: 10002 })
  } catch { return }
  try {
    await ElMessageBox.confirm('保存方式', '请选择', {
      confirmButtonText: '覆盖原文件', cancelButtonText: '另存为新文件', type: 'info', zIndex: 10002,
    })
    await saveToFile(props.filePath)
  } catch {
    // save as
    const p = props.filePath
    const dot = p.lastIndexOf('.')
    const newPath = dot >= 0 ? p.substring(0, dot) + '_edited' + p.substring(dot) : p + '_edited'
    await saveToFile(newPath)
    ElMessage.success('已另存为: ' + newPath.split('\\').pop() || newPath.split('/').pop())
  }
}

async function saveToFile(outputPath) {
  const loadingMsg = ElMessage({ message: '正在保存...', duration: 0 })
  try {
    if (pendingPixelChanges) {
      const blob = await getCurrentImageBlob()
      const buf = await blob.arrayBuffer()
      await invoke('write_file', { path: outputPath, data: Array.from(new Uint8Array(buf)) })
    }
    await invoke('save_annotations', { path: props.filePath, data: JSON.stringify(annotations.value) })
    pendingPixelChanges = false; savedAnnotations = JSON.stringify(annotations.value)
    loadingMsg.close(); ElMessage.success('保存成功'); close()
  } catch (e) {
    loadingMsg.close(); ElMessage.error('保存失败: ' + e)
  }
}

function deleteSelected() {
  if (selectedIndex.value < 0 || selectedIndex.value >= annotations.value.length) return
  pushHistory(); annotations.value.splice(selectedIndex.value, 1); selectedIndex.value = -1; render()
}

function pushHistory() {
  undoStack.value.push(JSON.stringify(annotations.value))
  if (undoStack.value.length > maxHistory) undoStack.value.shift()
  redoStack.value = []
}

function editorUndo() {
  if (undoStack.value.length === 0) return
  redoStack.value.push(JSON.stringify(annotations.value))
  annotations.value = JSON.parse(undoStack.value.pop()); selectedIndex.value = -1; render()
}

function editorRedo() {
  if (redoStack.value.length === 0) return
  undoStack.value.push(JSON.stringify(annotations.value))
  annotations.value = JSON.parse(redoStack.value.pop()); selectedIndex.value = -1; render()
}

function setTool(t) {
  if (tool.value === t) return
  if (tool.value === 'crop') cancelCrop()
  if (editingText.value) commitText()
  tool.value = t; selectedIndex.value = -1; updateCursor()
}

function editorZoomIn() { const wrap = ipCanvasWrap.value; if (!wrap) return; const wr = wrap.getBoundingClientRect(); zoomToPoint(zoomLevel.value * 1.25, wr.width / 2, wr.height / 2); updateCursor() }
function editorZoomOut() { const wrap = ipCanvasWrap.value; if (!wrap) return; const wr = wrap.getBoundingClientRect(); zoomToPoint(zoomLevel.value / 1.25, wr.width / 2, wr.height / 2); updateCursor() }

function startTextEdit(ix, iy, existingIdx) {
  if (editingText.value) commitText()
  textEditAnnotIndex = existingIdx
  if (existingIdx >= 0) {
    const a = annotations.value[existingIdx]
    textInputValue.value = a.text || ''; textInputFS.value = (a.fontSize || 24) * zoomLevel.value; textInputColor.value = a.color || '#ff0000'
    const sp = imageToScreen(a.x, a.y); textInputPos.value = { x: sp.x, y: sp.y }; pendingTextAnnot = null
  } else {
    textInputValue.value = ''; textInputFS.value = styleFontSize.value * zoomLevel.value; textInputColor.value = styleColor.value
    const sp = imageToScreen(ix, iy); textInputPos.value = { x: sp.x, y: sp.y }
    pendingTextAnnot = { type: 'text', x: ix, y: iy, text: '', fontSize: styleFontSize.value, color: styleColor.value, id: 'text_' + Date.now() }
  }
  editingText.value = true; nextTick(() => textInputEl.value?.focus())
}

function commitText() {
  if (!editingText.value) return; editingText.value = false; const text = textInputValue.value.trim()
  if (!text) {
    if (pendingTextAnnot) pendingTextAnnot = null
    if (textEditAnnotIndex >= 0) { annotations.value.splice(textEditAnnotIndex, 1); selectedIndex.value = -1; render() }
    return
  }
  pushHistory()
  if (textEditAnnotIndex >= 0 && textEditAnnotIndex < annotations.value.length) { annotations.value[textEditAnnotIndex].text = text }
  else if (pendingTextAnnot) { pendingTextAnnot.text = text; annotations.value.push({ ...pendingTextAnnot }); pendingTextAnnot = null }
  selectedIndex.value = -1; textEditAnnotIndex = -1; render()
}

function cancelText() {
  if (editingText.value) { editingText.value = false; if (pendingTextAnnot) pendingTextAnnot = null; if (textEditAnnotIndex >= 0) selectedIndex.value = textEditAnnotIndex; textEditAnnotIndex = -1; render() }
}

function handleTextKeydown(e) {
  if (e.key === "Enter") { e.preventDefault(); commitText() }
  else if (e.key === "Escape") { e.stopPropagation(); e.preventDefault(); cancelText() }
}

function syncStyleFromAnnot(a) {
  styleColor.value = a.color || '#ff0000'
  if (a.type === 'rect' || a.type === 'arrow') { styleStroke.value = a.strokeWidth || 2; if (a.type === 'arrow') headStyle.value = a.headStyle || 'wedge' }
  if (a.type === 'text') { styleFontSize.value = a.fontSize || 24 }
  if (a.type === 'mosaic') mosaicBlockSize.value = a.bs || 8
}

watch(styleColor, (val) => { const a = selectedAnnot.value; if (a && a.type !== 'mosaic') { a.color = val; render() } })
watch(styleStroke, (val) => { const a = selectedAnnot.value; if (a && (a.type === 'rect' || a.type === 'arrow')) { a.strokeWidth = val; render() } })
watch(styleFontSize, (val) => { const a = selectedAnnot.value; if (a && a.type === 'text') { a.fontSize = val; render() } })
watch(mosaicBlockSize, (val) => { const a = selectedAnnot.value; if (a && a.type === 'mosaic') { a.bs = val; render() } })
watch(headStyle, (val) => {
  const a = selectedAnnot.value
  if (a && a.type === 'arrow') { a.headStyle = val; render() }
})

async function rotate(deg) {
  if (editing.value && annotations.value.length) {
    try { await ElMessageBox.confirm('旋转后将清空已有标注，是否继续？', '旋转确认') } catch { return }
  }
  const radians = deg * Math.PI / 180; const cos = Math.abs(Math.cos(radians)); const sin = Math.abs(Math.sin(radians))
  const newW = Math.ceil(imageNaturalWidth * cos + imageNaturalHeight * sin); const newH = Math.ceil(imageNaturalWidth * sin + imageNaturalHeight * cos)
  const oc = document.createElement('canvas'); oc.width = newW; oc.height = newH; const octx = oc.getContext('2d')
  octx.translate(newW / 2, newH / 2); octx.rotate(radians); octx.drawImage(imgEl, -imageNaturalWidth / 2, -imageNaturalHeight / 2)
  const mimeType = mimeFromExt(ext.value); const quality = mimeType === 'image/jpeg' ? 0.92 : undefined
  const blob = await new Promise(resolve => oc.toBlob(resolve, mimeType, quality)); const url = URL.createObjectURL(blob)
  const el = new Image()
  el.onload = () => {
    imgEl = el; imageNaturalWidth = el.naturalWidth; imageNaturalHeight = el.naturalHeight
    annotations.value = []; savedAnnotations = '[]'; undoStack.value = []; redoStack.value = []; zoomToFit(); pendingPixelChanges = true; URL.revokeObjectURL(url)
  }
  el.onerror = () => { URL.revokeObjectURL(url); ElMessage.error('旋转失败') }; el.src = url
}

async function confirmCrop() {
  if (!drawPreviewAnnot || !drawPreviewAnnot.isCrop) return
  hasPendingCrop.value = false
  let { x, y, w, h } = drawPreviewAnnot; x = Math.max(0, Math.min(imageNaturalWidth, x)); y = Math.max(0, Math.min(imageNaturalHeight, y))
  w = Math.min(w, imageNaturalWidth - x); h = Math.min(h, imageNaturalHeight - y)
  drawPreviewAnnot = null; cropStart = null; render()
  if (w < 10 || h < 10) { ElMessage.warning('裁切区域太小'); return }
  const oc = document.createElement('canvas'); oc.width = Math.round(w); oc.height = Math.round(h); const octx = oc.getContext('2d')
  octx.drawImage(imgEl, Math.round(x), Math.round(y), Math.round(w), Math.round(h), 0, 0, Math.round(w), Math.round(h))
  const mimeType = mimeFromExt(ext.value); const quality = mimeType === 'image/jpeg' ? 0.92 : undefined
  const blob = await new Promise(resolve => oc.toBlob(resolve, mimeType, quality)); const url = URL.createObjectURL(blob)
  const el = new Image()
  el.onload = () => {
    imgEl = el; imageNaturalWidth = el.naturalWidth; imageNaturalHeight = el.naturalHeight; cancelCrop()
    annotations.value = []; savedAnnotations = '[]'; undoStack.value = []; redoStack.value = []; zoomToFit(); editing.value = true; tool.value = 'select'; pendingPixelChanges = true; URL.revokeObjectURL(url)
  }
  el.onerror = () => { URL.revokeObjectURL(url); ElMessage.error('裁切失败'); cancelCrop() }; el.src = url
}

function cancelCrop() { hasPendingCrop.value = false; drawPreviewAnnot = null; cropStart = null; render() }

function mimeFromExt(ext) { const map = { jpg: 'image/jpeg', jpeg: 'image/jpeg', png: 'image/png', gif: 'image/gif', webp: 'image/webp', bmp: 'image/bmp' }; return map[ext] || 'image/png' }

function getCurrentImageBlob() {
  const oc = document.createElement('canvas'); oc.width = imageNaturalWidth; oc.height = imageNaturalHeight; const octx = oc.getContext('2d')
  octx.drawImage(imgEl, 0, 0, imageNaturalWidth, imageNaturalHeight)
  const mimeType = mimeFromExt(ext.value); const quality = mimeType === 'image/jpeg' ? 0.92 : undefined
  return new Promise(resolve => oc.toBlob(resolve, mimeType, quality))
}

function getCropRatio() {
  if (cropMode.value === 'original' && imageNaturalWidth > 0 && imageNaturalHeight > 0) {
    return imageNaturalWidth / imageNaturalHeight
  }
  if (cropMode.value === 'fixed' && ratioW.value > 0 && ratioH.value > 0) {
    return ratioW.value / ratioH.value
  }
  return null
}

function applyRectResize(snap, dir, dx, dy) {
  const minSize = 10; let clampDx = dx, clampDy = dy
  if (dir.includes('w') && snap.w - dx < minSize) clampDx = snap.w - minSize
  if (dir.includes('e') && snap.w + dx < minSize) clampDx = minSize - snap.w
  if (dir.includes('n') && snap.h - dy < minSize) clampDy = snap.h - minSize
  if (dir.includes('s') && snap.h + dy < minSize) clampDy = minSize - snap.h
  if (dir.includes('w')) { snap.x += clampDx; snap.w -= clampDx }
  if (dir.includes('e')) { snap.w += clampDx }
  if (dir.includes('n')) { snap.y += clampDy; snap.h -= clampDy }
  if (dir.includes('s')) { snap.h += clampDy }
}

function onResizeStart(e, dir) {
  if (editingText.value) return; isResizing = true; resizeDir = dir
  const rect = canvasEl.value.getBoundingClientRect(); const sp = screenToImage(e.clientX - rect.left, e.clientY - rect.top)
  resizeStartImage = { x: sp.x, y: sp.y }; const a = annotations.value[selectedIndex.value]
  resizeSnapshot = JSON.parse(JSON.stringify(a)); pushHistory()
  document.addEventListener('mousemove', onResizeMove); document.addEventListener('mouseup', onResizeEnd)
}

function onResizeMove(e) {
  if (!isResizing || !resizeSnapshot) return
  const rect = canvasEl.value.getBoundingClientRect(); const sp = screenToImage(e.clientX - rect.left, e.clientY - rect.top)
  const dx = sp.x - resizeStartImage.x; const dy = sp.y - resizeStartImage.y; const snap = JSON.parse(JSON.stringify(resizeSnapshot))
  const a = annotations.value[selectedIndex.value]; if (!a) return
  if (a.type === 'rect' || a.type === 'mosaic') { applyRectResize(snap, resizeDir, dx, dy) }
  else if (a.type === 'arrow') {
    if (resizeDir === 'start') { snap.x += dx; snap.y += dy }
    else if (resizeDir === 'end') { snap.x2 += dx; snap.y2 += dy }
    else if (resizeDir === 'move') { snap.x += dx; snap.y += dy; snap.x2 += dx; snap.y2 += dy }
  }
  Object.assign(a, snap); render()
}

function onResizeEnd() {
  if (!isResizing) return; isResizing = false; resizeDir = ''; resizeSnapshot = null
  document.removeEventListener('mousemove', onResizeMove); document.removeEventListener('mouseup', onResizeEnd); render()
}

function onCropOverlayMouseDown(e) {
  if (!drawPreviewAnnot || !drawPreviewAnnot.isCrop) return
  const rect = canvasEl.value.getBoundingClientRect()
  const sp = screenToImage(e.clientX - rect.left, e.clientY - rect.top)
  cropDragging = true; cropDragStart = { x: sp.x, y: sp.y }
  cropDragOrig = { x: drawPreviewAnnot.x, y: drawPreviewAnnot.y }
  document.addEventListener('mousemove', onCropDragMove)
  document.addEventListener('mouseup', onCropDragEnd)
  e.preventDefault()
}

function onCropDragMove(e) {
  if (!cropDragging || !drawPreviewAnnot) return
  const rect = canvasEl.value.getBoundingClientRect()
  const sp = screenToImage(e.clientX - rect.left, e.clientY - rect.top)
  const dx = sp.x - cropDragStart.x; const dy = sp.y - cropDragStart.y
  let nx = cropDragOrig.x + dx; let ny = cropDragOrig.y + dy
  nx = Math.max(0, Math.min(imageNaturalWidth - drawPreviewAnnot.w, nx))
  ny = Math.max(0, Math.min(imageNaturalHeight - drawPreviewAnnot.h, ny))
  drawPreviewAnnot.x = nx; drawPreviewAnnot.y = ny
  cropTick.value++; render()
}

function onCropDragEnd() {
  if (!cropDragging) return; cropDragging = false
  document.removeEventListener('mousemove', onCropDragMove)
  document.removeEventListener('mouseup', onCropDragEnd)
}

function onCropResizeStart(e, dir) {
  if (!drawPreviewAnnot || !drawPreviewAnnot.isCrop) return
  cropResizing = true; cropResizeDir = dir
  const rect = canvasEl.value.getBoundingClientRect()
  const sp = screenToImage(e.clientX - rect.left, e.clientY - rect.top)
  cropResizeStart = { x: sp.x, y: sp.y }
  cropRectSnapshot = { ...drawPreviewAnnot }
  document.addEventListener('mousemove', onCropResizeMove)
  document.addEventListener('mouseup', onCropResizeEnd)
  e.preventDefault()
}

function onCropResizeMove(e) {
  if (!cropResizing || !cropRectSnapshot) return
  const rect = canvasEl.value.getBoundingClientRect()
  const sp = screenToImage(e.clientX - rect.left, e.clientY - rect.top)
  const dx = sp.x - cropResizeStart.x; const dy = sp.y - cropResizeStart.y
  const snap = { ...cropRectSnapshot }
  applyRectResize(snap, cropResizeDir, dx, dy)
  const imgW = imageNaturalWidth; const imgH = imageNaturalHeight; const minSize = 10
  if (snap.w < minSize) snap.w = minSize; if (snap.h < minSize) snap.h = minSize
  if (snap.x < 0) { snap.w += snap.x; snap.x = 0 }
  if (snap.y < 0) { snap.h += snap.y; snap.y = 0 }
  if (snap.x + snap.w > imgW) snap.w = imgW - snap.x
  if (snap.y + snap.h > imgH) snap.h = imgH - snap.y
  const curRatio = getCropRatio()
  if (curRatio && snap.w >= minSize && snap.h >= minSize) {
    const dir = cropResizeDir; const oldSnap = { ...snap }
    let newW = snap.w; let newH = snap.h
    if (Math.abs(dx) > Math.abs(dy)) { newH = Math.max(minSize, Math.round(newW / curRatio)) }
    else { newW = Math.max(minSize, Math.round(newH * curRatio)) }
    newW = Math.min(newW, imgW); newH = Math.min(newH, imgH)
    if (dir === 'se' || dir === 's' || dir === 'e') { snap.x = oldSnap.x; snap.y = oldSnap.y }
    else if (dir === 'nw') { snap.x = oldSnap.x + oldSnap.w - newW; snap.y = oldSnap.y + oldSnap.h - newH }
    else if (dir === 'n') { snap.x = oldSnap.x; snap.y = oldSnap.y + oldSnap.h - newH }
    else if (dir === 'w') { snap.x = oldSnap.x + oldSnap.w - newW; snap.y = oldSnap.y }
    else if (dir === 'ne') { snap.x = oldSnap.x; snap.y = oldSnap.y + oldSnap.h - newH }
    else if (dir === 'sw') { snap.x = oldSnap.x + oldSnap.w - newW; snap.y = oldSnap.y }
    snap.w = newW; snap.h = newH
    if (snap.x < 0) snap.x = 0; if (snap.y < 0) snap.y = 0
    if (snap.x + snap.w > imgW) snap.x = imgW - snap.w
    if (snap.y + snap.h > imgH) snap.y = imgH - snap.h
  }
  if (snap.w >= minSize && snap.h >= minSize && drawPreviewAnnot) {
    Object.assign(drawPreviewAnnot, snap)
    cropW.value = Math.round(snap.w); cropH.value = Math.round(snap.h)
    cropTick.value++; render()
  }
}

function onCropResizeEnd() {
  if (!cropResizing) return; cropResizing = false; cropResizeDir = ''; cropRectSnapshot = null
  document.removeEventListener('mousemove', onCropResizeMove)
  document.removeEventListener('mouseup', onCropResizeEnd)
}

function onCanvasWheel(e) {
  if (editingText.value) return; const rect = canvasEl.value.getBoundingClientRect()
  const sx = e.clientX - rect.left; const sy = e.clientY - rect.top; const { x: ix, y: iy } = screenToImage(sx, sy)
  const overImage = ix >= 0 && iy >= 0 && ix <= imageNaturalWidth && iy <= imageNaturalHeight
  const cx = overImage ? sx : rect.width / 2; const cy = overImage ? sy : rect.height / 2
  const z = editing.value ? Math.max(0.1, Math.min(10, zoomLevel.value * (0.999 ** e.deltaY))) : Math.max(0.1, Math.min(20, zoomLevel.value * (0.999 ** e.deltaY)))
  zoomToPoint(z, cx, cy); updateCursor()
}

function onOverlayClick() {
  if (!editing.value) close()
}

function onCanvasMouseDown(e) {
  if (isResizing || editingText.value) return; const sx = e.offsetX; const sy = e.offsetY; const { x: ix, y: iy } = screenToImage(sx, sy)
  if (editing.value) {
    if (tool.value === 'select') {
      const idx = hitTest(ix, iy)
      if (idx >= 0) {
        selectedIndex.value = idx; const a = annotations.value[idx]
        syncStyleFromAnnot(a)
        dragOffset = { x: ix - a.x, y: iy - a.y }; isDragging = false; render(); return
      }
      selectedIndex.value = -1
      if (ix >= 0 && iy >= 0 && ix <= imageNaturalWidth && iy <= imageNaturalHeight) {
        isPanning = true; panStartMouse = { x: e.clientX, y: e.clientY }; panStartPan = { x: panX, y: panY }; canvasEl.value.style.cursor = 'grabbing'; render()
      }
      render(); return
    }
    if (tool.value === 'crop') {
      if (hasPendingCrop.value) return
      const cx = Math.max(0, Math.min(imageNaturalWidth, ix)); const cy = Math.max(0, Math.min(imageNaturalHeight, iy))
      cropStart = { x: cx, y: cy }; drawPreviewAnnot = { type: 'rect', x: cx, y: cy, w: 0, h: 0, isCrop: true }; render(); return
    }
    drawStart = { x: ix, y: iy }
    if (tool.value === 'text') { startTextEdit(ix, iy, -1); return }
    if (tool.value === 'arrow') { drawPreviewAnnot = { type: 'arrow', x: ix, y: iy, x2: ix, y2: iy, color: styleColor.value, strokeWidth: styleStroke.value, headStyle: headStyle.value }; render(); return }
    if (tool.value === 'rect') { drawPreviewAnnot = { type: 'rect', x: ix, y: iy, w: 0, h: 0, color: styleColor.value, strokeWidth: styleStroke.value }; render() }
    if (tool.value === 'mosaic') { drawPreviewAnnot = { type: 'mosaic', x: ix, y: iy, w: 0, h: 0, id: 'mosaic_' + Date.now(), bs: mosaicBlockSize.value }; render() }
  } else {
    if (ix >= 0 && iy >= 0 && ix <= imageNaturalWidth && iy <= imageNaturalHeight) {
      isPanning = true; panStartMouse = { x: e.clientX, y: e.clientY }; panStartPan = { x: panX, y: panY }; canvasEl.value.style.cursor = 'grabbing'
    } else { close() }
  }
}

function onCanvasMouseMove(e) {
  if (isResizing || editingText.value) return; const sx = e.offsetX; const sy = e.offsetY; const { x: ix, y: iy } = screenToImage(sx, sy)
  if (editing.value) {
    if (tool.value === 'select') {
      if (selectedIndex.value >= 0 && e.buttons === 1) {
        const a = annotations.value[selectedIndex.value]
        if (!isDragging) { pushHistory(); isDragging = true }
        if (a.type === 'rect' || a.type === 'mosaic') { a.x = ix - dragOffset.x; a.y = iy - dragOffset.y }
        else if (a.type === 'arrow') { const ow = a.x2 - a.x, oh = a.y2 - a.y; a.x = ix - dragOffset.x; a.y = iy - dragOffset.y; a.x2 = a.x + ow; a.y2 = a.y + oh }
        else if (a.type === 'text') { a.x = ix - dragOffset.x; a.y = iy - dragOffset.y }
        render(); return
      }
      if (isPanning) { panX = panStartPan.x + (e.clientX - panStartMouse.x); panY = panStartPan.y + (e.clientY - panStartMouse.y); clampPan(); canvasEl.value.style.cursor = 'grabbing'; render() }
      else { canvasEl.value.style.cursor = (ix >= 0 && iy >= 0 && ix <= imageNaturalWidth && iy <= imageNaturalHeight) ? 'grab' : 'default' }
      return
    }
    if (tool.value === 'crop' && cropStart && drawPreviewAnnot) {
      let cix = Math.max(0, Math.min(imageNaturalWidth, ix)); let ciy = Math.max(0, Math.min(imageNaturalHeight, iy))
      const curRatio = getCropRatio()
      if (curRatio) {
        const dw = cix - cropStart.x; const dh = ciy - cropStart.y; const absDw = Math.abs(dw); const absDh = Math.abs(dh)
        if (absDw > absDh) { const signH = dh >= 0 ? 1 : -1; ciy = cropStart.y + signH * (absDw / curRatio) }
        else { const signW = dw >= 0 ? 1 : -1; cix = cropStart.x + signW * (absDh * curRatio) }
        ciy = Math.max(0, Math.min(imageNaturalHeight, ciy))
        cix = Math.max(0, Math.min(imageNaturalWidth, cix))
      }
      drawPreviewAnnot.x = Math.min(cropStart.x, cix); drawPreviewAnnot.y = Math.min(cropStart.y, ciy)
      drawPreviewAnnot.w = Math.abs(cix - cropStart.x); drawPreviewAnnot.h = Math.abs(ciy - cropStart.y); render(); return
    }
    if (drawPreviewAnnot && drawStart) {
      if (tool.value === 'arrow') { drawPreviewAnnot.x2 = ix; drawPreviewAnnot.y2 = iy; render() }
      else if (tool.value === 'rect' || tool.value === 'mosaic') { drawPreviewAnnot.x = Math.min(drawStart.x, ix); drawPreviewAnnot.y = Math.min(drawStart.y, iy); drawPreviewAnnot.w = Math.abs(ix - drawStart.x); drawPreviewAnnot.h = Math.abs(iy - drawStart.y); render() }
    }
  } else {
    if (!isPanning) { canvasEl.value.style.cursor = (ix >= 0 && iy >= 0 && ix <= imageNaturalWidth && iy <= imageNaturalHeight) ? 'grab' : 'default'; return }
    panX = panStartPan.x + (e.clientX - panStartMouse.x); panY = panStartPan.y + (e.clientY - panStartMouse.y); clampPan(); canvasEl.value.style.cursor = 'grabbing'; render()
  }
}

function onCanvasMouseUp(e) {
  if (isResizing || editingText.value) return
  if (editing.value) {
    if (isPanning) { isPanning = false; updateCursor(); return }
    if (tool.value === 'select') { isDragging = false; return }
    if (tool.value === 'crop') {
      if (drawPreviewAnnot && drawPreviewAnnot.w > 5 && drawPreviewAnnot.h > 5) {
        cropStart = null; cropW.value = Math.round(drawPreviewAnnot.w); cropH.value = Math.round(drawPreviewAnnot.h)
        hasPendingCrop.value = true; cropTick.value++; render()
      } else { cancelCrop() }
      drawStart = null; return
    }
    if (drawPreviewAnnot) {
      if (tool.value === 'arrow') {
        const dx = drawPreviewAnnot.x2 - drawPreviewAnnot.x; const dy = drawPreviewAnnot.y2 - drawPreviewAnnot.y
        if (Math.sqrt(dx * dx + dy * dy) < 10) { drawPreviewAnnot = null; drawStart = null; render(); return }
        const a = { ...drawPreviewAnnot, id: 'arrow_' + Date.now() }
        pushHistory(); annotations.value.push(a)
      } else if (tool.value === 'rect' && drawPreviewAnnot.w > 5 && drawPreviewAnnot.h > 5) {
        pushHistory(); annotations.value.push({ ...drawPreviewAnnot, id: 'rect_' + Date.now() })
      } else if (tool.value === 'mosaic' && drawPreviewAnnot.w > 10 && drawPreviewAnnot.h > 10) {
        pushHistory(); annotations.value.push({ ...drawPreviewAnnot })
      }
      drawPreviewAnnot = null; drawStart = null; render()
    }
  } else { isPanning = false; updateCursor() }
}

function hitTest(ix, iy) {
  for (let i = annotations.value.length - 1; i >= 0; i--) {
    const a = annotations.value[i]
    if (a.type === 'rect' || a.type === 'mosaic') {
      if (ix >= a.x - 3 && ix <= a.x + a.w + 3 && iy >= a.y - 3 && iy <= a.y + a.h + 3) return i
    } else if (a.type === 'arrow') {
      const dx = a.x2 - a.x; const dy = a.y2 - a.y; const len = Math.sqrt(dx * dx + dy * dy)
      if (len < 1) continue
      const t = Math.max(0, Math.min(1, ((ix - a.x) * dx + (iy - a.y) * dy) / (len * len)))
      const px = a.x + t * dx; const py = a.y + t * dy
      if (Math.sqrt((ix - px) ** 2 + (iy - py) ** 2) < 8 / zoomLevel.value) return i
    } else if (a.type === 'text') {
      const fs = a.fontSize || 24; ctx.font = `${fs}px sans-serif`
      const m = ctx.measureText(a.text || '')
      if (ix >= a.x && ix <= a.x + m.width && iy >= a.y && iy <= a.y + fs) return i
    }
  }
  return -1
}

function handleKeydown(e) {
  if (e.key === 'Escape') {
    e.preventDefault()
    if (editing.value) {
      stopEdit()
    } else {
      close()
    }
  }
  if (e.key === 'Delete' && editing.value && selectedIndex.value >= 0 && !isResizing) deleteSelected()
}

watch(() => props.filePath, (id) => {
  if (id) nextTick(() => initCanvas())
  else disposeCanvas()
})

onMounted(() => {
  if (props.filePath) initCanvas()
  document.addEventListener('keydown', handleKeydown)
})

onBeforeUnmount(() => {
  document.removeEventListener('keydown', handleKeydown)
  disposeCanvas()
})
</script>

<style scoped>
.ip-overlay {
  position: fixed; top: 0; left: 0; width: 100vw; height: 100vh;
  background: rgba(0, 0, 0, 0.55); z-index: 9999;
}
.ip-canvas {
  width: 100%; height: 100%; display: flex; align-items: center;
  justify-content: center; overflow: hidden; position: relative;
}
.ip-canvas canvas { display: block; }
.ip-info {
  position: fixed; top: 20px; left: 50%; transform: translateX(-50%);
  color: #fff; font-size: 16px; text-shadow: 0 2px 4px rgba(0,0,0,0.5);
  padding: 8px 20px; background: rgba(0, 0, 0, 0.5); border-radius: 20px; z-index: 10000;
  display: flex; align-items: center; gap: 10px;
}
.ip-info-name { max-width: 360px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.ip-info-divider { font-size: 14px; color: rgba(255,255,255,0.3); }
.ip-info-res { font-size: 13px; color: rgba(255,255,255,0.7); white-space: nowrap; }
.ip-toolbar {
  position: fixed; bottom: 30px; left: 50%; transform: translateX(-50%);
  display: flex; align-items: center; justify-content: center; gap: 6px;
  padding: 8px 14px; background: rgba(0, 0, 0, 0.4); border-radius: 30px; z-index: 10000;
}
.ip-toolbar .el-button {
  background: rgba(255, 255, 255, 0.12); border: none; color: rgba(255, 255, 255, 0.75);
  width: 36px; height: 36px;
}
.ip-toolbar .el-button:hover { background: rgba(255, 255, 255, 0.28); color: #fff; }
.ip-toolbar .el-button .el-icon { font-size: 16px; }
.ip-scale { font-size: 14px; color: #fff; min-width: 50px; text-align: center; }
.ip-toolbar-divider { width: 1px; height: 24px; background: rgba(255,255,255,0.2); flex-shrink: 0; }
.ip-property-bar {
  position: fixed; bottom: 96px; left: 50%; transform: translateX(-50%);
  display: flex; align-items: center; gap: 8px; padding: 6px 14px;
  background: rgba(0, 0, 0, 0.4); border-radius: 20px; z-index: 10000;
}
.ip-prop-label { color: rgba(255,255,255,0.55); font-size: 12px; user-select: none; }
.ip-text-group { position: absolute; display: flex; align-items: center; gap: 4px; z-index: 5; }
.ip-text-input {
  background: rgba(0,0,0,0.5); border: none; border-bottom: 2px solid #409eff;
  outline: none; padding: 2px 4px; min-width: 60px; color: #fff; font-family: sans-serif;
}
.ip-text-input:focus { border-bottom-color: #fff; }
.ip-text-confirm-btn { flex-shrink: 0; --el-button-size: 24px; }
.ip-arrow-styles { display: flex; gap: 2px; }
.ip-arrow-style-btn {
  display: flex; align-items: center; justify-content: center;
  width: 28px; height: 28px; border: 1px solid rgba(255,255,255,0.2);
  border-radius: 4px; background: rgba(255,255,255,0.1); color: rgba(255,255,255,0.7);
  cursor: pointer; padding: 0; transition: all 0.15s;
}
.ip-arrow-style-btn:hover { background: rgba(255,255,255,0.2); color: #fff; }
.ip-arrow-style-btn.active { background: rgba(64,158,255,0.3); border-color: #409eff; color: #fff; }
.ip-selection-overlay { position: absolute; pointer-events: none; z-index: 4; }
.ip-crop-actions { position: absolute; z-index: 5; display: flex; gap: 6px; pointer-events: auto; }
.ip-crop-btn {
  display: flex; align-items: center; justify-content: center;
  width: 22px; height: 22px; border-radius: 50%;
  font-size: 14px; color: #fff; cursor: pointer; user-select: none;
  box-shadow: 0 1px 4px rgba(0,0,0,0.25); transition: transform 0.12s;
}
.ip-crop-btn:hover { transform: scale(1.15); }
.ip-crop-btn:active { transform: scale(0.95); }
.ip-crop-btn-confirm { background: #67c23a; }
.ip-crop-btn-cancel { background: #f56c6c; }
.ip-crop-dim-label { font-size: 11px; color: #606266; white-space: nowrap; }
.ip-crop-dim-input {
  width: 48px; height: 20px; font-size: 11px; text-align: center;
  border: 1px solid #c0c4cc; border-radius: 3px; outline: none;
  background: #fff; -moz-appearance: textfield;
}
.ip-crop-dim-input::-webkit-inner-spin-button,
.ip-crop-dim-input::-webkit-outer-spin-button { -webkit-appearance: none; margin: 0; }
.ip-crop-dim-input:focus { border-color: #409eff; }
.ip-crop-center {
  position: absolute; top: 50%; left: 50%;
  width: 8px; height: 8px;
  background: rgba(255,255,255,.55);
  border: 1.5px solid rgba(64,158,255,.7);
  border-radius: 50%;
  transform: translate(-50%, -50%);
  pointer-events: none; z-index: 6;
}
.ip-crop-overlay { position: absolute; z-index: 4; pointer-events: none; }
.ip-crop-fill {
  position: absolute; inset: 0;
  pointer-events: auto; cursor: move;
}
.ip-selection-fill {
  position: absolute; inset: 0; background: rgba(64, 158, 255, 0.06);
  border: 2px solid #409eff; border-radius: 1px; pointer-events: none;
}
.ip-resize-handle {
  position: absolute; width: 10px; height: 10px; background: #fff;
  border: 2px solid #409eff; border-radius: 1px; box-sizing: border-box;
  z-index: 5; pointer-events: auto;
}
.ip-resize-handle::before { content: ''; position: absolute; inset: -5px; }
.ip-handle-nw { top: -6px; left: -6px; cursor: nw-resize; }
.ip-handle-n  { top: -6px; left: 50%; margin-left: -6px; cursor: n-resize; }
.ip-handle-ne { top: -6px; right: -6px; cursor: nesw-resize; }
.ip-handle-e  { top: 50%; right: -6px; margin-top: -6px; cursor: e-resize; }
.ip-handle-se { bottom: -6px; right: -6px; cursor: se-resize; }
.ip-handle-s  { bottom: -6px; left: 50%; margin-left: -6px; cursor: s-resize; }
.ip-handle-sw { bottom: -6px; left: -6px; cursor: sw-resize; }
.ip-handle-w  { top: 50%; left: -6px; margin-top: -6px; cursor: w-resize; }
.ip-arrow-endpoint {
  position: absolute; width: 14px; height: 14px; background: #fff;
  border: 2.5px solid #409eff; border-radius: 50%; box-sizing: border-box;
  z-index: 5; pointer-events: auto; cursor: move;
}
.ip-arrow-endpoint::before { content: ''; position: absolute; inset: -5px; border-radius: 50%; }
.ip-arrow-move {
  position: absolute; width: 12px; height: 12px;
  background: rgba(64, 158, 255, 0.25); border: 2px solid #409eff; border-radius: 50%;
  box-sizing: border-box; z-index: 5; pointer-events: auto; cursor: move;
}
.ip-arrow-move::before { content: ''; position: absolute; inset: -6px; border-radius: 50%; }
</style>

<style>
.ip-editor-popper { z-index: 10001 !important; }
.el-popper { z-index: 10001 !important; }
</style>
