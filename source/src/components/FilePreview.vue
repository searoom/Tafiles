<template>
  <teleport to="body">
    <div v-if="filePath">
      <ImageViewer
        v-if="fileType === 'image'"
        :file-path="filePath"
        :file-ext="fileExt"
        :file-name="fileName"
        @close="close"
      />
      <FileViewer
        v-else
        :file-path="filePath"
        :file-ext="fileExt"
        :file-name="fileName"
        :file-size="fileSize"
        @close="close"
      />
    </div>
  </teleport>
</template>

<script setup>
import { computed, onMounted, onBeforeUnmount } from 'vue'
import { useAppStore } from '@/stores/app'
import { imageExts } from '@/utils/fileType'
import ImageViewer from './ImageViewer.vue'
import FileViewer from './FileViewer.vue'

const props = defineProps({
  filePath: { type: String, default: '' },
  fileName: { type: String, default: '' },
  fileExt: { type: String, default: '' },
  fileSize: { type: Number, default: 0 },
})

const emit = defineEmits(['close'])

const ext = computed(() => (props.fileExt || '').toLowerCase().replace(/^\./, ''))

const fileType = computed(() => {
  const e = ext.value
  if (imageExts.includes(e)) return 'image'
  return 'file'
})

const close = () => emit('close')

const handleKeydown = (e) => {
  if (e.key === 'Escape') { e.preventDefault(); close() }
}

onMounted(() => document.addEventListener('keydown', handleKeydown))
onBeforeUnmount(() => document.removeEventListener('keydown', handleKeydown))
</script>
