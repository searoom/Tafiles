<template>
  <el-dialog v-model="visible" title="WebDAV 连接管理" width="560px" top="5vh">
    <div class="webdav-manager">
      <div class="conn-list">
        <div v-for="conn in store.connections" :key="conn.id" class="conn-item">
          <div class="conn-info">
            <div class="conn-name">{{ conn.name }}</div>
            <div class="conn-url">{{ conn.url }}</div>
          </div>
          <div class="conn-actions">
            <el-button size="small" text @click="editConn(conn)">编辑</el-button>
            <el-button size="small" text type="danger" @click="confirmRemove(conn)">删除</el-button>
          </div>
        </div>
        <div v-if="store.connections.length === 0" class="conn-empty">暂无连接</div>
      </div>

      <el-divider />

      <el-form :model="form" label-position="top" size="small" @submit.prevent="saveConnection">
        <el-form-item label="连接名称">
          <el-input v-model="form.name" placeholder="例如：我的 NAS" />
        </el-form-item>
        <el-form-item label="服务器地址">
          <el-input v-model="form.url" placeholder="https://example.com/remote.php/dav/files/user" />
        </el-form-item>
        <el-form-item label="用户名">
          <el-input v-model="form.username" />
        </el-form-item>
        <el-form-item label="密码">
          <el-input v-model="form.password" type="password" show-password />
        </el-form-item>
        <el-form-item>
          <el-button type="primary" native-type="submit" :loading="saving" @click="saveConnection">
            {{ editingId ? '更新连接' : '添加连接' }}
          </el-button>
          <el-button v-if="editingId" @click="cancelEdit">取消</el-button>
          <el-button :loading="testing" @click="testConnection">测试连接</el-button>
        </el-form-item>
      </el-form>
      <div v-if="testResult" :class="['test-result', testResult.ok ? 'success' : 'error']">
        {{ testResult.msg }}
      </div>
    </div>
  </el-dialog>
</template>

<script setup>
import { ref, reactive, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { ElMessage, ElMessageBox } from 'element-plus'
import { useWebdavStore } from '@/stores/webdav'

const store = useWebdavStore()
const visible = defineModel('visible', { type: Boolean, default: false })

const form = reactive({
  name: '',
  url: '',
  username: '',
  password: '',
})
const editingId = ref('')
const saving = ref(false)
const testing = ref(false)
const testResult = ref(null)

function editConn(conn) {
  editingId.value = conn.id
  form.name = conn.name
  form.url = conn.url
  form.username = conn.username
  form.password = conn.password || ''
  testResult.value = null
}

function cancelEdit() {
  editingId.value = ''
  resetForm()
}

function resetForm() {
  form.name = ''
  form.url = ''
  form.username = ''
  form.password = ''
  testResult.value = null
}

async function saveConnection() {
  if (!form.name || !form.url) {
    ElMessage.warning('请填写连接名称和服务器地址')
    return
  }
  saving.value = true
  try {
    if (editingId.value) {
      await store.updateConnection(editingId.value, form.name, form.url, form.username, form.password)
      ElMessage.success('连接已更新')
    } else {
      await store.addConnection(form.name, form.url, form.username, form.password)
      ElMessage.success('连接已添加')
    }
    resetForm()
    editingId.value = ''
  } catch (e) {
    ElMessage.error('操作失败: ' + e)
  } finally {
    saving.value = false
  }
}

async function testConnection() {
  if (!form.url) {
    ElMessage.warning('请先填写服务器地址')
    return
  }
  testing.value = true
  testResult.value = null
  let tempId = null
  try {
    const info = await invoke('webdav_connect', {
      url: form.url,
      name: form.name || '测试',
      username: form.username,
      password: form.password,
    })
    tempId = info.id
    await invoke('webdav_list', { sessionId: info.id, path: '/' })
    testResult.value = { ok: true, msg: '连接成功，服务器可用' }
  } catch (e) {
    testResult.value = { ok: false, msg: '连接失败: ' + e }
  } finally {
    if (tempId) {
      try { await invoke('webdav_disconnect', { sessionId: tempId }) } catch {}
    }
    testing.value = false
  }
}

async function confirmRemove(conn) {
  try {
    await ElMessageBox.confirm(`确定删除连接 "${conn.name}"？`, '确认', {
      type: 'warning',
    })
    await store.removeConnection(conn.id)
    ElMessage.success('已删除')
  } catch {}
}

watch(visible, (v) => {
  if (!v) {
    resetForm()
    editingId.value = ''
  }
})
</script>

<style scoped>
.webdav-manager {
  max-height: 70vh;
  overflow-y: auto;
}

.conn-list {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.conn-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 12px;
  border-radius: 6px;
  background: #f7f8fa;
}

.conn-name {
  font-size: 14px;
  font-weight: 500;
  color: #303133;
}

.conn-url {
  font-size: 12px;
  color: #909399;
  word-break: break-all;
}

.conn-actions {
  display: flex;
  gap: 4px;
  flex-shrink: 0;
}

.conn-empty {
  text-align: center;
  padding: 24px;
  color: #909399;
  font-size: 13px;
}

.test-result {
  margin-top: 8px;
  padding: 8px 12px;
  border-radius: 4px;
  font-size: 13px;
  white-space: pre-wrap;
  word-break: break-all;
}

.test-result.success {
  background: #f0f9eb;
  color: #67c23a;
}

.test-result.error {
  background: #fef0f0;
  color: #f56c6c;
}
</style>
