import { defineStore } from 'pinia'
import { ref } from 'vue'

export const useAppStore = defineStore('app', () => {
  const currentPath = ref('')
  const previewFile = ref(null) // { path, name, ext, size, type }

  function setCurrentPath(path) {
    currentPath.value = path
  }

  function openPreview(file) {
    previewFile.value = file
  }

  function closePreview() {
    previewFile.value = null
  }

  return {
    currentPath,
    previewFile,
    setCurrentPath,
    openPreview,
    closePreview,
  }
})
