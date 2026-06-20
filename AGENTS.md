# Tafiles — 系统级文件浏览/预览/编辑工具

## 技术栈
- **前端**: Vue3 + Vite + Element Plus + vue-router + Pinia
- **桌面框架**: Tauri 2.x (Rust)
- **关键依赖**: @vue-office/excel, @vue-office/docx, vue-codemirror, codemirror, pdfjs-dist

## 目录结构
```
D:\Studio\Tafiles\
├── AGENTS.md                    # 项目约定
├── source/                      # 源码根目录
│   ├── src-tauri/               # Tauri Rust 后端
│   │   ├── src/
│   │   │   ├── main.rs          # 入口
│   │   │   ├── lib.rs           # 插件+命令注册
│   │   │   └── commands.rs      # 所有 Tauri 命令
│   │   ├── Cargo.toml
│   │   └── tauri.conf.json
│   ├── preview.html              # 预览窗口入口
│   ├── src/                     # Vue3 前端
│   │   ├── main.js
│   │   ├── preview-main.js       # 预览窗口 JS 入口
│   │   ├── App.vue
│   │   ├── router.js
│   │   ├── styles/main.css
│   │   ├── utils/fileType.js
│   │   ├── stores/app.js
│   │   └── components/
│   │       ├── FileBrowser.vue  # 文件浏览
│   │       ├── FilePreview.vue  # 预览路由 (fallback)
│   │       ├── PreviewPage.vue  # 预览窗口页面
│   │       ├── ImageViewer.vue  # 图片查看+编辑
│   │       └── FileViewer.vue   # Office/PDF/文本预览
│   ├── package.json
│   └── vite.config.js
└── tafiles.exe                   # 构建产物 (release)
```

## 构建命令
```bash
cd source
npm run tauri build
# 产物: source/src-tauri/target/release/tafiles.exe
# → 自动复制到 D:\Studio\Tafiles\tafiles.exe
```

## 代码约定

### Vue 组件
- 使用 `<script setup>` 语法
- 样式使用 `<style scoped>`，全局样式放在 `styles/main.css`
- 组件命名: PascalCase, 多单词
- 事件命名: kebab-case (如 `@file-changed`)
- Props: camelCase, 使用 `defineProps`
- Emits: camelCase, 使用 `defineEmits`

### 文件引用替换模式
参考代码 `D:\Studio\jis6\web\src\components\` 中:
- `fetch('/api/files/${id}/preview')` → `invoke('read_file_binary', {path})` 转 Blob URL
- `fetch('/api/files/${id}/annotations')` → 本地 `.tafiles-annotations.json`
- `fetch('/api/files/${id}/thumb')` → `invoke('get_thumbnail', {path})` 转 Blob URL
- `Shell.isShell` / `Shell.send()` → 移除, Tauri 原生窗口控制
- `@/utils/request.js` → 移除, 无 HTTP API

### 文件类型判断
```js
// 图片
const imageExts = ['jpg','jpeg','png','gif','bmp','webp','svg']
// Office
const officeExts = { pdf, xlsx, docx, doc, ppt, pptx, xls }
// 代码
const codeExts = { js, ts, jsx, tsx, py, html, css, ... }
```

### Tauri 命令 (Invoke API)
| 命令 | 参数 | 返回 |
|------|------|------|
| `read_dir` | `path: String` | `FileEntry[]` |
| `read_file_binary` | `path: String` | `number[]` (u8) |
| `read_file_text` | `path: String` | `String` |
| `write_file` | `path: String, data: number[]` | `void` |
| `pick_folder` | 无 | `String \| null` |
| `get_image_dimensions` | `path: String` | `{width, height}` |
| `get_thumbnail` | `path: String, max_size: u32` | `number[]` |
| `rotate_image` | `path, degrees, output_path?` | `String` (输出路径) |
| `crop_image` | `path, x, y, w, h, output_path?` | `String` (输出路径) |
| `load_annotations` | `path: String` | `String` (JSON) |
| `save_annotations` | `path: String, data: String` | `void` |

### 窗口配置
- 主窗口标题: "Tafiles"
- 默认尺寸: 1200 x 800
- 最小尺寸: 800 x 600
- 可调整大小
- 预览窗口: 通过 `WebviewWindow` 以最大化方式打开，label 为 "preview"
- 预览窗口入口: `/preview.html?path=&name=&ext=&size=`