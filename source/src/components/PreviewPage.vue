<template>
  <ImageViewer
    v-if="fileType === 'image'"
    :file-path="filePath"
    :file-ext="fileExt"
    :file-name="fileName"
    @close="closeWindow"
  />
  <FileViewer
    v-else
    :file-path="filePath"
    :file-ext="fileExt"
    :file-name="fileName"
    :file-size="fileSize"
    @close="closeWindow"
  />
</template>

<script setup>
import { computed, onMounted, onBeforeUnmount } from 'vue'
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow'
import { imageExts } from '@/utils/fileType'
import ImageViewer from './ImageViewer.vue'
import FileViewer from './FileViewer.vue'

const params = new URLSearchParams(window.location.search)
const filePath = params.get('path') || ''
const fileName = params.get('name') || ''
const fileExt = (params.get('ext') || '').toLowerCase().replace(/^\./, '')
const fileSize = Number(params.get('size') || 0)

const fileType = computed(() => {
  if (imageExts.includes(fileExt)) return 'image'
  return 'file'
})

async function closeWindow() {
  const win = getCurrentWebviewWindow()
  try {
    const pos = await win.outerPosition()
    const size = await win.outerSize()
    const isMaximized = await win.isMaximized()
    if (!isMaximized) {
      localStorage.setItem('tafiles-preview-rect', JSON.stringify({
        x: pos.x, y: pos.y,
        width: size.width, height: size.height,
      }))
    }
  } catch {}
  await win.close()
}

function handleKeydown(e) {
  if (e.key === 'Escape') {
    e.preventDefault()
    closeWindow()
  }
}

onMounted(() => document.addEventListener('keydown', handleKeydown))
onBeforeUnmount(() => document.removeEventListener('keydown', handleKeydown))
</script>