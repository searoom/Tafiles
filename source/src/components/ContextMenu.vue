<template>
  <teleport to="body">
    <div v-if="visible" class="cm-overlay" @mousedown.self="close" @contextmenu.prevent="close">
      <div class="cm-menu" :style="{ left: x + 'px', top: y + 'px' }">
        <template v-for="(item, i) in items" :key="i">
          <div v-if="item.divider" class="cm-divider"></div>
          <div
            v-else
            class="cm-item"
            :class="{ disabled: item.disabled }"
            @click="handleClick(item)"
          >
            {{ item.label }}
          </div>
        </template>
      </div>
    </div>
  </teleport>
</template>

<script setup>
import { onMounted, onBeforeUnmount } from 'vue'

const props = defineProps({
  visible: Boolean,
  x: Number,
  y: Number,
  items: Array,
})

const emit = defineEmits(['close'])

function handleClick(item) {
  if (item.disabled || item.divider) return
  emit('close')
  item.onClick?.()
}

function close() {
  emit('close')
}

function onKeydown(e) {
  if (e.key === 'Escape') close()
}

onMounted(() => document.addEventListener('keydown', onKeydown))
onBeforeUnmount(() => document.removeEventListener('keydown', onKeydown))
</script>

<style scoped>
.cm-overlay {
  position: fixed;
  inset: 0;
  z-index: 9999;
}

.cm-menu {
  position: fixed;
  min-width: 160px;
  padding: 4px 0;
  background: #fff;
  border: 1px solid #e4e7ed;
  border-radius: 6px;
  box-shadow: 0 4px 16px rgba(0,0,0,0.12);
  user-select: none;
}

.cm-item {
  padding: 6px 16px;
  font-size: 13px;
  color: #303133;
  cursor: pointer;
  transition: background 0.12s;
  white-space: nowrap;
}

.cm-item:hover {
  background: #ecf5ff;
  color: #409eff;
}

.cm-item.disabled {
  color: #c0c4cc;
  cursor: default;
}

.cm-item.disabled:hover {
  background: transparent;
  color: #c0c4cc;
}

.cm-divider {
  height: 1px;
  background: #e4e7ed;
  margin: 4px 8px;
}
</style>
