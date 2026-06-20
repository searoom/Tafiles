# JIS-Tauri — ERP 桌面客户端替换计划

> 基于 Tafiles 项目架构，用 Tauri 2.x 桌面客户端替换 Delphi mqs.exe
> 分析来源：D:\Studio\jis6（FastAPI + Vue3 + Delphi MQS 珠宝企业管理系统）

---

## 一、项目背景

### 现有架构（jis6）
```
┌──────────────────────────────────────────┐
│  mqs.exe (Delphi WebView2 Shell)         │
│  ├── 窗口管理（无边框、拖拽、缩放）         │
│  ├── 外部标签页管理（子 WebView2）         │
│  ├── 打印桥接（→ jia.exe）                 │
│  ├── 系统托盘                               │
│  └── 自动更新                               │
├── Vue3 SPA（业务界面）                      │
│  ├── router/index.js                       │
│  ├── utils/bridge.js ← 通信抽象层           │
│  ├── utils/request.js ← Axios → FastAPI    │
│  ├── utils/printer.js ← 打印引擎           │
│  └── utils/shell.js ← 窗口控制包装          │
├── FastAPI 后端（42 路由）                    │
│  ├── /api/auth/* 认证                        │
│  ├── /api/files/* 文件                        │
│  ├── /api/sys-pages/* 动态页面               │
│  ├── /api/sys-pages/*/prints/* 打印模板      │
│  ├── /api/proxy/* 反向代理                   │
│  └── ……                                      │
├── jia.exe (Delphi FastReport 打印服务)       │
│  ├── /preview - 打印预览                      │
│  ├── /print - 直接打印                        │
│  ├── /preview-pdf - PDF 生成                  │
│  ├── /design - 模板设计                       │
│  └── /api/* 代理到 FastAPI                   │
└── SSO Service（独立认证服务）                 │
    └── /api/auth/*  SSO 登录/注册              │
```

### 痛点与机会
| 问题 | Delphi (mqs.exe) | Tauri 替换方案 |
|------|-----------------|---------------|
| 窗口管理 | 自定义 DWM 兼容代码，复杂 | Tauri 原生 window API |
| 外部标签页 | 子 EdgeBrowser 控件，内存高 | Tauri 多 WebviewWindow |
| 打印桥接 | 需 jia.exe HTTP 通信 | Rust 端直接调用打印 API / 保留 jia |
| 硬件设备 | 通过 Delphi 串口/DLL 驱动 | Rust 端 serial / hid / USB crate |
| 跨平台 | Windows only | 天然跨平台 |
| 包体积 | 大（Delphi RTL + DLL） | 小（单 exe + WebView2） |
| 开发效率 | Delphi 生态萎缩 | Rust 生态活跃，Vue3 积累 |

---

## 二、总体架构

### 目标架构
```
┌────────────────────────────────────────────────────┐
│  jis-tauri (Tauri 2.x Desktop Client)              │
│  ├── 主 WebView（内嵌 Vue3 SPA - 业务界面）        │
│  │   └── bridge.js → window.__TAURI__.invoke       │
│  ├── 独立预览窗口（类似 tafiles PreviewPage）          │
│  │   ├── 文件预览（PDF/Office/图片）                │
│  │   ├── 打印预览                                   │
│  │   └── 外部网页（钉钉/企微等）                     │
│  ├── 系统托盘 + 通知                                 │
│  ├── 硬件抽象层                                      │
│  │   ├── 打印机（→ jia.exe / Rust 打印）            │
│  │   ├── 条码扫描器（Serial / HID）                  │
│  │   ├── 电子秤（Serial）                            │
│  │   └── 标签打印机（ZPL/EPL）                       │
│  └── 本地服务                                        │
│      ├── 文件下载管理器                               │
│      ├── 本地缓存 + SQLite                            │
│      ├── 自动更新                                     │
│      └── 日志/诊断                                    │
├── Vue3 SPA（来自 jis6/web/，业务界面）               │
│  └── bridge.js 适配 Tauri invoke                     │
├── FastAPI 后端（来自 jis6/api/，保持不变）           │
├── jia.exe（可选保留，或逐步替换为 Rust 打印引擎）     │
└── SSO Service（来自 jis6/sso/，保持不变）             │
```

### 核心原则
1. **业务与系统分离**：Vue3 SPA 只负责业务 UI，调用 `/api/*`；Tauri 客户端负责系统级功能
2. **前后端分离**：所有业务 API 走 FastAPI，Tauri 只做 IPC 桥接 + 本地操作
3. **增量替换**：先替换窗口管理 + 外部标签页，再替换打印 + 硬件，最后替换自动更新
4. **向后兼容**：bridge.js 保持现有接口签名，让 Vue3 SPA 无需大改

---

## 三、项目结构（参考 Tafiles）

```
D:\Studio\jis-tauri\
├── AGENTS.md                    # 项目约定
├── config.json                  # 运行时配置（复用 jis6 格式）
├── tafiles.exe                  # 构建产物
│
├── source/                      # 源码根目录
│   ├── src-tauri/               # Tauri Rust 后端
│   │   ├── src/
│   │   │   ├── main.rs          # 入口
│   │   │   ├── lib.rs           # 插件 + 命令注册 + 系统托盘
│   │   │   ├── commands.rs      # 所有 Tauri 命令（窗口/标签页/打印/硬件）
│   │   │   ├── config.rs        # 配置加载（解析 config.json）
│   │   │   ├── tray.rs          # 系统托盘 + 菜单
│   │   │   ├── printer.rs       # 打印引擎（调用 jia.exe 或 Rust 打印）
│   │   │   ├── hardware.rs      # 硬件设备（串口/HID 通信）
│   │   │   └── updater.rs       # 自动更新
│   │   ├── Cargo.toml
│   │   └── tauri.conf.json
│   │
│   ├── index.html                # 主窗口入口（加载 SPA）
│   ├── login.html                # 登录窗口入口（精简登录界面）
│   ├── preview.html              # 预览窗口入口（参考 tafiles）
│   ├── print.html                # 打印预览窗口
│   │
│   ├── src/                     # Vue3 前端（Tauri 客户端 UI）
│   │   ├── main.js              # 主入口（路由挂载）
│   │   ├── login-main.js        # 登录窗口入口
│   │   ├── preview-main.js      # 预览窗口入口
│   │   ├── App.vue
│   │   ├── router.js
│   │   ├── styles/main.css
│   │   ├── utils/
│   │   │   ├── bridge.js        # 通信桥接（替换 jis6/web/src/utils/bridge.js）
│   │   │   └── request.js       # Axios 实例（从 jis6 引入）
│   │   ├── stores/app.js
│   │   └── components/
│   │       ├── WebViewer.vue    # 包裹 iframe 或 Tauri Webview 加载 SPA
│   │       ├── TitleBar.vue      # 自定义标题栏（参考 jis6 TitleBar）
│   │       ├── PreviewPage.vue  # 预览窗口（参考 tafiles）
│   │       └── PrintPage.vue    # 打印预览窗口
│   │
│   ├── package.json
│   └── vite.config.js
│
└── web/                          # 符号链接或 submodule → jis6/web/
    └── src/                     # 原始 Vue3 SPA（业务界面）
```

---

## 四、功能模块规划

### Phase 1：基础替换（1-2 月）
**目标：替换 mqs.exe 的窗口管理 + 外部标签页功能**

#### 1.1 窗口管理
- [ ] 无边框主窗口（decorations: false）
- [ ] 自定义标题栏（TitleBar.vue，复用 jis6 TitleBar 样式）
- [ ] 拖拽移动（window_start_drag）
- [ ] 窗口缩放（window_start_resize）
- [ ] 最大化/最小化/关闭
- [ ] 登录时小窗（690x350）→ 登录后展开
- [ ] 窗口状态记忆（位置/大小，参考 tafiles PreviewPage）

#### 1.2 外部标签页
- [ ] 子 WebviewWindow 创建/关闭
- [ ] 定位/大小同步（跟随主窗口）
- [ ] 标签页切换（Vue3 管理标签栏，Tauri 管理子窗口）
- [ ] 钉钉/企微等三方页面嵌入

#### 1.3 通信桥接
- [ ] bridge.js 适配 Tauri invoke 通道
- [ ] 保持现有接口签名（WindowClose, WindowMinimize, 等）
- [ ] 命令响应格式兼容（id + success + data 模式）
- [ ] BridgeReady 事件（HTTP port + token）
- [ ] 事件推送（WindowCloseRequest, Logout, 等）

#### 1.4 系统托盘
- [ ] 托盘图标 + 菜单（显示/隐藏/退出/关于）
- [ ] 托盘通知（消息提醒、打印完成等）

---

### Phase 2：文件与打印（2-3 月）
**目标：替换/增强文件管理与打印功能**

#### 2.1 文件管理
- [ ] 文件上传/下载管理器（本地存储管理与进度）
- [ ] 文件预览（参考 tafiles 多窗口预览模式）
- [ ] 图片查看器（参考 tafiles ImageViewer）
- [ ] 本地缓存管理（离线文件）

#### 2.2 打印引擎
- [ ] 保留 jia.exe 桥接（向后兼容现有打印流程）
- [ ] 替换为 Rust 本地打印（可选项）
  - [ ] FR3 模板解析（Rust 端）
  - [ ] PDF 生成（printpdf / genpdf crate）
  - [ ] 直接打印（windows 打印 API）
- [ ] 打印机列表枚举
- [ ] 打印状态监控
- [ ] 打印队列管理

#### 2.3 标签打印
- [ ] ZPL/EPL 标签语言生成
- [ ] 标签打印机直接驱动（网络/串口）
- [ ] 模板管理（本地缓存 + 同步服务端）

---

### Phase 3：硬件集成（3-5 月）
**目标：连接 MES 关键硬件设备**

#### 3.1 条码扫描器
- [ ] USB HID 条码枪接入（hidapi crate）
- [ ] 串口条码枪接入（serialport crate）
- [ ] 扫描事件 → 注入到 SPA 输入框
- [ ] 批量扫描模式

#### 3.2 电子秤
- [ ] 串口电子秤通信（serialport crate）
- [ ] 重量读取（标准协议：Mettler Toledo / Sartorius 等）
- [ ] 称重事件 → 填充表单字段
- [ ] 连续称重/稳定检测

#### 3.3 打印机驱动
- [ ] 热敏标签打印机（ZPL/EPL）
- [ ] 针式打印机（票据打印）
- [ ] 打印进度回调
- [ ] 打印错误处理

#### 3.4 其他硬件
- [ ] IC/ID 卡读卡器（USB CCID）
- [ ] RFID 读写器
- [ ] 摄像头（条码/二维码扫描）
- [ ] 指纹识别器

---

### Phase 4：系统增强（5-8 月）
**目标：补充桌面客户端系统级能力**

#### 4.1 自动更新
- [ ] 更新检查（启动/定时）
- [ ] 增量下载
- [ ] 静默安装 + 回滚
- [ ] 更新日志展示

#### 4.2 离线能力
- [ ] 本地数据库（SQLite，关键业务数据缓存）
- [ ] 离线操作队列（网络恢复后同步）
- [ ] 网络状态检测（online/offline 切换）

#### 4.3 系统集成
- [ ] 文件关联（双击 .jis 文件打开）
- [ ] 桌面快捷方式
- [ ] 开机自启
- [ ] 锁屏/解锁监听
- [ ] 快捷键全局注册

#### 4.4 本地服务
- [ ] 日志系统（文件 + 远程）
- [ ] 性能监控（CPU/内存/磁盘）
- [ ] 诊断工具（连接测试、日志导出）
- [ ] 数据备份/恢复

---

### Phase 5：打印引擎替换（8-12 月，可选）
**目标：用 Rust 原生打印引擎替换 jia.exe**

- [ ] FR3 模板解析器（逆向工程 fr3 格式？或改用 JSON 模板）
- [ ] FastReport 兼容层（可选）
- [ ] Rust 端 PDF 渲染引擎
- [ ] 打印预览（参考 tafiles PreviewPage）
- [ ] 模板设计器（Web 端 hiprint 替代）

---

## 五、通信协议桥接设计

### 现有 bridge.js 接口（需在 Tauri 端实现）

```js
// 窗口控制（已实现）
window_close()           → commands.window_close
window_minimize()        → commands.window_minimize
window_maximize()        → commands.window_maximize
window_start_drag()      → commands.window_start_drag
window_start_resize(dir) → commands.window_start_resize
is_window_maximized()    → commands.is_window_maximized

// 外部标签页（已部分实现）
open_external_tab(url, id, title, options) → commands.open_external_tab
resize_external_tab(id, x, y, w, h)        → commands.resize_external_tab
close_external_tab(id)                     → commands.close_external_tab
show_external_tab(id)                      → commands.show_external_tab
hide_external_tab(id)                      → commands.hide_external_tab

// 打印（需实现）
print_preview(page_id, record_id, template_id)
print_direct(page_id, record_id, template_id, printer_name)
print_design(page_id, record_id, template_id)
ensure_jia_ready()

// 系统信息
get_config(key)
get_app_version()

// 对话框（可保留，或使用 Element Plus 原生）
msg_message(text)
err_message(text)
confirm(text) → boolean
```

### Tauri 命令对应
```rust
// commands.rs — 参考 18 个现有命令 + 新增

// 窗口（已实现）
#[tauri::command] fn window_close(app)
#[tauri::command] fn window_minimize(app)
#[tauri::command] fn window_maximize(app)
#[tauri::command] fn window_start_drag(app)
#[tauri::command] fn window_start_resize(app, direction)
#[tauri::command] fn is_window_maximized(app) → bool
#[tauri::command] fn set_window_size(app, width, height)
#[tauri::command] fn set_window_position(app, x, y)

// 外部标签页（已实现）
#[tauri::command] fn open_external_tab(app, url, id, title)
#[tauri::command] fn resize_external_tab(app, id, x, y, w, h)
#[tauri::command] fn close_external_tab(app, id)
#[tauri::command] fn show_external_tab(app, id)
#[tauri::command] fn hide_external_tab(app, id)
#[tauri::command] fn list_external_tabs(app)

// 打印 + 硬件（新增）
#[tauri::command] async fn print_preview(page_id, record_id, template_id)
#[tauri::command] async fn print_direct(page_id, record_id, template_id, printer_name)
#[tauri::command] async fn print_design(page_id, record_id, template_id)
#[tauri::command] async fn list_printers() → Vec<String>
#[tauri::command] fn ensure_jia_ready() → bool

// 硬件（新增）
#[tauri::command] async fn list_serial_ports() → Vec<String>
#[tauri::command] async fn open_serial_port(path, baud) → bool
#[tauri::command] async fn read_serial(timeout) → String
#[tauri::command] async fn write_serial(data)
#[tauri::command] async fn list_hid_devices() → Vec<DeviceInfo>

// 系统（新增）
#[tauri::command] fn get_config(key) → Option<serde_json::Value>
#[tauri::command] fn get_app_version() → String
#[tauri::command] fn open_in_browser(url)
#[tauri::command] fn open_file_explorer(path)
#[tauri::command] async fn check_update() → Option<UpdateInfo>
#[tauri::command] async fn install_update()
```

---

## 六、关键技术选型

| 领域 | 技术/依赖 | 说明 |
|------|----------|------|
| 桌面框架 | Tauri 2.x | 从 tafiles 沿用 |
| 前端框架 | Vue3 + Element Plus | 与现有 web 端一致 |
| 串口通信 | serialport 2.x | Rust crate，硬件连接 |
| HID 设备 | hidapi 2.x | Rust crate，条码枪/读卡器 |
| 打印 | reqwest → jia.exe | Phase 1-4 保留 jia |
| PDF 生成 | printpdf / genpdf | Phase 5 Rust 原生打印 |
| SQLite | rusqlite / sqlx | 本地缓存/离线 |
| 自动更新 | tauri-plugin-updater / self-replace | 内置插件或自实现 |
| IM 通知 | reqwest → WuKongIM API | 通过 Rust HTTP 客户端 |
| 配置解析 | serde_json | 读取 config.json |
| 窗口管理 | Tauri Window API | 多窗口管理 |

---

## 七、与 Tafiles 的差异点

| 维度 | Tafiles | JIS-Tauri |
|------|------|-----------|
| 核心场景 | 文件浏览 + 图片编辑 | ERP 桌面客户端 |
| 业务 UI | 自有 Vue3 组件 | iframe/Webview 加载远程 SPA |
| 预览目标 | 本地文件 | 远程业务文件 + 打印预览 |
| 硬件要求 | 无 | 串口/HID 设备驱动 |
| 打印要求 | 无 | 支持 ERP 打印模板（FR3） |
| 网络要求 | 无（纯本地） | 在线为主，离线可缓存 |
| 用户管理 | 单用户 | 多用户 + 角色 + SSO |
| 窗口需求 | 1 主 + N 预览 | 1 主 + N 标签页 + N 外部 |

---

## 八、实施路线图

```
Phase 1 (1-2月)
┌─────────────────────────────────────────────┐
│ 窗口管理 │ 外部标签页 │ 通信桥接 │ 系统托盘   │
│  ┌──────┐  ┌──────┐  ┌──────┐  ┌──────┐    │
│  │ Tauri │  │子窗口 │  │bridge│  │ 托盘 │    │
│  │ Window│  │管理   │  │适配  │  │ 菜单 │    │
│  └──────┘  └──────┘  └──────┘  └──────┘    │
└─────────────────────────────────────────────┘

Phase 2 (2-3月)
┌─────────────────────────────────────────────┐
│ 文件管理 │ 打印桥接 │ 标签打印 │ 预览增强    │
│  ┌──────┐  ┌──────┐  ┌──────┐  ┌──────┐    │
│  │下载器 │  │ jia  │  │ ZPL  │  │ 图片  │    │
│  │管理器 │  │桥接  │  │生成  │  │预览器 │    │
│  └──────┘  └──────┘  └──────┘  └──────┘    │
└─────────────────────────────────────────────┘

Phase 3 (3-5月)
┌─────────────────────────────────────────────┐
│ 条码扫描 │ 电子秤  │ 打印机  │ 其他硬件     │
│  ┌──────┐  ┌──────┐  ┌──────┐  ┌──────┐    │
│  │ HID  │  │串口  │  │热敏/  │  │读卡器│    │
│  │扫描枪 │  │ 秤   │  │针式  │  │RFID  │    │
│  └──────┘  └──────┘  └──────┘  └──────┘    │
└─────────────────────────────────────────────┘

Phase 4 (5-8月)
┌─────────────────────────────────────────────┐
│ 自动更新 │ 离线能力 │ 系统集成 │ 本地服务    │
│  ┌──────┐  ┌──────┐  ┌──────┐  ┌──────┐    │
│  │更新器 │  │SQLite│  │文件   │  │日志/ │    │
│  │      │  │队列  │  │关联   │  │诊断  │    │
│  └──────┘  └──────┘  └──────┘  └──────┘    │
└─────────────────────────────────────────────┘

Phase 5 (8-12月, 可选)
┌─────────────────────────────────────────────┐
│ Rust 打印引擎（替换 jia.exe）               │
│  ┌─────────────────────────────────┐        │
│  │ 模板解析 │ PDF 生成 │ 打印预览  │        │
│  └─────────────────────────────────┘        │
└─────────────────────────────────────────────┘
```

---

## 九、风险与缓解

| 风险 | 影响 | 缓解策略 |
|------|------|---------|
| jia.exe FastReport 模板兼容 | 高 | Phase 1-4 保留 jia.exe，Phase 5 可选替换 |
| 串口/HID 设备兼容性 | 中 | 抽象硬件层，支持模拟器调试 |
| Vue SPA 集成复杂度 | 中 | bridge.js 接口不变，分步适配 |
| 多 Tauri 窗口性能 | 低 | 参考 tafiles 已验证的多窗口模式 |
| 离线数据一致性 | 中 | 最后写入者获胜 + 冲突提示 |
| 自动更新安装权限 | 低 | NSIS/WiX 安装包，system level 安装 |
