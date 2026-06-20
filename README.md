# Tafiles — 系统级文件浏览 / 预览 / 编辑工具

Tafiles 是一款跨平台桌面工具，集文件浏览、多格式预览、图片编辑于一体。基于 **Tauri 2.x + Rust + Vue3 + Element Plus** 构建，提供原生性能体验。

## 功能

- **文件浏览** — 多视图（大/中/小图标、列表）、树状目录侧栏、面包屑导航、Windows 原生图标、图片缩略图
- **图片编辑器** — 标注（矩形/箭头/文字/马赛克）、裁剪（自由/原比例/固定比例）、旋转、撤销/重做、保存/另存
- **多格式预览** — PDF、Excel、Word、代码（CodeMirror 语法高亮）、纯文本，支持独立窗口多开
- **系统集成** — 多窗口独立预览、窗口位置记忆、原生对话框、系统默认程序打开

## 快速开始

```bash
cd source
npm install
npm run tauri dev     # 开发模式
npm run tauri build   # 构建发布
```

## 技术栈

| 层 | 技术 |
|---|------|
| 前端 | Vue 3 + Vite + Element Plus + Pinia + CodeMirror 6 |
| 桌面 | Tauri 2.x (Rust) |
| 后端 | Rust (image, serde, tokio, windows-sys) |
| 预览 | @vue-office/excel, @vue-office/docx, pdfjs-dist |

## 预览支持

图片 — PDF — Excel — Word — 代码(JS/TS/Python/HTML/CSS/JSON/XML/SQL/…) — 纯文本
