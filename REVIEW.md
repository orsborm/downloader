# 代码审查报告 — 迭代 36（修复后审查）

> 日期：2026-06-03
> 审查范围：迭代 36 修复验证 + 剩余问题评估
> 基线：迭代 35 全面审查
> 测试：616 前端测试全部通过

---

## 审查总结

| 状态 | 数量 |
|---|---|
| 迭代 36 已修复 | 8 项 |
| 安全加固已完成（累计） | 9 项 |
| 剩余 MEDIUM | 7 项 |
| 剩余 LOW | 5 项 |

---

## 迭代 36 修复验证

### H1. Shell 命令注入加固 ✅
- **文件**: `main.rs:73-92`
- **验证**: 改用白名单方式，仅允许 `a-zA-Z0-9 .-_/:=@[],+~`
- **安全性**: 阻断所有 shell 元字符 (`|;&$`(){}<>!\"'#*?%`)，500 字符长度限制
- **评估**: 安全。`~` 允许是合理的（非攻击向量），`[]` 允许支持路径 glob

### H2. HTTP 客户端 expect() → 安全降级 ✅
- **文件**: `plugin/loader.rs:121`、`engine/http.rs:106-111`、`engine/hls/mod.rs:114-117`
- **验证**: 3 处 `.expect()` 改为 `.unwrap_or_default()` 或 `.unwrap_or_else(|e| { warn!(...); default() })`
- **评估**: 安全。reqwest Client 构建失败时降级为默认客户端，无 panic 风险

### H3. DB JSON 链式 unwrap 修复 ✅
- **文件**: `storage/db.rs:800-810`
- **验证**: `unwrap().unwrap()` 改为 `match ... { Some(arr) => arr, None => return Ok(()) }`
- **评估**: 安全。双层防御：先 `is_some()` 确保字段存在，再 `match` 安全访问

### H4. 前端 Error Boundary ✅
- **文件**: `App.tsx` + 新增 `SectionErrorBoundary.tsx`
- **验证**: TaskList、SpeedChart、TaskDetail、DownloadHistory 各有独立 ErrorBoundary
- **评估**: 正确。局部崩溃不再导致全白屏

### M1. App.tsx re-render 优化 ✅
- **文件**: `App.tsx:82-84`
- **验证**: 移除 `s.tasks` 订阅，改用 `getState()` 获取 selectedTask
- **评估**: 正确。TaskDetail 已同步修复为直接订阅 store 获取实时数据

### M2. 任务参数持久化日志 ✅
- **文件**: `commands/task.rs:75-78`
- **验证**: `let _ =` 改为 `if let Err(e) = ... { warn!(...) }`
- **评估**: 正确。warn 级别适合此非致命错误

### M6. Dead code 清理 ✅
- **删除**: `Dialog.tsx` (153 行)，无组件导入
- **清理**: `lib/i18n.ts` 移除未使用的 `STORAGE_KEY` 导出
- **验证**: 无断裂导入

### 版本号管理机制 ✅
- **新增**: `scripts/version.ps1` 支持 show/major/minor/patch/set
- **同步**: package.json + tauri.conf.json + Cargo.toml 三处版本号

---

## 安全加固项（全部确认）

| 项目 | 文件 | 状态 |
|------|------|------|
| Shell 命令注入白名单 | `main.rs:73-92` | ✅ |
| CORS 白名单 | `api/mod.rs:157-175` | ✅ |
| API Token 认证 | `api/mod.rs:205-251` | ✅ |
| SpeedChart XSS | `SpeedChart.tsx:210-232` | ✅ |
| WASM 沙箱 | `plugin/loader.rs` | ✅ |
| 路径遍历防护 | `util/mod.rs` + `plugin/loader.rs` | ✅ |
| DB 初始化 panic | `main.rs:64-68` | ✅ |
| 敏感信息混淆 | `storage/config.rs:346-358` | ✅ |
| HTTP 客户端安全降级 | 3 处引擎文件 | ✅ |
| DB JSON 安全访问 | `storage/db.rs:806` | ✅ |

---

## 剩余 MEDIUM 级别问题（7 项）

| # | 文件 | 问题 | 建议 |
|---|------|------|------|
| M3 | `StatusBar.tsx:68` | BT 轮询无条件执行 | 仅 BT 激活时启动 |
| M4 | `storage/db.rs` | tokio::sync::Mutex 包裹同步 rusqlite | 改为 std::sync::Mutex |
| M5 | `commands/task.rs` | resume_all_tasks O(n) 锁竞争 | 批量获取 |
| M7 | `archive/password.rs` | PasswordManager 未连接生产代码 | 连接或删除 |
| M8 | `plugin/api.rs:98-119` | HostApi trait 未实现 | 实现或删除 |
| M9 | `TaskList.tsx:109` | useMemo 依赖 Map 引用 | 优化依赖 |
| M10 | 所有对话框 | 缺少 ARIA 属性 | 添加 role="dialog" 等 |

---

## 剩余 LOW 级别问题（5 项）

| # | 文件 | 问题 |
|---|------|------|
| L1 | `main.tsx:2` | 多余 `import React` |
| L2 | `TaskList.tsx:298-309` | 进度条缺少 `role="progressbar"` |
| L3 | `TaskList.tsx:448` | 右键菜单缺少键盘导航 |
| L4 | `SpeedChart.tsx` | Canvas 无屏幕阅读器替代 |
| L5 | `SectionErrorBoundary.tsx` | 硬编码中文错误文本 |

---

## 生产代码 unwrap() 审计

| 位置 | 类型 | 风险 |
|------|------|------|
| `api/mod.rs:170-175` | 编译时常量 URI parse | 无风险 |
| `api/mod.rs:271,290` | 静态 Response builder | 极低风险 |
| `engine/http.rs:81-82` | 前置 `len < 2` 守卫 | 无风险 |
| `main.rs:646` | Tauri 启动失败 | 合理 panic |
| `plugin/loader.rs:121` | unwrap_or_default | 无风险 |
| `engine/http.rs:106-111` | unwrap_or_else + 日志 | 无风险 |
| `engine/hls/mod.rs:114-117` | unwrap_or_default | 无风险 |
| `storage/db.rs:806` | match 安全访问 | 无风险 |

**结论**: 所有生产代码 unwrap 均已安全处理或有合理守卫。

---

## 架构优势（确认）

- 模块化清晰：engine/storage/api/plugin/rss/schedule/archive 各司其职
- 错误处理一致：`anyhow::Result` 内部 + `Result<T, String>` IPC 边界
- 测试覆盖良好：前端 616 测试 (17 files)、Rust 9 个模块含 `#[cfg(test)]`
- 无 `unsafe` 代码、无 `dangerouslySetInnerHTML`/`innerHTML`/`eval`
- 安全防御多层：CORS 白名单 + Token 认证 + WASM 沙箱 + 路径遍历防护 + 命令白名单
- 版本管理：三文件同步 + 打包脚本

---

*审查人：Claude Code Agent*
*最后更新：2026-06-03*
