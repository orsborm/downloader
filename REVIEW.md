# 代码审查报告 — 迭代 31

> 日期：2026-05-31
> 审查范围：Rust 后端 (src-tauri/src/) + 前端 (src/) + 测试覆盖
> 总测试数：804 (前端) + 57 (Rust) = 861

---

## 审查总结

| 严重级别 | Rust 后端 | 前端 | 合计 |
|---|---|---|---|
| CRITICAL | 2 | 1 | 3 |
| HIGH | 4 | 5 | 9 |
| MEDIUM | 6 | 8 | 14 |
| LOW | 4 | 4 | 8 |
| **合计** | **16** | **18** | **34** |

---

## CRITICAL 级别

### C1. WASM 插件沙箱路径遍历
- **文件**：`plugin/loader.rs:257-273`
- **问题**：`read_file`/`write_file` 宿主函数仅检查 `".."` 子串，允许绝对路径读写（如 `C:\Windows\System32\...`）完全绕过沙箱
- **修复**：使用 `Path::canonicalize()` 后检查是否在允许目录内，拒绝绝对路径和符号链接逃逸

### C2. WASM 宿主函数死锁风险
- **文件**：`plugin/loader.rs:216`
- **问题**：同步 wasmtime 宿主函数内调用 `tokio::runtime::Handle::current().block_on()`，若在 Tokio 异步上下文中调用 `execute_plugin` 会死锁
- **修复**：将 WASM 宿主函数改为异步，或使用 `spawn_blocking` 隔离同步调用

### C3. 前端 XSS 风险
- **文件**：`SpeedChart.tsx:215`
- **问题**：`tooltip.innerHTML` 使用模板字符串注入格式化数据，若 `formatSpeed`/`formatTimeLabel` 返回值被污染可导致 XSS
- **修复**：改用 `document.createElement` + `textContent` 或 React ref 方案

---

## HIGH 级别

### H1. JSON-RPC API CORS 全开
- **文件**：`api/mod.rs:156`
- **问题**：`CorsLayer::permissive()` 允许任意来源发起请求，任意网页可对下载器发起 CSRF 攻击
- **修复**：限制为 `localhost` 来源或配置白名单

### H2. API 默认无认证
- **文件**：`api/mod.rs:183-213`
- **问题**：`api_token` 默认为空（`config.rs:144`），任何本地进程或浏览器页面可无凭据调用所有 RPC 方法
- **修复**：首次启动自动生成随机 token，空 token 时拒绝所有请求

### H3. 7z 解压路径遍历
- **文件**：`archive/mod.rs:603-607`
- **问题**：`extract_7z` 直接调用 `sevenz_rust::decompress_file`，未像 zip/tar/rar 那样执行 `validate_safe_path` 检查
- **修复**：在解压前对每个条目路径执行 `validate_safe_path`

### H4. ed2k 无限递归栈溢出
- **文件**：`engine/ed2k/mod.rs:601-603`
- **问题**：`receive_piece` 在 `OP_QUEUERANK` 包上无限递归，恶意 peer 发送大量队列排名响应可导致栈溢出
- **修复**：改为循环处理或设置递归深度限制

### H5. 前端无 Error Boundary
- **文件**：`App.tsx`（整体）
- **问题**：无 React Error Boundary，任何子组件渲染异常导致整个应用白屏
- **修复**：在主内容区域包裹 `ErrorBoundary` 组件

### H6. StatusBar 每次渲染遍历全部任务
- **文件**：`StatusBar.tsx:12`
- **问题**：`getGlobalStats()` 每次渲染调用，遍历整个 task Map；速度更新每秒触发多次重渲染
- **修复**：使用 `useMemo` 或 Zustand selector 订阅派生状态

### H7. Toolbar 重复创建数组
- **文件**：`Toolbar.tsx:60-62, 79-88`
- **问题**：`Array.from(tasks.values())` 每次渲染计算两次（hasActive + filteredCount），速度 tick 触发频繁重渲染
- **修复**：使用 `useMemo` 缓存计算结果

### H8. 键盘快捷键错误静默吞没
- **文件**：`App.tsx:163, 175, 178`
- **问题**：`removeTask`/`pauseTask`/`resumeTask` 的 `.catch(() => {})` 吞没错误，用户无反馈
- **修复**：catch 中调用统一错误处理 `handleError`

### H9. useTaskEvents 不必要的清理触发
- **文件**：`useTaskEvents.ts:21`
- **问题**：订阅 `tasks` Map 仅用于清理 `notifiedTasks`，每次速度更新都触发 useEffect 遍历
- **修复**：将清理逻辑移至 store action 内，或使用 `useRef` 避免订阅

---

## MEDIUM 级别

| # | 文件 | 问题 | 修复建议 |
|---|---|---|---|
| M1 | `main.rs:64` | `Database::new().expect()` 在 DB 损坏/锁定时 panic，无恢复路径 | 改为 `?` 传播 + 用户提示 |
| M2 | `main.rs:188-200` | 数据库持久化失败被 `let _ =` 静默丢弃，掩盖数据丢失 | 至少记录 warning 日志 |
| M3 | `plugin/loader.rs:217-229` | `http_get` 宿主函数获取响应体后未写回 WASM 内存，插件无法使用数据 | 将响应体写入 WASM 线性内存 |
| M4 | `config.rs:54` | `api_token` 和 `proxy_password` 明文存储在 config.toml | 使用系统 keyring 或至少 base64 编码 |
| M5 | `storage/db.rs` | `tokio::sync::Mutex` 包裹同步 rusqlite Connection，调度开销不必要 | 改用 `std::sync::Mutex` |
| M6 | `commands/task.rs:306-320` | `resume_all_tasks` 每个任务单独加锁检查 `has_task`，O(n) 锁竞争 | 批量获取待恢复任务列表 |
| M7 | `main.tsx:7` | `document.getElementById("root") as HTMLElement` 不安全强制转换 | 添加 null 检查 |
| M8 | `BatchImportDialog.tsx:266` | `parseUrls()` 在渲染期间调用，每次创建新数组 | 改用 `useMemo` |
| M9 | `TaskList.tsx:89` | `getSortedTasks()` 的 useMemo 依赖不完整，`searchQuery`/`statusFilter` 变化时返回旧数据 | 将这些值加入依赖数组 |
| M10 | `PluginManager.tsx:10-18` | `PluginInfo` 接口与 `tauri-api.ts` 重复定义 | 从共享位置导入 |
| M11 | `SettingsDialog.tsx:247-259` | `SettingSwitch` 缺少 `role="switch"` 和 `aria-checked` | 添加 ARIA 属性 |
| M12 | 所有对话框组件 | 无焦点捕获，Tab 可移到遮罩层后方 | 使用 Radix UI 内置焦点管理或 `focus-trap-react` |
| M13 | `SpeedChart.tsx:228-239` | `requestAnimationFrame` 循环在无下载时仍持续运行 | 数据过期时 early-exit |
| M14 | `TaskList.tsx:329-438` | 右键菜单缺少 `role="menuitem"` 和键盘导航 | 添加 ARIA 角色和方向键/Esc 处理 |

---

## LOW 级别

| # | 文件 | 问题 | 修复建议 |
|---|---|---|---|
| L1 | `api/mod.rs:26` | `allow_remote` 配置字段定义但未使用，死代码 | 实现远程访问控制或移除字段 |
| L2 | `engine/http.rs:70-71` | `SpeedTracker::speed()` 中 `.unwrap()` 在 `len() < 2` 检查后调用，守卫条件脆弱 | 改用 `.expect()` 带说明或 `if let` |
| L3 | `archive/mod.rs:325,372,419,466` | tar 头部大小解析失败时 `.unwrap_or(0)` 静默返回零 | 记录 warning 日志 |
| L4 | `App.tsx:36-45` | 9 个独立 `useState` 控制对话框可见性 | 改用 reducer 或 `openDialog: string \| null` |
| L5 | `Toolbar.tsx:241` | Archive 按钮 tooltip 使用 `t("rss.title")` 而非解压相关 key，复制粘贴错误 | 改为正确的 i18n key |
| L6 | `Toolbar.tsx:91` | 不必要的动态 `import()`，模块已静态导入 | 移除动态导入 |
| L7 | `useTaskEvents.ts:11` | 模块级 `notifiedTasks` Set 长期运行时无限增长 | 定期清理或基于任务生命周期管理 |
| L8 | `tauri-api.ts` | `getAllTasks` 同时有静态和动态导入，模式不一致 | 统一为静态导入 |

---

## 测试覆盖分析

### 前端（804 测试，17 个测试文件）

覆盖良好的模块：format (144), components (114), taskStore (95), tauri-api (83), utils (47)

**无测试覆盖的组件**：
- `App.tsx` — 主应用组件
- `ArchiveDialog.tsx`, `BatchImportDialog.tsx`, `DownloadHistoryDialog.tsx`
- `PluginManager.tsx`, `RssManager.tsx`, `SettingsDialog.tsx`
- `StatusBar.tsx`, `TaskDetail.tsx`, `Toolbar.tsx`

注意：`components.test.ts` 仅测试提取的工具逻辑（解析、颜色映射），不包含组件渲染测试。

### Rust 后端（57 测试，9 个文件有测试，30 个文件无测试）

有测试的模块：util, schedule, rss/rules, rss/feed, hls/m3u8, ed2k/hash, ed2k/mod, ed2k/proto, ed2k/tag

**高风险无测试模块**：
1. `engine/task_manager.rs` — 核心下载编排、状态机、并发控制
2. `storage/db.rs` — SQLite 持久化、崩溃恢复
3. `commands/task.rs` — Tauri 命令桥接层
4. `engine/http.rs` — HTTP 下载引擎
5. `archive/mod.rs` + `archive/password.rs` — 解压（安全敏感）
6. `plugin/loader.rs` + `plugin/api.rs` — 插件加载（任意代码执行面）
7. `api/websocket.rs` + `api/rpc.rs` — 实时通信层

---

## 架构建议

1. **错误处理统一**：Rust 端多处 `let _ =` 丢弃错误，应建立统一的错误上报机制（至少日志 + 前端通知）
2. **安全基线**：WASM 沙箱路径验证、API 认证、CORS 策略需优先修复，当前状态下本地攻击面过大
3. **前端性能**：StatusBar/Toolbar 的频繁全量 Map 遍历应通过 Zustand selector 优化，避免每秒多次无意义重渲染
4. **测试分层**：建议引入集成测试层（Tauri IPC 端到端），当前仅覆盖工具函数，核心业务逻辑零覆盖

---

*审查完成：2026-05-31*
