import { createRouter, createWebHistory } from 'vue-router'
import FileBrowser from '@/components/FileBrowser.vue'

const routes = [
  {
    path: '/',
    name: 'browser',
    component: FileBrowser,
  },
]

const router = createRouter({
  history: createWebHistory(),
  routes,
})

export default router
