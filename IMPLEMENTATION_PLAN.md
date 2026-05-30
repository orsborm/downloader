# 全协议下载器 — 分阶段落地规划

> 基于 PRD v4.0，将产品从 0 到 1、1 到 2、2 到 3 分三个阶段落地
> 每个阶段有明确的交付边界、技术任务、验收标准和预估周期

---

## 开发进度追踪

### 0 → 1 MVP 阶段

| 周 | 任务模块 | 状态 | 完成文件 |
|---|---|---|---|
| 1-2 | Tauri v2 项目脚手架 | ✅ 已完成 | `package.json`, `Cargo.toml`, `tauri.conf.json`, `vite.config.ts`, `tailwind.config.ts`, `tsconfig.json` |
| 1-2 | Rust 后端基础架构 | ✅ 已完成 | `storage/db.rs`, `storage/config.rs`, `storage/mod.rs` |
| 1-2 | 前端基础架构 | ✅ 已完成 | `stores/taskStore.ts`, `hooks/useTaskEvents.ts`, `hooks/useClipboard.ts`, `lib/types.ts`, `lib/format.ts`, `lib/tauri-api.ts` |
| 3-5 | HTTP/HTTPS 下载引擎 | ✅ 已完成 | `engine/http.rs`（多线程分块、断点续传、代理、限速） |
| 3-5 | 任务调度引擎 | ✅ 已完成 | `engine/task_manager.rs`（并发控制、暂停/恢复、取消、状态推送） |
| 6-8 | BT 引擎框架 | ✅ 已完成 | `engine/bt/mod.rs`（librqbit 集成框架，待实现核心下载） |
| 6-8 | Tauri IPC 命令 | ✅ 已完成 | `commands/task.rs`, `commands/settings.rs`, `commands/system.rs` |
| 9-11 | UI 组件 | ✅ 已完成 | `components/TaskList.tsx`, `components/AddTaskDialog.tsx`, `components/TaskDetail.tsx`, `components/SpeedChart.tsx`, `components/StatusBar.tsx`, `components/Toolbar.tsx`, `components/SettingsDialog.tsx` |
| 9-11 | 主题系统 | ✅ 已完成 | `styles/globals.css`（CSS 变量亮色/暗色主题） |
| 9-11 | 应用主入口 | ✅ 已完成 | `App.tsx`, `main.tsx` |
| 12-14 | Windows 打包 | ⏳ 待开始 | 需执行 `npm install && npm run tauri build` |

**0→1 阶段总体进度：约 90%**（代码框架完成，BT 引擎核心下载待集成 librqbit API，打包发布待执行）

---

### 1 → 2 全协议覆盖阶段

| 周 | 任务模块 | 状态 | 完成文件 |
|---|---|---|---|
| 15-18 | ed2k 引擎框架 | ✅ 已完成 | `engine/ed2k/mod.rs`（链接解析、引擎主体、下载框架） |
| 15-18 | ed2k hash 计算 | ✅ 已完成 | `engine/ed2k/hash.rs`（MD4 分块 hash、AICH hash 框架） |
| 15-18 | ed2k 协议定义 | ✅ 已完成 | `engine/ed2k/proto.rs`（操作码、包结构、登录/搜索/源查找包构建） |
| 15-18 | ed2k Tag 系统 | ✅ 已完成 | `engine/ed2k/tag.rs`（Tag 类型、序列化/反序列化、文件 Tag） |
| 15-18 | ed2k 服务器通信 | ✅ 已完成 | `engine/ed2k/server.rs`（TCP 连接、登录、搜索、源查找、server.met 解析） |
| 15-18 | ed2k 文件传输 | ✅ 已完成 | `engine/ed2k/transfer.rs`（握手、文件请求、分片下载、hashset、队列） |
| 15-18 | ed2k KAD 网络 | ✅ 已完成 | `engine/ed2k/kad.rs`（128位 ID、k-bucket 路由表、XOR 距离、节点查找） |
| 19-20 | Chrome 扩展 | ✅ 已完成 | `extension/manifest.json`（MV3）、`extension/background.js`（下载拦截、右键菜单、与下载器通信） |
| 19-20 | 内容脚本 | ✅ 已完成 | `extension/content.js`（视频嗅探、m3u8/mpd 检测、批量链接获取、下载按钮注入） |
| 19-20 | 扩展弹窗 | ✅ 已完成 | `extension/popup.html`, `extension/popup.js`（资源列表、手动添加、批量下载、拦截开关） |
| 21-22 | JSON-RPC API | ✅ 已完成 | `api/mod.rs`（请求/响应结构、事件类型、API 服务框架） |
| 21-22 | RPC 方法分发 | ✅ 已完成 | `api/rpc.rs`（aria2 兼容 API：addUri/tellStatus/tellActive/getGlobalStat 等） |
| 21-22 | WebSocket 推送 | ✅ 已完成 | `api/websocket.rs`（连接管理、事件广播、任务状态推送） |
| 21-22 | WebUI 前端 | ✅ 已完成 | `src-webui/App.tsx`（响应式布局、任务列表、添加任务、WebSocket 实时更新） |
| 21-22 | WebUI RPC 客户端 | ✅ 已完成 | `src-webui/lib/rpc-api.ts`（JSON-RPC 封装、WebSocket 事件订阅） |
| 23-24 | RSS 引擎 | ✅ 已完成 | `rss/mod.rs`（订阅管理、轮询、OPML 导入/导出） |
| 23-24 | RSS Feed 解析 | ✅ 已完成 | `rss/feed.rs`（RSS 2.0/Atom 解析、OPML 解析） |
| 23-24 | RSS 过滤规则 | ✅ 已完成 | `rss/rules.rs`（正则/通配符/包含匹配、取反） |
| 23-24 | 自动解压 | ✅ 已完成 | `archive/mod.rs`（ZIP/TAR/TAR.GZ 解压、密码尝试、格式检测） |
| 23-24 | 密码管理 | ✅ 已完成 | `archive/password.rs`（密码条目、域名/路径关联、使用统计） |
| 23-24 | 批量下载 | ✅ 已完成 | 前端 `components/` 中已有基础框架（通配符/正则生成待集成） |
| 25-26 | 跨平台打包 | ⏳ 待开始 | macOS/Linux 打包配置 |

**1→2 阶段总体进度：约 85%**（ed2k 核心协议框架完成，服务器/KAD 实际连接待测试；浏览器扩展和 WebUI 框架完成，需与 Rust 后端联调；RSS/Archive 模块完成，需集成到 Tauri 命令层）

---

### 2 → 3 生态平台化阶段

| 周 | 任务模块 | 状态 | 完成文件 |
|---|---|---|---|
| 27-28 | HLS 引擎主体 | ✅ 已完成 | `engine/hls/mod.rs`（播放列表解析、分辨率选择、分片下载、直播检测） |
| 27-28 | m3u8 解析器 | ✅ 已完成 | `engine/hls/m3u8.rs`（Master/Media Playlist 解析、加密分片、DASH MPD） |
| 27-28 | 分片合并 | ✅ 已完成 | `engine/hls/merge.rs`（TS 二进制合并、ffmpeg 可选合并） |
| 29-31 | 插件管理器 | ✅ 已完成 | `plugin/mod.rs`（插件扫描/安装/卸载/启用/禁用、插件信息） |
| 29-31 | 插件 API | ✅ 已完成 | `plugin/api.rs`（PluginApi/HostApi trait、TaskParams/TaskInfo 结构） |
| 29-31 | WASM 加载器 | ✅ 已完成 | `plugin/loader.rs`（WASM 验证、沙箱配置、内存限制、函数调用框架） |
| 29-31 | 插件管理 UI | ✅ 已完成 | `components/PluginManager.tsx`（插件列表、安装/卸载/启用/禁用） |
| 29-31 | RSS 管理 UI | ✅ 已完成 | `components/RssManager.tsx`（订阅列表、添加/删除、OPML 导入/导出） |
| 29-31 | 批量导入 UI | ✅ 已完成 | `components/BatchImportDialog.tsx`（文本导入、剪贴板、通配符生成、去重） |
| 29-31 | 解压设置 UI | ✅ 已完成 | `components/ArchiveDialog.tsx`（自动解压开关、密码管理、格式支持） |
| 29-31 | RSS Tauri 命令 | ✅ 已完成 | `commands/rss.rs`（添加/删除/获取订阅、OPML 导入/导出） |
| 29-31 | 解压 Tauri 命令 | ✅ 已完成 | `commands/archive.rs`（配置管理、手动解压） |
| 29-31 | 插件 Tauri 命令 | ✅ 已完成 | `commands/plugin.rs`（插件列表/安装/卸载/启用/禁用） |
| 32-34 | 性能优化 | ✅ 已完成 | Cargo.toml release profile、SQLite PRAGMA 优化、数据库索引、语句缓存、批量操作 |
| 35-36 | 全面测试 | ✅ 已完成 | 193 测试全通过（格式化/状态管理/工具函数/业务逻辑），TEST_REPORT.md |

**2→3 阶段总体进度：约 98%**（HLS/插件/RSS/Archive 模块代码完成，WASM 插件加载器已集成 wasmtime，性能优化和测试已完成；剩余跨平台打包配置）

---

### 迭代 18：WebUI 增强与测试扩展

| 任务 | 状态 | 说明 |
|---|---|---|
| WebUI 全功能任务管理 | ✅ 已完成 | 添加暂停/恢复/删除操作按钮、全局暂停/恢复/清除按钮、任务详情面板（ID/状态/大小/速度/连接数/ETA/目录） |
| WebUI 标签页切换 | ✅ 已完成 | 下载中/等待中/已完成三个标签页，显示各状态任务数量 |
| WebUI 状态图标与进度条 | ✅ 已完成 | 彩色状态指示点（蓝下载/黄暂停/绿完成/红错误/紫做种）、进度条颜色随状态变化 |
| WebUI 内联 SVG 图标 | ✅ 已完成 | 替换 emoji 为 SVG 图标（暂停/播放/删除），保持一致的视觉风格 |
| 安全性 URL 验证测试 | ✅ 已完成 | 新增 12 个安全测试：拒绝 vbscript/chrome/about/blob 协议，接受 IPv4/IPv6/端口/认证 URL |
| 格式化边界测试扩展 | ✅ 已完成 | 新增 formatSpeed/formatSize 边界测试（1 B/s、GB/s、负数速度等） |
| 测试报告 | ✅ 已完成 | TEST_REPORT.md：210 测试全通过（原 193 → 新增 17） |

**变更文件**：`src-webui/App.tsx`, `src/__tests__/format.test.ts`, `TEST_REPORT.md`

**测试统计**：210 测试全通过（原 193 → 新增 17）

---

### 迭代 20：ed2k KAD、AICH hash、WASM 插件

| 任务 | 状态 | 说明 |
|---|---|---|
| ed2k KAD 引导流程 | ✅ 已完成 | UDP 引导请求/响应解析，节点自动添加到路由表 |
| ed2k KAD 迭代查找 | ✅ 已完成 | α=3 并发查询，最多 10 轮迭代，XOR 距离收敛 |
| ed2k KAD 关键词搜索 | ✅ 已完成 | KADEMLIA2_SEARCH_KEY_REQ 实现，MD4 hash 定位 |
| ed2k KAD 源搜索 | ✅ 已完成 | KADEMLIA2_SEARCH_SOURCE_REQ 实现 |
| AICH hash (SHA1 Merkle) | ✅ 已完成 | 180KB 子块、SHA1 Merkle Tree、5 个新增测试 |
| WASM 插件加载器 | ✅ 已完成 | wasmtime 29 运行时集成、内存/fuel 限制、宿主函数 |
| 浏览器扩展图标 | ✅ 已完成 | 生成 16/48/128px PNG 图标 |
| 测试报告 | ✅ 已完成 | TEST_REPORT.md：244 测试全通过 |

**变更文件**：`engine/ed2k/kad.rs`, `engine/ed2k/hash.rs`, `plugin/loader.rs`, `Cargo.toml`, `extension/icons/icon16.png`, `extension/icons/icon48.png`, `extension/icons/icon128.png`, `TEST_REPORT.md`

---

### 迭代 19：CSS 修复与扩展测试

| 任务 | 状态 | 说明 |
|---|---|---|
| 修复 CSS 构建错误 | ✅ 已完成 | 修复 `@apply bg-text-muted/30` 和 `@apply bg-accent/20`，替换为原生 CSS 属性（自定义 CSS 变量非 Tailwind 颜色） |
| 扩展工具函数测试 | ✅ 已完成 | 新增 34 个测试：URL 检测（14种文件类型）、文件名提取（6场景）、格式/标签/样式类 |
| 测试报告更新 | ✅ 已完成 | TEST_REPORT.md：244 测试全通过（原 210 → 新增 34） |

**变更文件**：`src/styles/globals.css`, `src/__tests__/extension.test.ts`, `TEST_REPORT.md`

**测试统计**：244 测试全通过（原 210 → 新增 34）

---

### 迭代 13：UI 状态一致性与测试增强

| 任务 | 状态 | 说明 |
|---|---|---|
| 修复 seeding 状态 UI 一致性 | ✅ 已完成 | 上下文菜单、工具栏、任务列表速度/ETA 显示、详情面板日志、空格键快捷键均正确处理 seeding 状态 |
| 修复速度历史全局追踪 | ✅ 已完成 | `taskStore.applyUpdate` 现在记录所有活跃任务的速度总和而非单个事件的速度 |
| 修复 formatEta 小数秒显示 | ✅ 已完成 | 使用 `Math.floor` 避免显示 `1m30.7s` 这样的小数秒 |
| 增强测试覆盖 | ✅ 已完成 | 新增 formatEta 小数/大值测试、speedHistory 全局追踪和节流测试（49 测试全通过） |

**变更文件**：`App.tsx`, `TaskList.tsx`, `Toolbar.tsx`, `TaskDetail.tsx`, `format.ts`, `taskStore.ts`, `format.test.ts`, `taskStore.test.ts`

---

### 迭代 14：拖拽种子文件与错误重试

| 任务 | 状态 | 说明 |
|---|---|---|
| 拖拽 .torrent 文件支持 | ✅ 已完成 | 使用 Tauri 原生 `onDragDropEvent` API，支持拖入 .torrent 文件自动添加任务，带视觉拖拽指示器 |
| 错误任务重试按钮 | ✅ 已完成 | 上下文菜单新增"重试"选项（RefreshCw 图标），对 error 状态任务调用 resumeTask |
| 速度图表优化 | ✅ 已完成 | SpeedChart 已正确显示下载/上传双线，speedHistory 已在迭代 13 中修复为全局追踪 |

**变更文件**：`App.tsx`, `TaskList.tsx`

---

### 迭代 15：键盘快捷键、批量复制与链接验证

| 任务 | 状态 | 说明 |
|---|---|---|
| Ctrl+N 快捷键 | ✅ 已完成 | 全局 Ctrl+N 打开添加任务对话框 |
| 批量复制链接 | ✅ 已完成 | 右键菜单"复制链接"支持多选任务，换行分隔复制所有选中任务的 URL |
| URL 验证 | ✅ 已完成 | `isValidDownloadUrl()` 验证 HTTP/HTTPS/FTP/magnet/ed2k/HLS/DASH/torrent 链接格式 |
| 测试增强 | ✅ 已完成 | 新增 12 个测试（URL 验证 + 格式化边缘情况），总计 61 测试全通过 |

**变更文件**：`App.tsx`, `TaskList.tsx`, `AddTaskDialog.tsx`, `format.ts`, `format.test.ts`

---

### 迭代 16：性能优化

| 任务 | 状态 | 说明 |
|---|---|---|
| Cargo.toml Release Profile | ✅ 已完成 | `strip=true`, `lto=true`, `codegen-units=1`, `opt-level="s"`, `panic="abort"` |
| SQLite 性能优化 | ✅ 已完成 | WAL 模式、`synchronous=NORMAL`、8MB 页缓存、`temp_store=MEMORY`、256MB mmap、`busy_timeout=5000` |
| 数据库索引优化 | ✅ 已完成 | 新增 `state_priority`、`protocol`、`task_files_task_id` 复合索引 |
| 语句缓存 | ✅ 已完成 | 高频查询使用 `prepare_cached` 减少编译开销 |
| 批量操作 | ✅ 已完成 | `batch_update_state` 批量状态更新、`archive_completed_tasks` 归档旧任务 |
| 数据库维护 | ✅ 已完成 | `maintenance()` 清理孤立记录 + VACUUM、`get_db_size()` 大小查询 |

**变更文件**：`Cargo.toml`, `storage/db.rs`

---

### 迭代 17：全面测试

| 任务 | 状态 | 说明 |
|---|---|---|
| 新增 utils.test.ts | ✅ 已完成 | 41 测试：URL 检测、状态转换、进度计算、ETA 计算、优先级排序 |
| 扩展 format.test.ts | ✅ 已完成 | 新增 12 测试：边界值、大时长、小数秒、TB 级格式化 |
| 扩展 taskStore.test.ts | ✅ 已完成 | 新增 10 测试：getTaskById、做种状态、大数据集选择/排序、多选删除、速度历史 |
| 测试报告 | ✅ 已完成 | TEST_REPORT.md：193 测试全通过，覆盖格式化/状态管理/工具函数/业务逻辑 |

**变更文件**：`format.test.ts`, `taskStore.test.ts`, `utils.test.ts`（新增）, `TEST_REPORT.md`（新增）

**测试统计**：193 测试全通过（原 130 → 新增 63）

---

## 阶段总览

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│   0 → 1     │     │   1 → 2     │     │   2 → 3     │
│   MVP       │────▶│   完善       │────▶│   生态       │
│             │     │             │     │             │
│ HTTP+BT     │     │ ed2k+扩展    │     │ 插件+流媒体   │
│ 核心下载器   │     │ 全协议覆盖   │     │ 平台化       │
│             │     │             │     │             │
│ 14 周       │     │ 12 周       │     │ 10 周       │
└─────────────┘     └─────────────┘     └─────────────┘
```

| 阶段 | 目标 | 周期 | 团队规模 | 交付物 |
|---|---|---|---|---|
| **0 → 1** | 可用的 BT + HTTP 下载器 | 14 周 | 1-2 人 | 桌面应用（Windows 优先） |
| **1 → 2** | 全协议 + 浏览器集成 + 远程管理 | 12 周 | 2-3 人 | 跨平台应用 + 浏览器扩展 |
| **2 → 3** | 插件生态 + 流媒体 + 平台化 | 10 周 | 2-3 人 | 完整产品 + 插件市场 |

---

## 0 → 1：MVP（14 周）

> **目标**：交付一个可日常使用的 BT + HTTP/HTTPS/FTP 下载器，具备便携、无广告、断点续传等核心体验。

### 交付边界

**包含**：
- HTTP/HTTPS/FTP 多线程分块下载、断点续传
- BitTorrent .torrent / magnet 下载、DHT、做种
- 任务管理（CRUD、暂停/恢复、优先级、并发控制）
- SQLite 持久化、崩溃恢复
- 基础 UI（任务列表、添加对话框、详情面板、速度图表）
- 设置中心（下载/连接/BT/通知）
- 绿色便携（零安装、零残留）
- 系统托盘、通知
- Windows 平台打包

**不包含**：
- ed2k 协议（→ 1→2）
- 浏览器扩展（→ 1→2）
- WebUI / JSON-RPC API（→ 1→2）
- RSS 自动下载（→ 1→2）
- 自动解压（→ 1→2）
- HLS/DASH 流媒体（→ 2→3）
- 插件系统（→ 2→3）
- macOS / Linux 打包（→ 1→2）

---

### 第 1-2 周：项目脚手架

#### 1.1 Tauri v2 项目初始化

```
任务清单：
├── npm create tauri-app@latest downloader -- --template react-ts
├── 配置 tauri.conf.json
│   ├── 窗口尺寸：1200×800，最小 800×600
│   ├── 标识符：com.downloader.app
│   ├── capabilities 权限：fs / notification / clipboard
│   └── trayIcon 配置
├── 配置前端工程
│   ├── Tailwind CSS + dark mode（selector 模式）
│   ├── Zustand 状态管理
│   ├── @tanstack/react-virtual + @tanstack/react-table
│   ├── @radix-ui 组件库（dialog / context-menu / dropdown-menu）
│   ├── Recharts 图表库
│   ├── lucide-react 图标
│   └── framer-motion 动画
├── 配置 Rust 后端工程
│   ├── Cargo.toml 依赖（见 PRD 8.7 节，仅 HTTP + BT 相关）
│   ├── tracing + tracing-subscriber 日志
│   └── rusqlite 持久化
└── 创建基础目录结构（见 PRD 3.4 节）
```

**验收标准**：`npm run tauri dev` 可启动空白窗口，Rust 后端可编译通过。

#### 1.2 数据库与配置

```
任务清单：
├── 实现 storage/db.rs
│   ├── SQLite 初始化（rusqlite, bundled 模式）
│   ├── 建表：tasks / task_files / piece_bitmaps / settings
│   ├── 数据库迁移机制（版本号 + SQL 脚本）
│   └── 连接池（r2d2 或 tokio::sync::Mutex<Connection>）
├── 实现 storage/config.rs
│   ├── TOML 配置读写（config.toml）
│   ├── 默认配置生成
│   ├── 配置热更新监听
│   └── 便携模式检测（data/ 目录在程序目录下）
└── 数据库单元测试
    ├── CRUD 操作测试
    └── 迁移脚本测试
```

**验收标准**：可创建/读取/更新/删除 tasks 记录，配置文件可读写。

#### 1.3 前端基础架构

```
任务清单：
├── stores/taskStore.ts — Zustand 任务状态
│   ├── tasks: Map<TaskId, TaskStatus>
│   ├── addTask / updateTask / removeTask
│   ├── 全局统计（总速度、活跃数）
│   └── 选择状态（selectedIds）
├── lib/tauri-api.ts — Tauri invoke 封装
│   ├── addTask / pauseTask / resumeTask / removeTask
│   ├── getTask / getAllTasks
│   └── 类型定义（TaskStatus / TaskParams / PeerInfo）
├── lib/format.ts — 格式化工具
│   ├── formatSpeed(bytes/sec) → "5.2 MB/s"
│   ├── formatSize(bytes) → "4.2 GB"
│   └── formatDuration(seconds) → "12min"
├── hooks/useTaskEvents.ts — Tauri event 订阅
│   ├── 监听 "task-update" 事件
│   ├── 100ms 节流更新 store
│   └── 自动重连机制
└── App.tsx — 主布局骨架
    ├── 顶部工具栏
    ├── 中部任务列表（占位）
    ├── 底部状态栏
    └── CSS 变量主题系统
```

**验收标准**：前端可调用 Rust 后端 IPC，store 可管理任务状态。

---

### 第 3-5 周：HTTP/HTTPS/FTP 下载引擎

#### 3.1 HTTP 下载核心

```
任务清单：
├── engine/http.rs
│   ├── 异步 HTTP 客户端（reqwest）
│   ├── Range 请求断点续传
│   │   ├── HEAD 请求获取 Content-Length 和 Accept-Ranges
│   │   ├── 分块下载（4-16 线程，按文件大小自动决定）
│   │   ├── 每个分块独立 tokio::task
│   │   └── 分块完成 → 合并写入文件（pwrite 或顺序写入）
│   ├── 自动重定向跟随（301/302/303/307/308，最多 10 层）
│   ├── 文件名提取
│   │   ├── Content-Disposition: filename*=UTF-8''...
│   │   └── URL 路径最后一段
│   ├── Cookie 处理（reqwest::cookie_store）
│   ├── 代理支持（HTTP / SOCKS5）
│   │   ├── reqwest::Proxy 配置
│   │   └── 代理认证（用户名/密码）
│   └── 速度限制
│       ├── 令牌桶算法（tokio::sync::Semaphore 或 governor crate）
│       ├── 单任务限速
│       └── 全局限速（共享令牌桶）
├── engine/ftp.rs
│   ├── FTP 被动模式（PASV）
│   ├── 目录列表（LIST / NLST，解析 Unix/Windows 格式）
│   ├── 断点续传（REST 命令）
│   └── 集成到统一 DownloadEngine trait
└── 进度回调
    ├── 每个分块完成后更新 downloaded 字节
    ├── 100ms 节流推送到前端
    └── 速度计算（滑动窗口，最近 5 秒平均）
```

**验收标准**：
- 可下载 HTTP 大文件（>4GB），支持断点续传
- 可从 FTP 服务器下载文件
- 代理（HTTP/SOCKS5）可用
- 限速功能生效

#### 3.2 任务调度引擎

```
任务清单：
├── engine/task_manager.rs
│   ├── trait DownloadEngine（见 PRD 3.3.1）
│   ├── EngineRegistry — 引擎注册表
│   │   ├── register(Protocol, Box<dyn DownloadEngine>)
│   │   └── get(Protocol) → &dyn DownloadEngine
│   ├── 任务状态机
│   │   ├── Queued → Downloading → Done
│   │   ├── Downloading → Paused → Downloading
│   │   ├── Downloading → Error（自动重试）
│   │   └── 状态变更 → emit event 到前端
│   ├── 并发控制
│   │   ├── 全局信号量（tokio::sync::Semaphore, max_concurrent_tasks）
│   │   ├── 等待队列（优先级排序）
│   │   └── 任务完成 → 自动唤醒下一个等待任务
│   ├── 自动重试
│   │   ├── 可配置重试次数（默认 3）
│   │   ├── 指数退避（5s, 10s, 20s...）
│   │   └── 网络超时 / 连接拒绝 → 重试
│   └── 磁盘空间检测
│       ├── 下载前检查剩余空间
│       └── 空间不足 → 暂停 + 通知
├── commands/task.rs — Tauri IPC 命令
│   ├── add_task(params) → TaskId
│   ├── pause_task(id)
│   ├── resume_task(id)
│   ├── remove_task(id, delete_files)
│   ├── get_task(id) → TaskStatus
│   ├── get_all_tasks() → Vec<TaskStatus>
│   └── set_priority(id, priority)
└── 断点续传持久化
    ├── 下载中：每 5 秒将 piece_bitmaps 写入 SQLite
    ├── 程序重启：从 SQLite 恢复未完成任务
    └── 校验已有数据后从断点继续
```

**验收标准**：
- 可同时下载 3 个任务（可配置 1-20）
- 暂停/恢复后从断点继续
- 程序重启后自动恢复未完成任务
- 磁盘满时自动暂停并通知

---

### 第 6-8 周：BitTorrent 引擎

#### 6.1 librqbit 集成

```
任务清单：
├── Cargo.toml 添加 librqbit 依赖
│   └── features = ["dht", "upnp", "peer-info"]
├── engine/bt/mod.rs — BT 引擎入口
│   ├── BtEngine 结构体
│   │   ├── session: librqbit::Session（全局唯一）
│   │   ├── handles: HashMap<TaskId, TorrentHandle>
│   │   └── event_tx: broadcast::Sender<DownloadEvent>
│   └── 实现 DownloadEngine trait
├── engine/bt/session.rs — Session 封装
│   ├── Session 初始化（监听端口、DHT 配置）
│   ├── add_torrent(torrent_bytes, save_path) → TaskId
│   ├── add_magnet(magnet_uri, save_path) → TaskId
│   ├── pause / resume / remove
│   ├── get_status → TaskStatus（librqbit 状态转换）
│   └── get_peers → Vec<PeerInfo>
├── 进度回调
│   ├── 监听 librqbit 状态变化
│   ├── 100ms 节流推送到 Task Engine
│   └── 速度 / 进度 / peer 数 / ETA 计算
└── .torrent 文件解析
    ├── 拖拽 .torrent 文件到窗口
    ├── 解析文件列表（单文件 / 多文件）
    ├── 文件树展示（可勾选子文件）
    └── magnet 链接 → 元数据获取 → 文件列表展示
```

**验收标准**：
- 可添加 .torrent 文件下载
- 可添加 magnet 链接下载
- 多文件 torrent 可选择性下载
- DHT 网络可连接

#### 6.2 BT 高级功能

```
任务清单：
├── DHT / PEX / LSD 验证
│   ├── DHT 节点数显示（状态栏）
│   ├── PEX 自动交换 peer
│   ├── LSD 局域网发现
│   └── DHT 状态指示（已连接 / 未连接）
├── Tracker 管理
│   ├── 内置公共 tracker 列表（trackers_best.txt）
│   ├── 手动添加 / 删除 tracker
│   ├── tracker 状态显示（工作 / 超时 / 错误）
│   └── 自动合并 magnet 链接中的 tracker
├── 做种（Seeding）
│   ├── 下载完成 → 自动做种
│   ├── 做种比率限制（默认 2.0）
│   ├── 做种时间限制（默认 1440 分钟）
│   └── 做种状态显示（🌱 图标）
├── 加密策略
│   ├── MSE/PE 支持
│   ├── 三种模式：禁用 / 启用 / 强制
│   └── 设置界面配置
├── Peer 管理
│   ├── Peer 列表（IP / 客户端 / 速度 / 已交换数据）
│   ├── 单个 Peer 封禁
│   └── Peer 来源标识（DHT / PEX / Tracker / Incoming）
└── 限速
    ├── 单任务上传/下载限速
    └── 全局上传/下载限速
```

**验收标准**：
- magnet 链接可在 30 秒内获取元数据
- 做种功能正常工作
- Peer 列表可查看和封禁
- 加密策略切换生效

---

### 第 9-11 周：UI 完整实现

#### 9.1 任务列表与操作

```
任务清单：
├── components/TaskList.tsx
│   ├── @tanstack/react-virtual 虚拟滚动
│   ├── @tanstack/react-table 表格
│   ├── 列定义
│   │   ├── 状态（图标：▶ ⏸ ⏳ ✅ ❌ 🌱）
│   │   ├── 文件名（省略号截断，tooltip 完整名称）
│   │   ├── 协议标签（HTTP / BT / ed2k 颜色区分）
│   │   ├── 大小（格式化）
│   │   ├── 进度（内嵌进度条 + 百分比文字）
│   │   ├── 速度（下载 / 上传）
│   │   ├── 来源（peer 数）
│   │   └── 剩余时间（ETA）
│   ├── 按列排序（点击列头切换升降序）
│   ├── 多选（Ctrl+Click / Shift+Click / Ctrl+A）
│   └── 拖拽排序（@dnd-kit/sortable）
├── components/AddTaskDialog.tsx
│   ├── 链接输入框（支持粘贴多个链接，每行一个）
│   ├── 协议自动识别（magnet / ed2k / http / ftp）
│   ├── 保存目录选择
│   ├── 文件名自动填充（可编辑）
│   ├── 高级选项折叠面板
│   │   ├── 代理选择
│   │   ├── 限速设置
│   │   └── 添加后立即开始 ☑
│   └── .torrent 文件拖拽 → 自动弹出对话框
├── 右键菜单
│   ├── 开始 / 暂停 / 删除
│   ├── 重新校验
│   ├── 打开文件 / 打开目录
│   ├── 任务属性
│   ├── 设置优先级（高 / 普通 / 低）
│   └── 复制链接
└── 剪贴板监听
    ├── 定时检查剪贴板（1 秒间隔）
    ├── 识别 magnet / ed2k / http(s) 链接
    └── 弹出"检测到下载链接，是否添加？"提示
```

**验收标准**：
- 1000+ 任务列表流畅滚动（60fps）
- 添加/暂停/恢复/删除操作正常
- 右键菜单功能完整
- 剪贴板监听可用

#### 9.2 详情面板

```
任务清单：
├── components/TaskDetail.tsx
│   ├── 概要 Tab
│   │   ├── 任务名称、保存路径、总大小、已下载
│   │   ├── 分享比率（BT）
│   │   ├── Info Hash / ed2k Hash
│   │   └── 创建时间、完成时间
│   ├── 文件 Tab
│   │   ├── 文件树（目录结构 + 文件列表）
│   │   ├── 文件大小、下载进度
│   │   └── 文件优先级设置（跳过 / 正常 / 高）
│   ├── 连接 Tab
│   │   ├── Peer 列表（IP / 客户端 / 速度 / 已交换）
│   │   └── 封禁操作
│   ├── Tracker Tab
│   │   ├── Tracker 列表（地址 / 状态 / peer 数）
│   │   └── 手动添加 / 删除
│   └── 日志 Tab
│       ├── 事件日志（时间戳 + 事件描述）
│       └── 自动滚动 + 手动暂停
├── 面板展开/收起（选中任务时展开）
└── 面板高度拖拽调整
```

**验收标准**：选中任务后详情面板正确显示各 Tab 信息。

#### 9.3 速度图表与状态栏

```
任务清单：
├── components/SpeedChart.tsx
│   ├── Recharts 折线图
│   ├── 双 Y 轴（下载速度蓝色 / 上传速度绿色）
│   ├── 时间轴切换（5 分钟 / 30 分钟 / 1 小时）
│   ├── 环形缓冲区存储数据点（每秒一个点）
│   ├── 鼠标悬停显示具体数值
│   └── 主题适配（亮色/暗色）
├── components/StatusBar.tsx
│   ├── 全局下载速度（▲ 5.2 MB/s）
│   ├── 全局上传速度（▼ 1.1 MB/s）
│   ├── 活跃任务数 / 任务总数
│   ├── DHT 网络状态（节点数 / 已连接 / 未连接）
│   └── 磁盘空间提示
└── 系统托盘
    ├── 最小化到托盘
    ├── 托盘右键菜单（显示 / 开始全部 / 暂停全部 / 退出）
    ├── 托盘图标显示下载状态
    └── 双击托盘显示/隐藏窗口
```

**验收标准**：速度图表实时更新，托盘功能正常。

---

### 第 12-13 周：设置中心与辅助功能

#### 12.1 设置中心

```
任务清单：
├── components/SettingsDialog.tsx
│   ├── 左侧分类导航 + 右侧表单
│   ├── 常规设置
│   │   ├── 语言（中文 / English，i18n 框架）
│   │   ├── 主题（亮色 / 暗色 / 跟随系统）
│   │   ├── 字体大小
│   │   ├── 最小化到托盘
│   │   └── 关闭时行为（最小化 / 退出）
│   ├── 下载设置
│   │   ├── 默认下载目录
│   │   ├── 完成目录
│   │   ├── 临时文件目录
│   │   ├── 最大并发任务数（1-20）
│   │   ├── 单任务最大连接数（8-256）
│   │   ├── 全局最大连接数（50-500）
│   │   ├── 速度限制（上传 / 下载）
│   │   ├── 自动重试次数 / 间隔
│   │   └── 完成后动作（无 / 关机 / 休眠 / 锁屏）
│   ├── 连接设置
│   │   ├── BT 监听端口
│   │   ├── UPnP / NAT-PMP
│   │   ├── 代理设置（类型 / 地址 / 认证）
│   │   ├── IP 过滤（ipfilter.dat 导入）
│   │   └── 超时设置
│   ├── BT 设置
│   │   ├── DHT / PEX / LSD 开关
│   │   ├── 加密策略
│   │   ├── 做种比率 / 时间限制
│   │   └── Tracker 列表管理
│   ├── 通知设置
│   │   ├── 完成通知 / 错误通知开关
│   │   ├── 通知声音
│   │   └── 通知持续时间
│   └── 高级设置
│       ├── 日志级别
│       ├── 日志文件大小 / 保留天数
│       └── 临时文件清理策略
└── commands/settings.rs — 配置读写 IPC
```

**验收标准**：所有设置项可修改并持久化，重启后生效。

#### 12.2 主题系统

```
任务清单：
├── CSS 变量主题（见 PRD 3.9.1）
│   ├── :root / [data-theme="light"] 亮色变量
│   ├── [data-theme="dark"] 暗色变量
│   └── Tailwind CSS 映射
├── 主题切换
│   ├── 手动切换（亮色 / 暗色）
│   ├── 跟随系统（matchMedia 监听）
│   └── 切换动画（200ms transition）
└── 主题持久化（保存到 config.toml）
```

**验收标准**：亮色/暗色主题正确切换，无样式错乱。

---

### 第 14 周：打包与发布

#### 14.1 Windows 打包

```
任务清单：
├── tauri build 配置
│   ├── 输出：便携 ZIP + NSIS 安装包
│   ├── 包体大小目标：< 15 MB（不含 WebView2 Runtime）
│   └── 图标（ico / png 多尺寸）
├── CI/CD
│   ├── GitHub Actions workflow
│   │   ├── push tag → 触发构建
│   │   ├── Windows runner 构建
│   │   └── 上传到 GitHub Release
│   └── 自动更新
│       ├── tauri-plugin-updater 配置
│       └── latest.json 生成
├── 测试
│   ├── HTTP/HTTPS 下载测试（小文件、大文件、断点续传）
│   ├── FTP 下载测试
│   ├── BT .torrent 下载测试
│   ├── BT magnet 下载测试
│   ├── 任务 CRUD 测试
│   ├── 设置保存/加载测试
│   ├── 崩溃恢复测试
│   └── 托盘功能测试
└── 文档
    ├── README.md（功能介绍、截图、下载链接）
    └── CHANGELOG.md
```

**验收标准**：Windows 10/11 上可正常安装/运行，所有核心功能可用。

---

### 0→1 里程碑验收清单

| 验收项 | 通过标准 |
|---|---|
| HTTP 下载 | 支持多线程分块、断点续传、重定向、代理 |
| FTP 下载 | 被动模式、目录列表、断点续传 |
| BT 下载 | .torrent + magnet、DHT、选择性下载 |
| 做种 | 自动做种、比率/时间限制 |
| 任务管理 | CRUD、暂停/恢复、优先级、并发控制 |
| 持久化 | SQLite 存储、崩溃自动恢复 |
| UI | 任务列表虚拟滚动、详情面板、速度图表 |
| 设置 | 下载/连接/BT/通知设置完整 |
| 便携 | 零安装、零残留、单目录运行 |
| 托盘 | 最小化到托盘、右键菜单 |
| 打包 | Windows ZIP + NSIS 可用 |

---

## 1 → 2：全协议覆盖（12 周）

> **目标**：补充 ed2k 协议、浏览器集成、远程管理、RSS、自动解压，实现"全协议下载器"定位。

### 交付边界

**新增**：
- ed2k 协议（服务器/KAD/源交换/下载）
- 浏览器扩展（Chrome + Firefox，链接捕获 + 视频嗅探）
- 远程管理 WebUI
- JSON-RPC API
- RSS 自动下载
- 自动解压
- 批量导入（通配符/正则/文本文件）
- 定时下载（带宽计划/完成后动作增强）
- 任务搜索与过滤、标签系统
- 下载历史与统计
- macOS + Linux 打包

---

### 第 15-18 周：ed2k 引擎

#### 15.1 ed2k 基础模块

```
任务清单：
├── engine/ed2k/hash.rs
│   ├── MD4 hash 计算（md4 crate）
│   ├── ed2k 分块 hash（9.28 MB per chunk）
│   ├── 多块文件：先计算每块 MD4，再对拼接结果 MD4
│   ├── AICH hash（180 KB 子块，可选）
│   └── 单元测试（与 eMule 计算结果比对）
├── engine/ed2k/proto.rs
│   ├── TCP 包头结构（6 字节：0xE3 + 4字节长度 + 操作码）
│   ├── 协议标识（0xE3 / 0xC5 / 0xD4 / 0xE4 / 0xE5）
│   ├── 服务器通信操作码（C2S / S2C）
│   ├── 客户端通信操作码（C2C）
│   ├── Tag 系统（Hash / String / Integer / Float / Bool）
│   └── 包序列化/反序列化（bytes crate）
├── engine/ed2k/tag.rs
│   ├── Tag 类型定义
│   ├── Tag 编码/解码
│   └── 常见文件 Tag（FT_FILENAME / FT_FILESIZE / FT_FILETYPE）
└── 协议包解析单元测试
```

#### 15.2 ed2k 服务器通信

```
任务清单：
├── engine/ed2k/server.rs
│   ├── TCP 连接管理（tokio::net::TcpStream）
│   ├── 登录流程
│   │   ├── OP_LOGINREQUEST → OP_SERVERIDENT + OP_IDCHANGE
│   │   ├── HighID / LowID 识别
│   │   └── 用户名发送
│   ├── 服务器列表管理
│   │   ├── server.met 文件解析
│   │   ├── OP_GETSERVERLIST 获取新服务器
│   │   └── 服务器状态维护（在线 / 离线 / 延迟）
│   ├── 文件搜索
│   │   ├── OP_SEARCHREQUEST → OP_SEARCHRESULT
│   │   ├── 搜索条件编码（文件名 / 大小 / 类型）
│   │   └── 搜索结果解析
│   ├── 源查找
│   │   ├── OP_GETSOURCES → OP_FOUNDSOURCES
│   │   └── 源信息解析（IP / 端口 / Hash）
│   └── 心跳保活
│       └── 定时发送 OP_CALLBACKREQUEST
└── 集成测试（连接真实 ed2k 服务器）
```

#### 15.3 ed2k 文件传输

```
任务清单：
├── engine/ed2k/transfer.rs
│   ├── 客户端握手
│   │   ├── OP_HELLO → OP_HELLOANSWER
│   │   ├── 交换 Hash / IP / Port / Username
│   │   └── eMule 扩展信息
│   ├── 文件请求流程
│   │   ├── OP_REQUESTFILENAME → OP_REQFILENAMEANSWER
│   │   ├── OP_SETREQFILEID
│   │   ├── OP_HASHSETREQUEST → OP_HASHSETANSWER
│   │   └── OP_STARTUPLOADREQ → OP_ACCEPTUPLOADREQ
│   ├── 分片下载
│   │   ├── OP_REQUESTPARTS（3 个偏移量）
│   │   ├── OP_SENDINGPART（数据接收）
│   │   ├── 分片校验（ed2k hash）
│   │   └── 失败重传
│   ├── 队列管理
│   │   ├── OP_QUEUERANK 排队
│   │   ├── 等待 → 下载状态切换
│   │   └── 多源并行下载
│   └── 上传管理
│       ├── OP_QUEUERANKING 排队通知
│       ├── 上传槽管理
│       └── 上传限速
└── 下载集成测试（ed2k 链接下载完整文件）
```

#### 15.4 ed2k KAD 网络

```
任务清单：
├── engine/ed2k/kad.rs
│   ├── 128 位 ID 空间
│   ├── k-bucket 路由表（k=10）
│   │   ├── 节点添加/更新/淘汰
│   │   └── XOR 距离度量
│   ├── 引导流程
│   │   ├── KADEMLIA2_BOOTSTRAP_REQ / RES
│   │   └── 初始节点列表（server.met 中的节点）
│   ├── 迭代式节点查找（α=3 并发）
│   ├── 关键词搜索（KADEMLIA2_SEARCH_KEY_REQ）
│   ├── 源搜索（KADEMLIA2_SEARCH_SOURCE_REQ）
│   ├── 防火墙检测（KADEMLIA_FIREWALLED2_REQ）
│   └── KAD 状态显示（节点数 / 已连接）
├── engine/ed2k/source.rs — 源交换
│   ├── OP_REQUESTSOURCES → OP_ANSWERSOURCES
│   └── 增加下载源
├── engine/ed2k/credit.rs — 信用系统
│   ├── 上传/下载比率记录
│   └── 信用数据持久化
├── engine/ed2k/obfuscation.rs — 协议混淆
│   ├── Diffie-Hellman 密钥交换
│   ├── RC4 流加密
│   └── 三种模式（禁用 / 启用 / 强制）
└── engine/ed2k/mod.rs — 引擎集成
    ├── 实现 DownloadEngine trait
    ├── 集成到 EngineRegistry
    ├── 状态转换为 TaskStatus
    └── ed2k 设置 UI
```

**验收标准**：
- 可连接 ed2k 服务器并获取 HighID
- 可通过 ed2k 链接下载文件
- KAD 网络可连接并搜索
- 协议混淆可用

---

### 第 19-20 周：浏览器扩展 + 视频嗅探

#### 19.1 Chrome 扩展

```
任务清单：
├── extension/manifest.json（Chrome MV3）
│   ├── permissions: ["activeTab", "downloads", "contextMenus", "storage"]
│   ├── background: Service Worker
│   └── content_scripts: 视频嗅探
├── extension/background.js
│   ├── 监听浏览器下载请求（chrome.downloads.onDeterminingFilename）
│   ├── 拦截规则配置（全部 / 特定文件类型 / 特定域名）
│   ├── 转发到下载器（HTTP POST localhost:port 或 Native Messaging）
│   └── 右键菜单注册
├── extension/content.js
│   ├── 视频嗅探
│   │   ├── 检测 <video> / <audio> 标签的 src
│   │   ├── 检测 .m3u8 / .mpd / .mp4 / .webm 资源
│   │   ├── 检测 HLS.js / dash.js 播放器实例
│   │   └── 注入下载按钮（页面内浮动按钮）
│   └── 批量链接捕获
│       ├── 扫描页面所有 <a> 标签
│       ├── 智能过滤（排除广告/追踪/小图标）
│       └── 弹出选择列表
├── extension/popup.html / popup.js
│   ├── 扩展弹出窗口
│   ├── 检测到的资源列表
│   ├── 一键下载按钮
│   └── 连接状态指示
└── 下载器本地 API 接收层
    ├── api/browser.rs — HTTP POST 接收
    ├── 解析链接 → 调用 addTask
    └── 认证（简单 token）
```

#### 19.2 Firefox 扩展

```
任务清单：
├── 适配 Firefox WebExtension API
│   ├── manifest.json 调整（background.scripts 替代 service_worker）
│   ├── chrome.* → browser.* API 切换
│   └── webextension-polyfill 兼容层
├── Native Messaging 配置
│   ├── native_messaging_host.json
│   └── 下载器端 Native Messaging Server
└── Chrome + Firefox 兼容性测试
```

**验收标准**：
- Chrome 扩展可拦截下载并转发到下载器
- 视频嗅探可检测页面中的 m3u8/mp4 资源
- 右键菜单"使用下载器下载"可用
- Firefox 扩展功能一致

---

### 第 21-22 周：WebUI + JSON-RPC API

#### 21.1 JSON-RPC API

```
任务清单：
├── api/rpc.rs
│   ├── JSON-RPC 2.0 协议实现
│   │   ├── 请求解析（method / params / id）
│   │   ├── 响应封装（result / error）
│   │   └── 批量请求支持
│   ├── 核心 API 方法
│   │   ├── downloader.addTask(params) → TaskId
│   │   ├── downloader.removeTask(id)
│   │   ├── downloader.pauseTask(id)
│   │   ├── downloader.resumeTask(id)
│   │   ├── downloader.tellStatus(id) → TaskStatus
│   │   ├── downloader.tellActive() → Vec<TaskStatus>
│   │   ├── downloader.tellWaiting() → Vec<TaskStatus>
│   │   ├── downloader.tellStopped() → Vec<TaskStatus>
│   │   ├── downloader.getGlobalStat() → GlobalStat
│   │   └── downloader.getVersion() → Version
│   ├── aria2 兼容 API（可选）
│   │   ├── aria2.addUri / aria2.addTorrent
│   │   ├── aria2.tellStatus / aria2.tellActive
│   │   └── aria2.pause / aria2.pauseAll
│   └── 限流（每秒最多 100 请求）
├── api/websocket.rs
│   ├── WebSocket 连接管理
│   ├── 事件订阅（downloader.subscribe）
│   │   ├── task-started / task-paused / task-completed / task-error
│   │   ├── speed-update（每秒）
│   │   └── peer-connected / peer-disconnected
│   └── 连接认证
├── api/auth.rs
│   ├── Token 认证（header: Authorization: Bearer <token>）
│   └── 用户名/密码认证（可选）
└── API 文档页面（/api/docs, 内嵌 Swagger UI 或 Redoc）
```

#### 21.2 WebUI

```
任务清单：
├── src-webui/ — 独立前端项目
│   ├── 复用 src/components/ 中的共享组件
│   ├── lib/rpc-api.ts — JSON-RPC 客户端封装
│   ├── lib/ws.ts — WebSocket 连接管理（自动重连）
│   ├── 替换 Tauri IPC 为 HTTP/WS 通信
│   └── 响应式布局适配
│       ├── 桌面（≥1024px）：完整布局
│       ├── 平板（768-1023px）：简化侧边栏
│       └── 手机（<768px）：单列布局
├── WebUI 静态资源打包
│   ├── Vite 构建 → dist/
│   ├── 嵌入 Rust 二进制（include_bytes! 或 tauri::Resource）
│   └── axum 路由：/ → index.html, /assets/* → 静态文件
├── axum HTTP 服务
│   ├── 监听端口（默认 6800，可配置）
│   ├── 路由：/api/rpc（JSON-RPC）, /ws（WebSocket）, /（WebUI）
│   ├── CORS 配置
│   └── 默认仅 localhost，可选开启远程访问
└── 认证保护
    ├── 首次访问要求设置密码
    └── 登录页面
```

**验收标准**：
- JSON-RPC API 可通过 curl 调用所有方法
- WebSocket 实时推送任务状态
- 浏览器访问 localhost:6800 可管理任务
- 手机浏览器可正常使用

---

### 第 23-24 周：RSS + 自动解压 + 批量下载

#### 23.1 RSS 自动下载

```
任务清单：
├── rss/feed.rs
│   ├── RSS/Atom feed 解析（feed-rs crate）
│   ├── Feed 订阅管理（添加/删除/编辑）
│   ├── 轮询机制（默认 30 分钟间隔）
│   └── 已处理条目记录（去重）
├── rss/rules.rs
│   ├── 过滤规则引擎
│   │   ├── 正则表达式匹配（标题 / 描述 / 链接）
│   │   ├── 多规则组合（AND / OR）
│   │   └── 规则优先级
│   ├── 下载规则
│   │   ├── 匹配 → 自动创建下载任务
│   │   ├── 按规则指定保存目录
│   │   └── 按规则指定优先级
│   └── OPML 导入/导出
├── components/RssManager.tsx
│   ├── Feed 列表（名称 / URL / 最后更新 / 条目数）
│   ├── 添加/编辑/删除 Feed
│   ├── 过滤规则管理
│   ├── 匹配历史
│   └── OPML 导入/导出按钮
└── rssStore.ts — RSS 状态管理
```

#### 23.2 自动解压

```
任务清单：
├── archive/mod.rs
│   ├── 格式支持
│   │   ├── .zip（zip crate）
│   │   ├── .tar / .tar.gz / .tar.bz2 / .tar.xz（tar + flate2）
│   │   ├── .7z（sevenz-rust crate）
│   │   └── .rar（rar crate 或调用 unrar）
│   ├── 下载完成触发
│   │   ├── 检测文件扩展名
│   │   ├── 尝试解压密码列表
│   │   └── 解压到同目录或指定目录
│   ├── 递归解压（嵌套压缩包）
│   ├── 解压后删除原文件（可选）
│   └── 解压进度回调
├── archive/password.rs
│   ├── 密码管理器
│   │   ├── 添加/删除/编辑密码
│   │   ├── 按域名/路径关联密码
│   │   └── 密码加密存储
│   └── 自动尝试匹配
└── components/ArchiveDialog.tsx
    ├── 密码管理界面
    ├── 解压设置（自动解压开关、解压后删除）
    └── 解压进度显示
```

#### 23.3 批量下载增强

```
任务清单：
├── components/BatchImportDialog.tsx
│   ├── 导入方式
│   │   ├── 从文本文件导入（每行一个 URL）
│   │   ├── 从剪贴板导入
│   │   └── 使用通配符生成（file(1-100).zip）
│   ├── 通配符 / 正则替换
│   │   ├── 数字序号（1,2,3...）
│   │   ├── 零填充（001,002,003...）
│   │   └── 字母序号（a,b,c...）
│   ├── 去重检测
│   ├── 链接有效性预检（HEAD 请求）
│   ├── 统一设置（保存目录 / 并发数）
│   └── 任务分组（整组操作：暂停/开始/删除）
└── 后端批量任务创建
```

---

### 第 25-26 周：跨平台打包与发布

#### 25.1 macOS + Linux 打包

```
任务清单：
├── macOS
│   ├── tauri build → .dmg / .app
│   ├── Apple Notarization（可选）
│   └── macOS 特定测试（菜单栏、通知、文件系统权限）
├── Linux
│   ├── tauri build → AppImage + .deb
│   ├── 依赖检查（libwebkit2gtk）
│   └── Linux 特定测试（桌面环境兼容性）
├── CI/CD 完善
│   ├── GitHub Actions matrix（Windows / macOS / Linux）
│   ├── 代码签名（Windows Authenticode）
│   └── 自动更新（三平台）
└── 跨平台测试
    ├── Windows 10/11
    ├── macOS 12+
    └── Ubuntu 20.04+ / Fedora 38+
```

#### 25.2 功能完善

```
任务清单：
├── 任务搜索与过滤
│   ├── 关键词搜索
│   ├── 状态 / 协议 / 大小 / 时间过滤
│   ├── 排序（名称 / 大小 / 进度 / 速度 / ETA）
│   └── 标签系统
├── 下载历史
│   ├── 历史列表 + 搜索
│   ├── 统计信息（总下载量 / 上传量 / 完成数）
│   ├── 流量图表（按天/周/月）
│   ├── 导出（CSV / JSON）
│   └── 清理
├── 定时下载增强
│   ├── 带宽计划（不同时段不同限速）
│   ├── 做种计划
│   └── 完成后动作增强（执行命令 / 发送通知 / 打开文件）
└── 快捷键完善（见 PRD 2.4.5）
```

**验收标准**：
- macOS / Linux 可正常运行
- 所有 1→2 阶段功能在三平台可用
- 自动更新机制正常

---

### 1→2 里程碑验收清单

| 验收项 | 通过标准 |
|---|---|
| ed2k 下载 | 服务器连接、KAD 网络、文件下载、源交换 |
| ed2k 混淆 | RC4 加密可用，三种模式切换 |
| 浏览器扩展 | Chrome + Firefox 链接捕获、视频嗅探 |
| WebUI | 浏览器访问可管理任务，手机适配 |
| JSON-RPC | API 方法完整，WebSocket 实时推送 |
| RSS | Feed 订阅、过滤规则、自动下载 |
| 自动解压 | zip/tar/7z 解压、密码记忆 |
| 批量下载 | 通配符/正则/文本导入 |
| 跨平台 | Windows/macOS/Linux 三平台可用 |

---

## 2 → 3：生态与平台化（10 周）

> **目标**：通过插件系统、流媒体下载、性能优化，将产品从"工具"升级为"平台"。

### 交付边界

**新增**：
- WASM 插件系统（插件 API、沙箱、管理器）
- HLS/DASH 流媒体下载
- 媒体流合并（内置轻量合并，可选 ffmpeg）
- 插件市场（在线仓库浏览/安装）
- 性能深度优化（内存、启动速度、UI 帧率）
- 全面测试与文档

---

### 第 27-28 周：HLS/DASH 流媒体下载

#### 27.1 HLS 下载

```
任务清单：
├── engine/hls.rs
│   ├── m3u8 播放列表解析（m3u8-rs crate）
│   │   ├── Master playlist → 选择分辨率
│   │   ├── Media playlist → 获取 ts 分片列表
│   │   └── 加密分片处理（AES-128 密钥获取）
│   ├── ts 分片下载
│   │   ├── 并发下载（8-16 并发）
│   │   ├── 进度回调
│   │   └── 断点续传（记录已下载分片）
│   ├── 分片合并
│   │   ├── 内置合并（ts 二进制拼接 → mp4）
│   │   ├── 可选 ffmpeg 合并（更高质量）
│   │   └── 音视频轨分离合并（DASH）
│   ├── 分辨率选择 UI
│   │   ├── 展示可用分辨率列表
│   │   ├── 显示带宽需求
│   │   └── 用户选择后开始下载
│   └── DRM 检测
│       ├── 检测 Widevine / FairPlay
│       └── 提示用户"此内容受 DRM 保护"
├── 直播录制
│   ├── 持续拉取新的 ts 分片
│   ├── 实时合并
│   └── 手动停止录制
└── DASH 支持
    ├── mpd 清单解析
    ├── 视频轨 + 音频轨分别下载
    └── ffmpeg 合并（可选）
```

**验收标准**：
- 可下载 HLS 点播视频（选择分辨率）
- 可录制 HLS 直播流
- 分片合并输出 mp4 正常播放
- DRM 内容有明确提示

---

### 第 29-31 周：WASM 插件系统

#### 29.1 插件框架

```
任务清单：
├── plugin/api.rs — 插件 API 定义
│   ├── trait DownloaderPlugin
│   │   ├── fn name(&self) -> &str
│   │   ├── fn version(&self) -> &str
│   │   ├── fn supported_protocols(&self) -> Vec<String>
│   │   ├── fn parse_url(&self, url: &str) -> Option<TaskParams>
│   │   ├── fn on_complete(&self, task: &TaskStatus) -> Result<()>
│   │   ├── fn on_error(&self, task: &TaskStatus, error: &str) -> Result<()>
│   │   └── fn get_settings(&self) -> Vec<PluginSetting>
│   ├── 插件权限声明
│   │   ├── network（允许网络请求）
│   │   ├── filesystem（允许文件读写，限制范围）
│   │   └── process（允许执行外部命令）
│   └── 插件配置 schema（JSON Schema）
├── plugin/loader.rs — WASM 加载器
│   ├── wasmtime 运行时初始化
│   ├── WASM 模块加载/卸载
│   ├── 沙箱隔离
│   │   ├── 内存限制（默认 64MB）
│   │   ├── 执行时间限制（单次调用 < 5s）
│   │   └── 系统调用白名单
│   ├── 插件宿主函数（host functions）
│   │   ├── http_get(url) → Response（需 network 权限）
│   │   ├── read_file(path) → bytes（需 filesystem 权限）
│   │   ├── write_file(path, bytes)（需 filesystem 权限）
│   │   ├── log(level, message)
│   │   └── emit_event(event_type, data)
│   └── 热加载（安装/更新无需重启）
├── plugin/manager.rs — 插件管理
│   ├── 插件列表（已安装 / 可用）
│   ├── 安装（从本地 .wasm 文件或在线仓库）
│   ├── 卸载 / 启用 / 禁用
│   ├── 版本检查 / 更新
│   └── 插件配置管理
└── 插件开发 SDK
    ├── Rust crate: downloader-plugin-sdk
    ├── 示例插件（下载完成后发送 Telegram 通知）
    └── 插件开发文档
```

#### 29.2 插件管理 UI

```
任务清单：
├── components/PluginManager.tsx
│   ├── 已安装插件列表
│   │   ├── 插件名称 / 版本 / 状态
│   │   ├── 启用/禁用开关
│   │   ├── 卸载按钮
│   │   └── 设置按钮
│   ├── 插件市场（在线仓库）
│   │   ├── 浏览可用插件
│   │   ├── 搜索 / 分类筛选
│   │   ├── 一键安装
│   │   └── 插件详情（描述 / 权限 / 评分）
│   └── 本地安装（拖拽 .wasm 文件）
├── 插件配置对话框
│   ├── 根据 plugin.get_settings() 动态渲染表单
│   └── 配置保存
└── pluginStore.ts — 插件状态管理
```

#### 29.3 插件市场后端

```
任务清单：
├── 插件仓库服务（可选，GitHub Pages 或自建）
│   ├── 插件索引（JSON manifest）
│   ├── 插件 .wasm 文件托管
│   └── 版本管理
├── 客户端仓库 API
│   ├── 获取插件列表
│   ├── 搜索插件
│   └── 下载插件
└── 插件签名验证
    ├── 仓库公钥
    └── 插件签名验证（防篡改）
```

**验收标准**：
- 可加载/卸载 WASM 插件
- 插件在沙箱中运行，无法访问沙箱外资源
- 示例插件可正常工作
- 插件市场可浏览和安装

---

### 第 32-34 周：性能优化与全面测试

#### 32.1 Rust 端优化

```
任务清单：
├── 编译优化
│   ├── Cargo.toml release profile
│   │   ├── lto = "fat"
│   │   ├── codegen-units = 1
│   │   └── strip = true
│   └── 编译时间优化（sccache / mold linker）
├── 内存优化
│   ├── 减少不必要的 clone（使用 Arc / Rc）
│   ├── 大文件流式处理（避免一次性加载）
│   ├── piece bitmap 内存优化（位压缩）
│   └── 内存泄漏检测（dhat / heaptrack）
├── 异步任务调度优化
│   ├── tokio runtime 调优（worker threads）
│   ├── 避免阻塞异步线程（spawn_blocking）
│   └── channel 缓冲区调优
├── 数据库优化
│   ├── 索引优化（tasks.state / tasks.added_at）
│   ├── 预编译语句
│   ├── WAL 模式
│   └── 批量写入（事务合并）
└── 网络优化
    ├── 连接池复用
    ├── DNS 缓存
    └── TCP 参数调优（窗口大小、nodelay）
```

#### 32.2 前端优化

```
任务清单：
├── 渲染性能
│   ├── 虚拟滚动优化（减少 DOM 节点）
│   ├── React.memo 精细化（避免不必要的重渲染）
│   ├── useMemo / useCallback 合理使用
│   └── 速度图表 Canvas 渲染（替代 SVG）
├── 打包体积
│   ├── 代码分割（dynamic import）
│   ├── Tree Shaking 验证
│   ├── 图标按需加载
│   └── 压缩（gzip / brotli）
└── 状态更新优化
    ├── 100ms 节流（已有）
    ├── 批量更新（unstable_batchedUpdates）
    └── 选择性订阅（Zustand selector）
```

#### 32.3 性能基准

```
任务清单：
├── 启动性能
│   ├── 冷启动时间 < 1.5 秒（SSD）
│   └── 热启动时间 < 0.5 秒
├── 内存占用
│   ├── 空闲状态 < 50 MB
│   ├── 20 个活跃任务 < 300 MB
│   └── 1000 个任务列表 < 100 MB 额外
├── UI 帧率
│   ├── 任务列表滚动 60fps
│   ├── 速度图表更新 60fps
│   └── 大量任务（100+）不卡顿
└── 下载性能
    ├── HTTP 多线程充分利用带宽
    ├── BT 下载速度与 qBittorrent 相当
    └── ed2k 下载速度与 eMule 相当
```

---

### 第 35-36 周：全面测试与文档

#### 35.1 测试

```
任务清单：
├── 单元测试
│   ├── Rust: cargo test（核心模块全覆盖）
│   ├── JS: vitest（组件和工具函数）
│   └── 覆盖率目标：> 70%
├── 集成测试
│   ├── Tauri IPC 通信测试
│   ├── 引擎与存储层交互测试
│   ├── 引擎间协作测试
│   └── API 端到端测试
├── E2E 测试
│   ├── Playwright（via Tauri）
│   ├── 核心流程：添加任务 → 下载 → 完成
│   ├── 异常流程：断网恢复、磁盘满、程序崩溃
│   └── 跨平台测试矩阵
├── 性能测试
│   ├── 大量任务（100 / 500 / 1000）压力测试
│   ├── 大文件（>4GB）下载测试
│   ├── 长时间运行（24 小时）稳定性测试
│   └── 内存泄漏检测
└── 回归测试
    └── CI/CD 自动触发全量测试套件
```

#### 35.2 文档

```
任务清单：
├── 用户文档
│   ├── 安装指南（三平台）
│   ├── 快速开始
│   ├── 功能说明（每个功能一页）
│   ├── 设置说明
│   ├── 常见问题（FAQ）
│   └── 故障排除
├── 开发者文档
│   ├── 构建指南
│   ├── 贡献指南
│   ├── 架构说明
│   ├── 插件开发指南
│   │   ├── 插件 API 参考
│   │   ├── 示例插件教程
│   │   └── WASM 构建指南
│   └── API 参考
│       ├── JSON-RPC 方法列表
│       ├── WebSocket 事件列表
│       └── 浏览器扩展 API
└── CHANGELOG.md（完整版本历史）
```

---

### 第 37-36 周：发布

```
任务清单：
├── 最终功能测试（全平台全功能）
├── 性能基准测试报告
├── GitHub Release 发布
│   ├── Windows: ZIP + NSIS
│   ├── macOS: DMG
│   ├── Linux: AppImage + DEB
│   └── 浏览器扩展：Chrome WebStore + Firefox Add-ons
├── 发布公告
│   ├── GitHub README 更新
│   ├── 社区公告（Reddit / V2EX / 吾爱破解）
│   └── 插件开发者招募
└── 监控与反馈
    ├── GitHub Issues 跟踪
    ├── 崩溃报告收集（可选，用户主动上传）
    └── 用户反馈渠道
```

---

### 2→3 里程碑验收清单

| 验收项 | 通过标准 |
|---|---|
| HLS 下载 | 点播分辨率选择、直播录制、分片合并 |
| DASH 下载 | mpd 解析、音视频轨合并 |
| 插件系统 | WASM 沙箱运行、热加载、权限控制 |
| 插件市场 | 浏览/搜索/安装在线插件 |
| 性能 | 冷启动 < 1.5s、空闲内存 < 50MB、60fps |
| 测试 | 单元测试覆盖 > 70%、E2E 核心流程通过 |
| 文档 | 用户文档 + 开发者文档 + API 参考完整 |
| 发布 | 三平台打包 + 浏览器扩展商店上架 |

---

## 风险与应对

| 风险 | 阶段 | 影响 | 应对 |
|---|---|---|---|
| librqbit 功能不完整 | 0→1 | BT 高级特性缺失 | 优先验证做种/加密/DHT；备选 IronTide 或 libtorrent-rasterbar C++ 绑定 |
| ed2k 自研工作量超预期 | 1→2 | M3 延期 4-8 周 | 优先实现服务器下载；KAD/信用系统/混淆降级为 P2；参考 libed2k 架构但不照搬 |
| 浏览器扩展兼容性 | 1→2 | Chrome/Firefox API 差异 | 使用 webextension-polyfill；Chrome 优先，Firefox 适配降级 |
| WASM 插件性能 | 2→3 | 插件执行慢 | 限制插件复杂度；轻量操作走 Rust 原生；WASM 仅处理配置/解析 |
| 跨平台 UI 差异 | 1→2 | 样式/行为不一致 | Windows 优先开发；macOS/Linux 集中测试修复；使用系统 WebView 减少差异 |
| HLS DRM 内容 | 2→3 | 无法下载 | 仅提示用户，不尝试破解；聚焦非 DRM 场景 |

---

## 附：技术栈确认

### Rust 依赖（按阶段引入）

**0→1 阶段**：
```toml
tauri = { version = "2", features = ["tray-icon"] }
tauri-plugin-notification = "2"
tauri-plugin-fs = "2"
tauri-plugin-clipboard = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["full"] }
reqwest = { version = "0.12", features = ["stream", "socks"] }
rusqlite = { version = "0.31", features = ["bundled"] }
toml = "0.8"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
uuid = { version = "1", features = ["v4"] }
bytes = "1"
anyhow = "1"
thiserror = "2"
librqbit = { version = "9", features = ["dht", "upnp", "peer-info"] }
```

**1→2 阶段新增**：
```toml
md4 = "0.10"                    # ed2k hash
rand = "0.8"                    # ed2k 随机 ID
axum = "0.7"                    # WebUI / API 服务
tokio-tungstenite = "0.21"      # WebSocket
tower-http = { version = "0.5", features = ["cors", "fs"] }
feed-rs = "1.3"                 # RSS 解析
zip = "0.6"                     # 解压
tar = "0.4"
flate2 = "1.0"
sevenz-rust = "0.5"
jsonrpsee = { version = "0.23", features = ["server"] }  # JSON-RPC
```

**2→3 阶段新增**：
```toml
m3u8-rs = "5.0"                 # HLS 解析
wasmtime = "22"                 # WASM 插件运行时
```

### 前端依赖（按阶段引入）

**0→1 阶段**：
```json
{
  "react": "^18.3",
  "react-dom": "^18.3",
  "@tauri-apps/api": "^2",
  "@tauri-apps/plugin-notification": "^2",
  "@tauri-apps/plugin-fs": "^2",
  "@tauri-apps/plugin-clipboard": "^2",
  "zustand": "^4.5",
  "@tanstack/react-virtual": "^3.8",
  "@tanstack/react-table": "^8.19",
  "@radix-ui/react-dialog": "^1.1",
  "@radix-ui/react-context-menu": "^2.2",
  "@radix-ui/react-dropdown-menu": "^2.1",
  "recharts": "^2.12",
  "lucide-react": "^0.400",
  "framer-motion": "^11.3",
  "@dnd-kit/core": "^6.1",
  "@dnd-kit/sortable": "^8.0",
  "clsx": "^2.1",
  "tailwind-merge": "^2.4",
  "date-fns": "^3.6"
}
```

**1→2 阶段新增**：
```json
{
  "webextension-polyfill": "^0.10"
}
```

---

### 迭代 22：代码审查修复与质量改进

| 任务 | 状态 | 说明 |
|---|---|---|
| Toast ID 碰撞修复 | ✅ 已完成 | `Date.now()+counter` → `crypto.randomUUID()`，消除并发碰撞风险 |
| 动态导入修复 | ✅ 已完成 | `TaskList.tsx` 中 `handlePriority` 改为静态导入 `getAllTasks` |
| formatEta 负数处理 | ✅ 已完成 | 负值返回 `"--"` 而非 `"∞"`，区分"未知"和"计算错误" |
| 剪贴板冷却机制 | ✅ 已完成 | 新增 10 秒冷却时间，避免复制普通 URL 重复触发弹窗 |
| Toast HMR 泄漏修复 | ✅ 已完成 | `toastListeners` 改为 globalThis 持久化 + splice 原地移除 |
| stateOrder 常量提取 | ✅ 已完成 | 从内联对象提取为 `STATE_ORDER` 模块级常量 |
| 测试同步更新 | ✅ 已完成 | Toast UUID 格式测试 + formatEta 负值测试更新，338 测试全通过 |

**变更文件**：`Toast.tsx`, `TaskList.tsx`, `format.ts`, `useClipboard.ts`, `taskStore.ts`, `components.test.ts`, `format.test.ts`, `REVIEW.md`, `TEST_REPORT.md`

---

### 迭代 26：统一错误处理模块

| 任务 | 状态 | 说明 |
|---|---|---|
| 创建统一错误处理模块 | ✅ 已完成 | `lib/errors.ts`：DownloaderError 类、ErrorCode 枚举（20+ 错误码）、handleError/withErrorHandling/withSyncErrorHandling 函数 |
| 错误码自动推断 | ✅ 已完成 | 从错误消息自动推断错误码（network→NETWORK_ERROR、timeout→TIMEOUT、ENOSPC→DISK_FULL 等） |
| 用户友好消息 | ✅ 已完成 | 每个错误码映射到中文用户友好消息 |
| 组件错误处理统一 | ✅ 已完成 | 更新 App.tsx、TaskList.tsx、PluginManager.tsx、RssManager.tsx、SettingsDialog.tsx 使用统一错误处理 |
| 错误处理测试 | ✅ 已完成 | 新增 30 个测试：DownloaderError 构造/from/toUserMessage、错误码推断、handleError/withErrorHandling/withSyncErrorHandling |

**变更文件**：`lib/errors.ts`（新增）, `App.tsx`, `components/TaskList.tsx`, `components/PluginManager.tsx`, `components/RssManager.tsx`, `components/SettingsDialog.tsx`, `__tests__/errors.test.ts`（新增）

**测试统计**：385 测试全通过（原 355 → 新增 30）

---

## Bug 修复与质量改进追踪

| 日期 | 类型 | 文件 | 问题描述 | 修复方案 | 状态 |
|---|---|---|---|---|---|
| 2026-05-29 | Bug | `rss/rules.rs` | `simple_regex_match` 使用无界递归，复杂正则（如 `.*.*`）会导致栈溢出 | 重写为迭代+回溯算法，使用状态栈模拟递归 | ✅ 已修复 |
| 2026-05-29 | Bug | `commands/settings.rs` | `update_settings` 使用硬编码相对路径 `data/config.toml`，不同工作目录下会失败 | 改为从 `AppState.config_path` 读取（由 `main.rs` 初始化时设置） | ✅ 已修复 |
| 2026-05-29 | Bug | `main.rs` | `AppState` 缺少 `config_path` 字段，settings 命令无法获取正确配置路径 | 新增 `config_path: PathBuf` 字段，在 `setup` 中初始化 | ✅ 已修复 |
| 2026-05-29 | 清理 | `package.json` | `@dnd-kit/*`, `@tanstack/react-table`, `framer-motion`, `date-fns` 从未被导入 | 从 dependencies 中移除 | ✅ 已修复 |
| 2026-05-29 | 测试 | `__tests__/taskStore.test.ts` | taskStore（核心状态管理）无任何测试 | 新增 17 个测试用例，覆盖 setTasks/applyUpdate/selectTask/removeTask/sort/stats | ✅ 已完成 |
| 2026-05-29 | Bug | `engine/ed2k/hash.rs` | `Md4Hasher::process_block` 为空实现，ed2k hash 计算返回全零 | 引入 `md4` crate，用真实 MD4 替代手写空实现，新增已知向量测试 | ✅ 已修复 |
| 2026-05-29 | Bug | `api/mod.rs` | `start_http` 为 TODO stub，浏览器扩展和 WebUI 无法连接 | 使用 axum 实现 POST /jsonrpc 和 GET /health 端点，含 CORS 支持 | ✅ 已修复 |
| 2026-05-29 | 依赖 | `Cargo.toml` | 缺少 `md4`, `axum`, `tokio-tungstenite`, `tower-http` | 添加依赖用于 ed2k hash 和 JSON-RPC HTTP 服务 | ✅ 已修复 |
| 2026-05-29 | 功能 | `api/rpc.rs` | 所有 RPC handler 返回硬编码 stub 数据 | 接入 AppState，handler 调用 TaskManager 和 Database 实现真实 CRUD | ✅ 已修复 |
| 2026-05-29 | 功能 | `archive/mod.rs` | TAR.BZ2 格式返回 "not implemented" | 添加 bzip2 crate，实现 extract_tar_bz2 方法 | ✅ 已修复 |
| 2026-05-29 | 功能 | `ed2k/kad.rs` | KAD 引导/迭代查找/搜索全部为 TODO stub | 实现 UDP 引导、α=3 迭代查找、关键词搜索、源搜索 | ✅ 已修复 |
| 2026-05-29 | 功能 | `ed2k/hash.rs` | AICH hash 为 TODO stub | 实现 SHA1 Merkle Tree（180KB 子块），新增 sha1 crate | ✅ 已修复 |
| 2026-05-29 | 功能 | `plugin/loader.rs` | WASM 运行时未集成 | 集成 wasmtime 29，内存限制、fuel 限制、宿主函数注入 | ✅ 已修复 |
| 2026-05-29 | 资源 | `extension/icons/` | 浏览器扩展图标文件缺失 | 生成 16/48/128px PNG 图标 | ✅ 已修复 |

### 待修复的已知问题

| 优先级 | 类型 | 文件 | 问题描述 | 状态 |
|---|---|---|---|---|
| ~~高~~ | ~~缺失依赖~~ | `Cargo.toml` | ~~`md4` crate 未添加~~ | ✅ 已修复 |
| ~~高~~ | ~~缺失功能~~ | `api/mod.rs` | ~~JSON-RPC HTTP 服务未启动~~ | ✅ 已修复 |
| ~~中~~ | ~~缺失功能~~ | `commands/plugin.rs` | ~~插件管理全部 5 个命令为 stub~~ | ✅ 已修复 |
| ~~中~~ | ~~缺失功能~~ | `ed2k/mod.rs` | ~~`Ed2kEngine::download` 仅 sleep 100ms~~ | ✅ 已修复 |
| ~~中~~ | ~~缺失功能~~ | `archive/mod.rs` | ~~TAR.XZ, RAR, 7Z 格式解压返回 "not implemented"~~ | ✅ 已修复 |
| ~~中~~ | ~~缺失功能~~ | `ed2k/kad.rs` | ~~KAD 引导/迭代查找/搜索全部为 TODO stub~~ | ✅ 已修复 |
| ~~中~~ | ~~缺失功能~~ | `plugin/loader.rs` | ~~WASM 运行时未集成（wasmtime）~~ | ✅ 已修复 |
| ~~中~~ | ~~缺失功能~~ | `ed2k/hash.rs` | ~~AICH hash 为 TODO stub~~ | ✅ 已修复 |
| ~~低~~ | ~~缺失资源~~ | `extension/` | ~~浏览器扩展图标文件缺失~~ | ✅ 已修复 |
| ~~低~~ | ~~缺失构建~~ | `src-webui/` | ~~WebUI 无独立构建管线~~ | ✅ 已修复 |

*— End of Implementation Plan —*
