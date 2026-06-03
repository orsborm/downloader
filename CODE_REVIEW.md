# 全协议下载器 - 项目进展与代码审查报告

> 审查日期: 2026-05-29
> 审查范围: downloader/ 全部源码 (Rust 后端、React 前端、Chrome 扩展、WebUI)

---

## 一、项目进展概览

### 技术栈

| 层级 | 技术 |
|------|------|
| 桌面框架 | Tauri v2 (Rust + WebView) |
| 前端 | React 18 + TypeScript + Vite + Tailwind CSS + Zustand |
| 后端 | Rust (tokio + reqwest + librqbit + rusqlite + axum) |
| 远程管理 | JSON-RPC 2.0 API + WebSocket (aria2 兼容) |
| 浏览器扩展 | Chrome MV3 Service Worker + Content Script |
| 插件系统 | WASM (wasmtime，当前为 stub) |

### 功能完成度

| 迭代 | 内容 | 状态 |
|------|------|------|
| 迭代 1 | 限速功能 + 前端测试 | ✅ 已完成 |
| 迭代 7 | HLS/DASH 流媒体集成 | ✅ 已完成 |
| 迭代 8 | 全局下载限速 | ✅ 已完成 |
| 迭代 9 | Bug 修复 + 前端后端连接 | ✅ 已完成 |
| 迭代 10 | UX 改进 + 代码质量 | ✅ 已完成 |
| 迭代 11 | 插件系统 IPC 集成 | ✅ 已完成 |
| 迭代 12 | UX 改进 + 测试扩展 | ✅ 已完成 |

### 整体完成度评估

- **Phase 0→1 (MVP)**: ~90% — HTTP/BT 引擎、UI、IPC 命令已完成。BT 核心下载集成待完善。
- **Phase 1→2 (全协议)**: ~85% — ed2k、Chrome 扩展、WebUI、RSS、归档模块已编码。集成测试待完成。
- **Phase 2→3 (平台/生态)**: ~80% — HLS、插件、RSS UI、归档 UI 已完成。性能优化和全面测试待完成。

---

## 二、代码审查 - 严重问题 (CRITICAL)

### 2.1 Zip Slip 路径遍历漏洞 ⚠️

**文件**: `src-tauri/src/archive/mod.rs` 第 180、247、287、327 行

解压功能直接将归档条目路径拼接到目标目录，未校验 `../` 序列。恶意归档可写入任意位置。

```rust
// 危险: entry.path() 可能包含 "../../etc/passwd"
let out_path = extract_dir.join(entry.mangled_name());  // ZIP
let out_path = extract_dir.join(entry.path()...);         // TAR/TAR.GZ/TAR.BZ2
```

**修复建议**: 解析后校验 `out_path.canonicalize()` 是否以 `extract_dir` 开头。

### 2.2 插件系统完全不可用 ⚠️

**文件**: `src-tauri/src/plugin/loader.rs` 第 98-145 行

`execute_plugin` 函数体全部被注释，硬编码返回空结果。整个 WASM 插件系统无任何实际执行能力。

```rust
// 全部被注释的 wasmtime 实现...
Ok(Vec::new())  // 永远返回空
```

**附带问题**: `load_plugin_manifest` (mod.rs 第 200-223 行) 也是 stub，所有插件元数据为硬编码假数据。

### 2.3 死锁风险 - 锁顺序不一致 ⚠️

**文件**:
- `src-tauri/src/main.rs` 第 178 行: 先锁 `task_manager`，再锁 `db`
- `src-tauri/src/commands/task.rs` 第 261 行: 先锁 `task_manager`，循环内锁 `db`
- `src-tauri/src/commands/task.rs` 第 34 行: `blocking_lock()` 在异步上下文中使用

两条路径以不同顺序获取 `task_manager` 和 `db` 的 Mutex，高并发下可触发死锁。

---

## 三、代码审查 - 高危问题 (HIGH)

### 3.1 JSON-RPC API 无认证

**文件**: `src-tauri/src/api/rpc.rs` + `mod.rs`

API 服务器使用 `CorsLayer::permissive()` 且未校验 `ApiConfig.token`。任何本地进程（若开启 `allow_remote` 则包括远程）可无认证地添加/删除/暂停任务。

### 3.2 代理密码明文存储

**文件**: `src-tauri/src/storage/config.rs` 第 50-54 行

`proxy_password` 以明文写入 `config.toml`，且 `proxy_url()` 将密码嵌入 URL 字符串（第 211-229 行），可能泄露到日志和 HTTP Referer。

### 3.3 重试时下载进度重置

**文件**: `src-tauri/src/engine/task_manager.rs` 第 249、338-349 行

重试逻辑使用任务添加时的初始 `downloaded` 值，而非当前已下载字节数。HTTP 分块续传会从错误偏移恢复，导致重复下载或跳过数据。

### 3.4 RSS 轮询间隔失效

**文件**: `src-tauri/src/rss/mod.rs` 第 110-111 行

所有 RSS 源共享同一个 `last_poll` 时间戳。设置不同轮询间隔的源会在同一时刻被全部拉取，间隔配置形同虚设。

### 3.5 OPML 导出 XML 注入

**文件**: `src-tauri/src/rss/mod.rs` 第 179-193 行

`feed.name` 和 `feed.url` 直接拼入 XML，未转义 `<`、`>`、`&`、`"` 等特殊字符。

### 3.6 ZIP 密码解压 API 用法错误

**文件**: `src-tauri/src/archive/mod.rs` 第 196-209 行

对已获取的 `ZipFile` 调用 `with_password` 不符合 `zip` crate 的 API 设计，密码保护的 ZIP 文件无法正确解压。

### 3.7 浏览器扩展 suggest() 竞态

**文件**: `extension/background.js` 第 10-21 行

`chrome.downloads.onDeterminingFilename` 回调中异步读取 `chrome.storage.local`，`suggest` 回调可能在存储查询返回前超时。取消下载时未调用 `suggest()`，可能导致下载挂起。

### 3.8 Semaphore 翻转 panic

**文件**: `src-tauri/src/engine/task_manager.rs` 第 225 行

`semaphore.acquire().await.unwrap()` 在信号量被 `set_max_concurrent` 替换后会 panic，因为旧信号量已关闭。

---

## 四、代码审查 - 中危问题 (MEDIUM)

### 后端 (Rust)

| # | 文件 | 行号 | 问题 |
|---|------|------|------|
| 1 | `http.rs` | 363 | `seek().await.ok()` 静默忽略错误，后续 write 可能损坏文件 |
| 2 | `http.rs` | 187-189 | 限速值 u64→u32 截断，>4GB/s 时行为未定义 |
| 3 | `http.rs` | 382 | `progress` 可超过 1.0，缺少 `min(1.0)` 钳位 |
| 4 | `task_manager.rs` | 401-451 | `pause_task`/`resume_task` 找不到任务时静默返回 Ok |
| 5 | `task_manager.rs` | 472-474 | `set_max_concurrent` 替换信号量但不迁移运行中任务的许可 |
| 6 | `task_manager.rs` | 165-177 | BT 任务完成后 TaskHandle 不清理，内存泄漏 |
| 7 | `db.rs` | 262-276 | `update_task_progress` 接受 speed/peers 参数但不写入数据库 |
| 8 | `db.rs` | 33-42 | `from_str` 未知值静默回退到默认值，掩盖数据损坏 |
| 9 | `config.rs` | 103 | Windows 路径硬编码未做平台判断 |
| 10 | `rpc.rs` | 376-388 | `purgeCompleted` 只清理 Done 不清理 Error，与 IPC 命令不一致 |
| 11 | `websocket.rs` | 18,60-65 | `subscribed_events` 定义了但从未用于过滤 |
| 12 | `websocket.rs` | 68,74 | 断开的连接不清理，connections 无限增长 |
| 13 | `main.rs` | 160 | 500ms 硬编码 sleep 等待前端就绪，不可靠 |
| 14 | `bt/mod.rs` | 347-348 | `get_status()` 上传/下载速度硬编码为 0 |

### 前端 (TypeScript/React)

| # | 文件 | 行号 | 问题 |
|---|------|------|------|
| 1 | `App.tsx` | 41-51 | 任务加载与事件监听竞态：事件先到会覆盖初始加载 |
| 2 | `App.tsx` | 148-149 | 乐观删除无回滚，IPC 失败后 UI 与后端不一致 |
| 3 | `taskStore.ts` | 125 | Shift-select 按插入顺序而非排序后顺序选择 |
| 4 | `taskStore.ts` | 109 | 速度历史仅保留 5 分钟，但图表支持 30 分钟/1 小时 |
| 5 | `useTaskEvents.ts` | 46 | 通知显示 UUID 而非任务名称 |
| 6 | `useClipboard.ts` | 39 | 首次轮询触发检测，启动时可能弹出添加对话框 |
| 7 | `TaskList.tsx` | 83 | `getSortedTasks()` 每次渲染都排序，未 memoize |
| 8 | `TaskList.tsx` | 72 | 订阅 `allTasks` 导致每次进度更新都重新渲染整个列表 |
| 9 | `TaskList.tsx` | 353 | 使用 `window.confirm` 而非 Tauri 原生对话框 |
| 10 | `AddTaskDialog.tsx` | 51 | 动态 import 已静态导入的模块 |
| 11 | `AddTaskDialog.tsx` | 46 | `parseInt` 无 NaN 校验 |
| 12 | `TaskDetail.tsx` | 207-239 | LogsTab 每次渲染重新生成带时间戳的日志 |
| 13 | `TaskDetail.tsx` | 155-199 | ConnectionsTab/TrackerTab 为占位 stub |
| 14 | `SettingsDialog.tsx` | 全文 | 大量配置字段未暴露到 UI |
| 15 | `SpeedChart.tsx` | 32 | `Date.now()` 在 useMemo 内使用，语义不正确 |
| 16 | `format.ts` | 28 | `formatEta(0)` 返回 "∞" 而非 "0s" |
| 17 | `format.ts` | 96 | `isValidDownloadUrl` 对 ".m3u8" 子串误判 |

### Chrome 扩展

| # | 文件 | 行号 | 问题 |
|---|------|------|------|
| 1 | `manifest.json` | 15-16 | `host_permissions` 为 `<all_urls>`，权限过宽 |
| 2 | `content.js` | 184 | 注入按钮强制设置 `position: relative`，破坏页面布局 |
| 3 | `content.js` | 115 | 无扩展名 URL 返回整个文件名作为格式 |
| 4 | `popup.js` | 61-69 | 手动输入 URL 无校验直接发送到后端 |
| 5 | `popup.js` | 85 | `response.count` 可能为 undefined |

### WebUI

| # | 文件 | 行号 | 问题 |
|---|------|------|------|
| 1 | `App.tsx` | 41 | `as unknown as TaskInfo[]` 绕过类型安全 |
| 2 | `App.tsx` | 50-55 | 使用无重连的 `createWebSocket` 而非 `WsManager` |
| 3 | `rpc-api.ts` | 29 | `Date.now()` 作为 JSON-RPC id，同毫秒请求会冲突 |
| 4 | `rpc-api.ts` | 38 | 未检查 HTTP 状态码就解析 JSON |

---

## 五、代码审查 - 低危/质量问题 (LOW)

| # | 模块 | 问题 |
|---|------|------|
| 1 | `task_manager.rs:93` | `_max_upload_speed` 参数未使用（死代码） |
| 2 | `bt/mod.rs:221-232` | BT 任务完成事件丢失上传统计 |
| 3 | `db.rs:303-416` | `get_task`/`get_all_tasks`/`get_unfinished_tasks` 行映射代码重复 3 次 |
| 4 | `rss/mod.rs:118` | `md5_simple` 函数名误导（实际是 DJB2 哈希） |
| 5 | `rss/mod.rs:197-203` | DJB2 哈希碰撞风险用于去重 |
| 6 | `archive/mod.rs:149-158` | RAR/7Z/TAR.XZ 格式检测到但未实现 |
| 7 | `plugin/mod.rs:145-157` | 插件安装不检查同 ID 已存在，静默覆盖 |
| 8 | `plugin/mod.rs:160-186` | 卸载/启用/禁用不存在的插件静默返回 Ok |
| 9 | `plugin/mod.rs` vs `api.rs` | `PluginSetting` 结构体重复定义且字段不一致 |
| 10 | `extension/content.js:151` | 使用 `innerHTML` 而非 `textContent` |
| 11 | `useTaskEvents.ts:12` | 模块级 `notifiedTasks` 在 HMR 下共享 |
| 12 | `useClipboard.ts` | 重复复制同一 URL 会再次触发检测 |
| 13 | `webui/App.tsx:73` | 使用 `alert()` 做错误提示 |

---

## 六、安全问题汇总

| 严重度 | 问题 | 文件 |
|--------|------|------|
| **CRITICAL** | Zip Slip 路径遍历 | `archive/mod.rs:180,247,287,327` |
| **HIGH** | JSON-RPC API 无认证 | `api/rpc.rs` + `api/mod.rs` |
| **HIGH** | 代理密码明文存储 | `storage/config.rs:50-54` |
| **HIGH** | 代理密码嵌入 URL | `storage/config.rs:211-229` |
| **HIGH** | OPML 导出 XML 注入 | `rss/mod.rs:179-193` |
| **HIGH** | 扩展 suggest() 竞态 | `extension/background.js:10-21` |
| **MEDIUM** | 扩展 host_permissions 过宽 | `extension/manifest.json:15-16` |
| **MEDIUM** | URL 无 SSRF 校验 | `commands/task.rs:12-78` |
| **LOW** | 插件 http_get 无域名过滤 (已注释) | `plugin/loader.rs:117-120` |

---

## 七、改进建议优先级

### P0 - 必须立即修复

1. **修复 Zip Slip 漏洞** — 归档解压前校验路径
2. **统一锁顺序** — 制定并遵守 `db → task_manager` 的固定锁顺序
3. **修复重试进度重置** — 重试时从数据库读取当前 downloaded 值
4. **API 认证** — 实现 token 校验中间件

### P1 - 尽快修复

5. **RSS 轮询间隔** — 每个 feed 独立跟踪 last_poll
6. **OPML XML 转义** — 对特殊字符做 XML escape
7. **通知显示任务名** — 从 store 查询任务名而非显示 UUID
8. **前端竞态修复** — 初始加载完成后再订阅事件
9. **Semaphore panic** — `acquire()` 错误处理而非 unwrap
10. **格式化修复** — `formatEta(0)` 返回 "0s"，`progress` 钳位到 1.0

### P2 - 计划修复

11. 代理密码加密存储（或使用 OS keyring）
12. 前端性能优化（memoize sorted tasks, 减少不必要的重渲染）
13. Shift-select 按视觉顺序选择
14. 速度历史保留时长与图表范围匹配
15. SettingsDialog 补全缺失配置项
16. WebSocket 重连机制统一使用 WsManager

### P3 - 后续迭代

17. 插件系统完整实现（wasmtime 集成）
18. RAR/7Z/TAR.XZ 解压支持
19. ConnectionsTab/TrackerTab 真实数据
20. BT 上传速度统计
21. Chrome 扩展权限收窄

---

## 八、测试覆盖现状

| 模块 | 测试 | 状态 |
|------|------|------|
| `format.ts` | 45 个用例 | ✅ 通过 |
| `taskStore.ts` | 基础操作测试 | ✅ 通过 |
| Rust 后端 | 单元测试 | ⏳ 需 Rust 工具链 |
| 集成测试 | 端到端 | ⏳ 未实现 |

---

## 九、架构评价

### 优点

1. **清晰的分层架构** — Rust 引擎层、IPC 命令层、React UI 层职责分明
2. **多协议统一抽象** — TaskManager 通过 Protocol 枚举统一分发到各引擎
3. **aria2 兼容 API** — JSON-RPC 接口设计便于第三方集成
4. **完善的并发控制** — Semaphore 控制并发数，governor 实现双层限速
5. **前端状态管理** — Zustand store 设计合理，事件驱动更新
6. **虚拟滚动** — 大量任务场景下保持 UI 流畅

### 待改进

1. **锁粒度过粗** — task_manager 和 db 的 Mutex 在高并发下成为瓶颈，考虑 RwLock 或细粒度锁
2. **错误处理不一致** — 部分路径 unwrap/expect，部分路径静默忽略，缺乏统一策略
3. **代码重复** — DB 行映射、前端 API 调用模式存在重复
4. **测试覆盖不足** — 仅前端工具函数有测试，核心引擎和 IPC 层无测试

---

*本报告基于代码静态审查，未包含运行时测试。部分问题（如死锁、竞态）需要在实际运行环境中验证。*
