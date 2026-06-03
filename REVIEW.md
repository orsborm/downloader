# 代码审查报告 — 迭代 35（全面审查）

> 日期：2026-06-03
> 审查范围：Rust 后端 (38 files, ~14,500 LOC) + React 前端 (24 production + 17 test files, ~8,000 LOC)
> 基线：迭代 34 安全测试
> 测试：624 前端测试全部通过，Rust 测试已写入（cargo 未安装）

---

## 审查总结

| 状态 | 数量 |
|---|---|
| 安全加固已完成 | 9 项 |
| HIGH 级别问题 | 4 |
| MEDIUM 级别问题 | 11 |
| LOW 级别问题 | 8 |

---

## 安全加固项（已确认完成）

| 项目 | 文件 | 状态 |
|------|------|------|
| CORS 白名单 | `api/mod.rs:157-175` | ✅ AllowOrigin::list 仅 localhost |
| API Token 认证 | `api/mod.rs:205-251` | ✅ 空 token 拒绝所有请求 |
| SpeedChart XSS | `SpeedChart.tsx:210-232` | ✅ DOM API + textContent |
| WASM 沙箱 | `plugin/loader.rs` | ✅ 64MB 内存限制 + 路径遍历防护 |
| 路径遍历防护 | `util/mod.rs:143-183` | ✅ canonicalize + 目录白名单 |
| 7z 路径遍历 | `archive/mod.rs` | ✅ validate_safe_path |
| ed2k 无限递归 | `engine/ed2k/mod.rs` | ✅ 循环 + 最大重试 60 次 |
| DB 初始化 panic | `main.rs:64-68` | ✅ map_err + ? 传播 |
| 敏感信息混淆 | `storage/config.rs:346-358` | ✅ XOR + hex 编码（非明文） |

---

## HIGH 级别问题

### H1. Shell 命令注入过滤不完整
- **文件**: `main.rs:73-92`
- **问题**: `run_command` 的危险字符黑名单缺少 `!`, `~`, `#`, `\n`, `\r`, `'`, `[`, `]` 等 shell 元字符。攻击者可通过 Unicode 同形字或未列入的字符绕过。
- **建议**: 改用白名单方式（仅允许字母数字和有限符号），或使用参数数组而非 `cmd /C` 拼接。

### H2. HTTP 客户端 expect() 可能 panic
- **文件**: `plugin/loader.rs:121`, `engine/http.rs:111`, `engine/hls/mod.rs:117`
- **问题**: reqwest ClientBuilder 使用 `.expect()` 而非 `?` 传播。若 TLS 后端不可用会直接崩溃。
- **建议**: 改为 `.unwrap_or_default()` 或 `?` 传播。

### H3. DB JSON 操作链式 unwrap
- **文件**: `storage/db.rs:806`
- **问题**: `meta.get_mut("mirrorUrls").unwrap().as_array_mut().unwrap()` 在非测试代码中使用，若 JSON 结构异常会 panic。
- **建议**: 改用 `if let Some(...)` 或 `?` 操作符。

### H4. 前端缺少组件级 Error Boundary
- **文件**: `App.tsx:35-72`
- **问题**: 仅根级别有 ErrorBoundary。Dialog/TaskList/SpeedChart 崩溃会导致整个白屏。
- **建议**: 在 TaskList+TaskDetail 区域、SpeedChart、lazy-loaded Dialog 外层各加 ErrorBoundary。

---

## MEDIUM 级别问题

### M1. App.tsx 全局 re-render 传播
- **文件**: `App.tsx:81-83`
- **问题**: `useTaskStore((s) => s.tasks)` 订阅整个 Map 引用，每次任务更新都触发 App 及所有子组件 re-render。
- **建议**: App 组件不直接订阅 `tasks`，改为在子组件内用细粒度 selector。

### M2. 任务参数持久化失败被静默丢弃
- **文件**: `commands/task.rs:76`
- **问题**: `let _ = db.save_task_params(...)` 静默丢弃错误，重启后可能丢失代理/认证配置。
- **建议**: 改为 `if let Err(e) = ... { warn!(...) }`。

### M3. StatusBar 无条件轮询 BT 状态
- **文件**: `StatusBar.tsx:68`
- **问题**: 每 5 秒轮询 `getBtStatus()`，即使 BT 未启用。
- **建议**: 仅在 BT 引擎激活时启动轮询。

### M4. DB Mutex 类型不当
- **文件**: `storage/db.rs`
- **问题**: rusqlite 是同步库，使用 `tokio::sync::Mutex` 包裹会不必要地占用异步运行时。
- **建议**: 改为 `std::sync::Mutex`。

### M5. resume_all_tasks 锁竞争
- **文件**: `commands/task.rs`
- **问题**: 批量恢复任务时逐个获取锁，O(n) 锁竞争。
- **建议**: 批量获取待恢复任务列表，一次性处理。

### M6. Dialog.tsx 为死代码
- **文件**: `components/Dialog.tsx` (153 行)
- **问题**: 无任何组件导入此文件。所有对话框各自构建 overlay。
- **建议**: 统一使用 Dialog 组件或删除此文件。

### M7. PasswordManager 未连接
- **文件**: `archive/password.rs`
- **问题**: `PasswordManager` 定义了 `find_for_file`、`sorted_by_usage`、`record_use` 方法，但无生产代码调用。
- **建议**: 连接到 ArchiveManager 或删除。

### M8. HostApi trait 未实现
- **文件**: `plugin/api.rs:98-119`
- **问题**: `HostApi` trait 定义了接口但无任何实现。
- **建议**: 实现或标记为 future work 并删除。

### M9. TaskList useMemo 依赖不完整
- **文件**: `TaskList.tsx:109`
- **问题**: `useMemo` 依赖 Map 引用，每次 applyUpdate 都重算排序。
- **建议**: 依赖具体影响排序的字段而非整个 Map 引用。

### M10. 对话框缺少 ARIA 属性
- **涉及**: 所有自定义对话框（10+ 个）
- **问题**: 缺少 `role="dialog"`、`aria-modal="true"`、关闭按钮 `aria-label`。
- **建议**: 统一使用 Dialog 组件或逐个添加 ARIA 属性。

### M11. Tab 面板缺少 ARIA 语义
- **涉及**: `TaskDetail.tsx:57-73`、`SettingsDialog.tsx:162-177`
- **问题**: 缺少 `role="tablist"`、`role="tab"`、`role="tabpanel"`、`aria-selected`。

---

## LOW 级别问题

| # | 文件 | 问题 |
|---|------|------|
| L1 | `main.tsx:2` | 多余 `import React` (现代 JSX transform 不需要) |
| L2 | `lib/i18n.ts` | `STORAGE_KEY` 导出从未被导入 |
| L3 | `lib/types.ts` | 仅 re-export `shared/types.ts`，不必要的间接层 |
| L4 | `TaskList.tsx:298-309` | 进度条缺少 `role="progressbar"` |
| L5 | `TaskList.tsx:448` | 右键菜单缺少键盘箭头导航 |
| L6 | `SpeedChart.tsx` | Canvas 无屏幕阅读器文本替代 |
| L7 | `StatusBar.tsx:68` | 无 landmark roles (`<main>`, `<nav>`, `<header>`) |
| L8 | `Toast.tsx:105` | 关闭按钮缺少 `aria-label` |

---

## 架构优势

- **模块化清晰**: engine/storage/api/plugin/rss/schedule/archive 各司其职
- **错误处理一致**: `anyhow::Result` 内部 + `Result<T, String>` IPC 边界
- **测试覆盖良好**: 前端 624 测试 (17 files)、Rust 9 个模块含 `#[cfg(test)]`
- **安全意识强**: 已修复 9 项安全问题 + 多层防御
- **性能考虑到位**: 虚拟滚动 (@tanstack/react-virtual) + Canvas 渲染 (60fps)
- **无 unsafe 代码**: 全代码库无 unsafe 块
- **XSS 防护**: 无 dangerouslySetInnerHTML/innerHTML/eval

---

## 测试覆盖

| 层 | 测试数 | 覆盖模块 |
|----|--------|----------|
| 前端 | 624 | components, stores, format, errors, i18n, hooks, tauri-api |
| Rust API | 5 | CORS 白名单、JSON-RPC 格式、事件序列化 |
| Rust 插件 | 7 | 路径遍历防护、WASM 验证、内存限制 |
| Rust 存储 | 6 | 枚举 roundtrip、DB 创建与大小 |
| **待补全** | - | task_manager、commands/task、http engine、archive 解压 |

---

*审查人：Claude Code Agent*
*最后更新：2026-06-03*
