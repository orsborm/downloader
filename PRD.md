# PRD - 全协议下载器 (Full-Protocol Downloader)

## 产品愿景
绿色便携、全协议支持、零广告的跨平台下载管理器。

## 功能优先级矩阵

### P0 - 核心功能（必须实现）
| ID | 功能 | 状态 | 说明 |
|----|------|------|------|
| F-001 | HTTP/HTTPS 多线程下载 | ✅ 完成 | 10MB 分块，最多 16 并发 |
| F-002 | 断点续传 | ✅ 完成 | Range header + 暂停/恢复 |
| F-003 | BT/Magnet 下载 | ✅ 完成 | librqbit 引擎，DHT/PEX |
| F-004 | HLS 流媒体下载 | ✅ 完成 | m3u8 解析，分段下载，TS 合并 |
| F-005 | 任务管理 | ✅ 完成 | 增删改查、暂停恢复、批量操作 |
| F-006 | **单任务限速** | ✅ 完成 | governor 令牌桶限速，per-task speed_limit |
| F-007 | **全局限速** | ✅ 完成 | TaskManager 共享 RateLimiter，运行时动态更新 |

### P1 - 重要功能
| ID | 功能 | 状态 | 说明 |
|----|------|------|------|
| F-101 | 任务详情面板 | ✅ 完成 | 连接/Tracker/日志 tab 显示有意义数据 |
| F-102 | DHT 状态显示 | ✅ 完成 | 从 librqbit 获取真实 DHT 节点数 |
| F-103 | RSS 自动下载 | ✅ 完成 | IPC 已连接，支持添加/删除/OPML 导入导出 |
| F-104 | 自动解压 | ✅ 完成 | IPC 已连接，支持 ZIP/TAR/TAR.GZ 解压 |
| F-105 | 下载完成通知 | ✅ 完成 | 系统通知，任务完成/出错时触发 |

### P2 - 增强功能
| ID | 功能 | 状态 | 说明 |
|----|------|------|------|
| F-201 | ed2k 下载 | ✅ 下载逻辑完成 | 服务器连接、源查找、分块下载、hash 校验 |
| F-202 | WASM 插件系统 | ✅ IPC 完成 | PluginManager 集成到 AppState，5 个 IPC 命令实现，前端已连接 |
| F-203 | JSON-RPC API | ✅ 完成 | aria2 兼容 HTTP+WS 服务器，已接入 AppState |
| F-204 | 浏览器扩展 | ✅ 完成 | Chrome MV3，下载拦截/资源检测 |
| F-205 | Web UI | ✅ 完成 | 响应式 Web 界面，RPC 通信 |

### P3 - 测试与质量
| ID | 功能 | 状态 | 说明 |
|----|------|------|------|
| F-301 | Rust 单元测试 | ⚠️ 部分完成 | ed2k/hls/rss/util 有测试 |
| F-302 | **前端单元测试** | ✅ 完成 | 101 个测试覆盖 format.ts + taskStore.ts |
| F-303 | 集成测试 | ❌ 未实现 | 端到端下载流程测试 |

## 已完成迭代

### 迭代 1: 限速功能 + 前端测试
- F-006: 使用 governor crate 实现 per-task 令牌桶限速
- F-302: vitest 配置 + 23 个 format.ts 测试
- 修复: clipboard 插件包名 (`@tauri-apps/plugin-clipboard-manager`)

### 迭代 2: RSS IPC 实现
- F-103: 将 RssEngine 添加到 AppState，实现 5 个 IPC 命令
- 支持: 添加/删除订阅、获取列表、OPML 导入/导出

### 迭代 3: DHT 状态显示
- F-102: 添加 get_bt_status IPC 命令
- StatusBar 每 5 秒轮询 BT 状态，显示真实 DHT 节点数
- DHT 图标连接时变绿

### 迭代 4: 自动解压 IPC
- F-104: 将 ArchiveManager 添加到 AppState
- 实现 get_archive_config, update_archive_config, extract_archive IPC
- 添加 zip, tar, flate2 依赖

### 迭代 5: 下载完成通知
- F-105: 使用 @tauri-apps/plugin-notification 实现系统通知
- 任务完成和出错时自动触发通知
- 通知去重：每个任务只通知一次

### 迭代 6: 任务详情面板
- F-101: 增强 Connections/Tracker/Logs tab
- Connections: 显示 BT 任务的 Peer 数量
- Tracker: 显示 BT 任务的连接状态
- Logs: 显示任务状态、错误、速度、进度信息

### 迭代 7: HLS/DASH 流媒体集成
- F-401: HLS/DASH 协议集成到 TaskManager
  - Protocol 枚举新增 Hls/Dash 变体 (Rust + TypeScript)
  - HlsEngine 集成到 TaskManager 任务调度
  - 协议自动检测支持 .m3u8 和 .mpd URL
  - HlsEngine 实现 Clone trait 以兼容 TaskManager
- F-402: 前端 HLS/DASH 支持
  - Protocol 类型新增 HLS/DASH
  - protocolLabel 标签映射
  - AddTaskDialog 提示更新
  - 测试用例覆盖
- F-403: WebUI WebSocket 管理器
  - 新建 src-webui/lib/ws.ts
  - WsManager 类：自动重连、心跳保活、事件分发
  - 可配置重连间隔和最大重连次数

### 迭代 8: 全局限速
- F-007: 全局下载速度限制
  - TaskManager 持有共享 RateLimiter (governor crate)
  - 从 config.max_download_speed 初始化
  - set_global_speed_limit() 运行时动态更新
  - HTTP 引擎同时应用全局限速和单任务限速
  - 设置更新时自动同步限速/并发/重试配置

### 迭代 9: Bug 修复 + 前端后端连接
- **HLS task_id 修复**: HlsEngine.download() 新增 task_id 参数，TaskUpdateEvent 不再发送空 task_id
- **通知内存泄漏修复**: useTaskEvents 中 notifiedTasks 在任务删除时自动清理
- **RSS Manager 前端连接**: RssManager 组件接入后端 IPC (getRssFeeds/addRssFeed/removeRssFeed/importOpml/exportOpml)
- **Archive Dialog 前端连接**: ArchiveDialog 组件接入后端 IPC (getArchiveConfig/updateArchiveConfig)
- **tauri-api 扩展**: 新增 RSS 和 Archive 相关 IPC 封装函数
- **浏览器扩展修复**: manifest.json 添加缺失的 notifications 权限

### 迭代 10: UX 改进 + 代码质量
- **右键菜单修复**: TaskList 右键菜单视口边界检测，防止被裁剪
- **类型安全**: SettingsDialog 子组件 `Function` 类型替换为 `UpdateFieldFn`
- **空状态 UI**: TaskList 和 PluginManager 添加空状态引导提示
- **动态版本号**: StatusBar 从后端获取真实版本号替代硬编码
- **formatProgress 健壮性**: 处理 NaN/Infinity 输入
- **TypeScript 零警告**: 修复所有 TS6133 未使用变量警告
- **测试扩展**: 新增 2 个 formatProgress 边界测试 (25 total)

### 迭代 11: 插件系统 IPC 集成
- **PluginManager 接入 AppState**: 将 PluginManager 添加到 main.rs 的 AppState，启动时自动扫描 plugins 目录
- **5 个 IPC 命令实现**: list_plugins, install_plugin, uninstall_plugin, enable_plugin, disable_plugin
- **前端 PluginManager 连接**: 组件接入后端 IPC，支持安装/卸载/启用/禁用操作
- **WASM 验证**: install_plugin 自动验证 WASM 文件有效性（魔数 + 版本号）
- **plugin 模块注册**: main.rs 新增 `mod plugin;` 声明

### 迭代 12: UX 改进 + 测试扩展
- **工具栏导航**: 添加 RSS/插件/解压按钮到工具栏，之前这些组件存在但无法从 UI 访问
- **formatDateTime 修复**: 无效日期返回原始字符串而非 "Invalid Date"
- **formatDateTime 测试**: 新增 3 个测试用例（有效日期/无效日期/空字符串）
- **插件 API 封装**: tauri-api.ts 新增 listPlugins/installPlugin/uninstallPlugin/enablePlugin/disablePlugin
- **测试总数**: 45 个测试全部通过

### 迭代 13: JSON-RPC API 服务器接入 (F-203)
- **ApiService 接入 AppState**: main.rs 新增 ws_manager 字段，启动时初始化 WsManager
- **HTTP 服务器启动**: 从 connection.http_port 读取端口配置，http_port>0 时自动启动 axum 服务器
- **WebSocket 端点**: 新增 /ws 路由，支持 WebSocket 升级连接
- **事件转发**: task-update 事件同时广播到 Tauri 前端和 WebSocket 客户端
- **API 状态 IPC**: 新增 get_api_status 命令，返回 enabled/host/port/endpoint/wsConnections
- **前端 API 封装**: tauri-api.ts 新增 getApiStatus() 函数
- **WebUI 可用**: API 服务器启动后，WebUI (src-webui) 可通过 JSON-RPC 通信

### 迭代 14: 测试扩展 (F-302 增强)
- **isValidDownloadUrl 扩展**: 新增 FTPS/file:// 支持，null/undefined 边界测试，大小写不敏感测试
- **taskStore 边界测试**: applyUpdate 错误/完成状态、removeTask 空 store、getSortedTasks 按进度/速度排序
- **getGlobalStats 验证**: seeding 状态计入 activeCount 的行为确认
- **测试总数**: 61 → 76 个测试全部通过

### 迭代 15: API 设置 UI + 状态栏
- **HTTP API 端口设置**: ConnectionSettings 新增端口输入框（0=禁用）和认证 Token 输入
- **API 状态栏指示**: StatusBar 显示 API 端口和 WebSocket 连接数
- **类型同步**: types.ts 新增 apiToken 字段

### 迭代 16: 下载历史管理
- **DownloadHistoryDialog**: 新组件，表格展示下载历史（文件名/协议/大小/速度/完成时间）
- **清空历史**: clear_download_history IPC 命令 + 前端确认对话框
- **工具栏入口**: 新增"历史"按钮，Clock 图标
- **后端支持**: db.rs 新增 clear_download_history() 方法

### 迭代 17: UX 改进 - Toast 通知 + 优先级设置
- **Toast 通知系统**: 轻量级 Toast 组件，支持 success/error/info 类型，自动消失
- **用户反馈**: 工具栏操作（暂停/恢复/清理）显示 toast 通知
- **任务添加反馈**: AddTaskDialog 成功后显示 toast
- **拖拽反馈**: 种子文件拖拽添加成功/失败显示 toast
- **任务优先级**: 右键菜单新增高/普通/低优先级选项，带当前状态标记
- **API Token 认证**: JSON-RPC handler 支持 Bearer token 和 aria2 兼容 token: 前缀认证

## 统计
- **已完成 P0 功能**: 7/7 (100%)
- **已完成 P1 功能**: 5/5 (100%)
- **已完成 P2 功能**: 5/5 (100%)
- **已完成 P3 功能**: 2/3 (67%)
- **总完成功能**: 19/20 (95%)

### 迭代 18: ed2k 下载逻辑实现 (F-201)
- **Ed2kEngine.download() 完整实现**: 服务器连接 → 登录 → 源查找 → 分块下载 → hash 校验
- **Ed2kTransfer.stream_mut()**: 暴露 TCP 流供引擎直接读取数据包
- **ServerConnection.recv_packet() 公开化**: 允许引擎层读取服务器响应
- **Ed2kEngine 实现 Clone**: 兼容 TaskManager 的 spawned task 模式
- **TaskManager ed2k 集成**: 替换 stub 为真实 Ed2kEngine 调用
- **ed2k 链接解析增强**: 支持可选服务器地址参数 (server,ip:port)
- **hex_to_hash 工具函数**: 32 字符十六进制转 16 字节数组（无外部依赖）
- **ed2k 单元测试扩展**: 新增 4 个测试（含服务器链接解析、hex 转换、配置默认值）
- **前端测试扩展**: taskStore 新增 10 个 + format 新增 15 个边界测试，总计 101 个测试全部通过

### 迭代 19: 搜索过滤 + 错误处理改进
- **任务搜索**: 工具栏新增搜索框，按文件名和 URL 过滤任务（大小写不敏感）
- **状态过滤**: 工具栏新增状态下拉框，按 downloading/paused/done/error/seeding/queued 过滤
- **组合过滤**: 搜索和状态过滤可同时生效
- **空结果提示**: 过滤无结果时显示"没有匹配的任务"，带清除筛选按钮
- **错误处理统一**: 所有组件的 console.error 替换为 showToast，用户可见的错误提示
- **受影响组件**: TaskList, App, ArchiveDialog, SettingsDialog, RssManager, PluginManager, DownloadHistoryDialog
- **测试扩展**: 新增 17 个搜索/过滤测试，总计 118 个测试全部通过

### 迭代 20: UX 改进 - 批量导入入口 + 快捷键提示
- **批量导入按钮**: 工具栏新增 Layers 图标按钮，可直接打开批量导入对话框
- **过滤计数**: 搜索/过滤激活时，工具栏显示"X/Y 个任务"
- **快捷键提示**: AddTaskDialog 底部显示"Ctrl+Enter 确认 | Esc 取消"
- **App.tsx**: 新增 BatchImportDialog 导入和 showBatchImport 状态

### 迭代 21: UX 改进 - 搜索快捷键 + 选中计数 + 批量导入刷新
- **Ctrl+F 搜索**: 全局 Ctrl+F 快捷键聚焦搜索框，搜索框 placeholder 显示快捷键提示
- **选中计数**: 多选任务时工具栏显示"已选 X 个"
- **批量导入刷新**: 导入成功后自动刷新任务列表并显示 toast
- **BatchImportDialog**: 导入完成后调用 getAllTasks() + setTasks() 刷新 store

### 迭代 22: formatDuration + 做种速度显示 + 下载耗时
- **formatDuration 函数**: 新增时间区间格式化（秒 → 可读时长），支持时/分/秒
- **做种速度显示**: TaskList 做种任务显示上传速度（↑X.X MB/s）而非下载速度
- **下载耗时**: TaskDetail 概要 Tab 新增"耗时"行，显示从添加到完成的时间
- **测试扩展**: 新增 6 个 formatDuration 测试，总计 124 个测试全部通过

### 迭代 23: 下载历史耗时列
- **DownloadHistoryDialog**: 新增"耗时"列，使用 formatDuration 显示每条记录的下载时长
- **表头更新**: 平均速度和完成时间之间新增耗时列

## 下一步优先级
1. **F-303**: 集成测试 - 端到端下载流程测试
2. **F-304**: 错误处理增强 - 统一错误类型和用户提示（部分完成：前端 toast 已统一）
