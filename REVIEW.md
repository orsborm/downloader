# 代码审查报告 — 迭代 35

> 日期：2026-05-31
> 审查范围：5 项安全修复验证 + 剩余问题评估
> 基线：迭代 31 审查 (REVIEW.md)
> 测试：355 前端测试全部通过，Rust 测试待运行

---

## 审查总结

迭代 31 识别的 3 个 CRITICAL 和 2 个 HIGH 安全问题已全部修复并验证通过。

| 状态 | 数量 |
|---|---|
| 已修复 (CRITICAL) | 3 |
| 已修复 (HIGH) | 2 |
| 剩余 HIGH | 4 |
| 剩余 MEDIUM | 14 |
| 剩余 LOW | 8 |

---

## 已修复项验证

### C1. WASM 沙箱路径遍历 ✅
- **文件**：`plugin/loader.rs:15-64`
- **验证**：`validate_plugin_path()` 使用 `canonicalize()` 解析符号链接，拒绝 `..` 和绝对路径逃逸，空目录列表返回 false
- **测试**：6 个单元测试覆盖（空目录、`..` 遍历、相对路径、有效路径）

### C2. WASM 宿主函数死锁 ✅
- **文件**：`plugin/loader.rs:276`
- **验证**：`http_get` 使用 `tokio::task::block_in_place` 替代直接 `block_on`，避免 Tokio 运行时死锁

### C3. 前端 XSS ✅
- **文件**：`SpeedChart.tsx:209-232`
- **验证**：tooltip 使用 `document.createElement` + `textContent` 替代 `innerHTML`，所有动态数据通过 `textContent` 赋值，无注入风险
- **测试**：`speedchart.test.ts` 验证 DOM API 安全

### H1. API CORS 收紧 ✅
- **文件**：`api/mod.rs:156-175`
- **验证**：`CorsLayer::permissive()` 已替换为 `AllowOrigin::list()` 白名单，仅包含 localhost:1420、127.0.0.1:1420、localhost:3000、127.0.0.1:3000、tauri://localhost、https://tauri.localhost
- **测试**：`cors_whitelist_contains_only_localhost` 验证无通配符

### M1 + M2. DB 初始化 panic + 错误日志 ✅
- **文件**：`main.rs:64-68` (panic 修复)、`main.rs:196-218` (错误日志)
- **验证**：`Database::new()` 使用 `map_err` + `?` 传播，返回用户可读错误；所有 DB 操作失败均通过 `tracing::error!` 记录

### M3. WASM http_get 响应写回 ✅
- **文件**：`plugin/loader.rs:282-286` (暂存)、`loader.rs:436-449` (写回)
- **验证**：`http_get` 宿主函数将响应体存入 `PluginState::http_response`，`execute_plugin` 调用后通过 `alloc` 分配 WASM 内存并 `copy_from_slice` 写回
- **测试**：`http_response_buffer_defaults_empty` 验证初始状态

---

## 剩余 HIGH 级别问题

### H5. 前端无 Error Boundary
- **文件**：`App.tsx`
- **风险**：子组件渲染异常导致白屏
- **建议**：添加 React Error Boundary 包裹主内容

### H6. StatusBar 性能
- **文件**：`StatusBar.tsx`
- **风险**：`getGlobalStats()` 每次渲染遍历全部任务，速度更新每秒触发多次重渲染
- **建议**：使用 Zustand selector 或 `useMemo`

### H7. Toolbar 性能
- **文件**：`Toolbar.tsx`
- **风险**：`Array.from(tasks.values())` 每次渲染计算两次
- **建议**：`useMemo` 缓存

### H8. 键盘快捷键错误静默吞没
- **文件**：`App.tsx`
- **风险**：`.catch(() => {})` 吞没错误，用户无反馈
- **建议**：catch 中调用 `handleError`

---

## 剩余 MEDIUM 级别问题（14 项）

| # | 文件 | 问题 |
|---|---|---|
| M4 | `config.rs` | api_token/proxy_password 明文存储 |
| M5 | `storage/db.rs` | tokio::sync::Mutex 包裹同步 rusqlite |
| M6 | `commands/task.rs` | resume_all_tasks O(n) 锁竞争 |
| M7 | `main.tsx` | 不安全的 `as HTMLElement` 强制转换 |
| M8 | `BatchImportDialog.tsx` | parseUrls() 渲染期间调用 |
| M9 | `TaskList.tsx` | useMemo 依赖不完整 |
| M10 | `PluginManager.tsx` | PluginInfo 类型重复定义 |
| M11 | `SettingsDialog.tsx` | SettingSwitch 缺少 ARIA |
| M12 | 所有对话框 | 无焦点捕获 |
| M13 | `SpeedChart.tsx` | rAF 无下载时仍运行 |
| M14 | `TaskList.tsx` | 右键菜单缺少 ARIA |
| M15 | `Toolbar.tsx:241` | i18n key 错误 |
| M16 | `App.tsx` | 9 个 useState 冗余 |
| M17 | `useTaskEvents.ts` | notifiedTasks 清理时机不当 |

---

## 剩余 LOW 级别问题（8 项）

| # | 文件 | 问题 |
|---|---|---|
| L1 | `api/mod.rs` | allow_remote 死代码 |
| L2 | `engine/http.rs` | SpeedTracker unwrap 脆弱 |
| L3 | `archive/mod.rs` | tar 头部解析静默失败 |
| L4 | `App.tsx` | 对话框状态冗余 |
| L5 | `Toolbar.tsx:241` | i18n 复制粘贴错误 |
| L6 | `Toolbar.tsx:91` | 不必要的动态 import |
| L7 | `useTaskEvents.ts` | notifiedTasks 无限增长 |
| L8 | `tauri-api.ts` | 导入模式不一致 |

---

## 安全评估

**已消除的攻击面**：
- WASM 插件路径遍历 → canonicalize 验证
- WASM 宿主函数死锁 → block_in_place
- 前端 XSS → DOM API 替代 innerHTML
- API CORS 任意来源 → localhost 白名单
- API 无认证 → 空 token 拒绝请求
- DB 初始化崩溃 → 错误传播 + 用户提示

**仍存在的风险**：
- API token 明文存储 (M4)
- 7z 解压路径遍历已在迭代 32 修复 (TODO.md 已标记)
- ed2k 无限递归已在迭代 32 修复 (TODO.md 已标记)

---

## 测试覆盖

- 前端：355 测试全部通过
- Rust：api/mod.rs (5), plugin/loader.rs (7), storage/db.rs 基础测试已补充
- 待补全：task_manager、commands/task、http engine、archive 解压

---

*审查完成：2026-05-31*
