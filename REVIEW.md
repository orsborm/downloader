# 代码审查报告

> 日期：2026-06-04
> 基线：b3504b7 + 修复
> 审查范围：Rust 后端 (39 files, ~15,380 LOC) + React 前端 (49 files, ~15,600 LOC)
> 测试：617 前端 + 125 Rust = 742 测试

---

## 审查总结

| 状态 | 数量 |
|---|---|
| 安全加固（累计） | 11 项 ✅ |
| 稳定性修复 | 8 项 ✅ |
| UI 修复 | 5 项 ✅ |
| 剩余 LOW | 3 项 |

---

## 本次修复清单

### 安全修复
| 项目 | 文件 | 说明 |
|------|------|------|
| Shell 注入白名单 | `main.rs:73-96` | 黑名单→白名单，阻断所有 shell 元字符 |
| DB JSON 安全访问 | `storage/db.rs:806` | `match` 替代 `unwrap().unwrap()` |
| Plugin 域名白名单 | `plugin/loader.rs:274-287` | http_get 检查 allowed_domains |
| RPC 文件名验证 | `api/rpc.rs:117` | addTask 调用 validate_filename |

### 稳定性修复
| 项目 | 文件 | 说明 |
|------|------|------|
| save_task_params 日志 | `commands/task.rs:76` | `let _ =` → `warn!` |
| 暂停/恢复批量日志 | `commands/task.rs:329,381` | `let _ =` → `warn!` |
| 删除任务日志 | `commands/task.rs:447` | `let _ =` → `warn!` |
| 文件列表保存日志 | `commands/task.rs:751` | `let _ =` → `warn!` |
| 恢复任务状态日志 | `commands/task.rs:520` | `let _ =` → `warn!` |
| 启动恢复日志 | `main.rs:495` | `let _ =` → `warn!` |
| RPC DB 操作日志 | `api/rpc.rs:170,189,208` | `let _ =` → `warn!` |
| resume_all_tasks 竞争 | `commands/task.rs:367-377` | 批量获取 manager 锁 |

### UI 修复
| 项目 | 文件 | 说明 |
|------|------|------|
| 设置页文本换行 | `SettingsDialog.tsx:236` | 添加 `break-words` |
| 任务名 hover 提示 | `TaskList.tsx:276` | 添加 `title={task.name}` |
| URL hover 提示 | `TaskDetail.tsx:105` | 添加 `title={task.url}` |
| 插件描述 hover 提示 | `PluginManager.tsx:169` | 添加 `title` 属性 |
| Toast 定时器清理 | `Toast.tsx:87-91` | 卸载时清除所有 setTimeout |

### 代码清理
| 项目 | 文件 | 说明 |
|------|------|------|
| 删除 Dialog.tsx | `components/Dialog.tsx` | 153 行死代码（无组件导入） |

---

## 生产代码 unwrap() 审计（最终）

| 位置 | 风险 | 说明 |
|------|------|------|
| `http.rs:81-82` | 无风险 | 前置 `len < 2` 守卫 |
| `api/mod.rs:170-175` | 无风险 | 编译时常量 URI parse |
| `api/mod.rs:271,290` | 无风险 | Body::from(&str) 不可失败 |
| `main.rs:646` | 合理 | Tauri 启动失败无法恢复 |
| `plugin/loader.rs:123` | 低风险 | reqwest ClientBuilder，极不可能失败 |

---

## `let _ =` 审计（最终）

所有生产代码 `let _ =` 模式已修复为带日志的 `if let Err(e) =` 模式。剩余 `let _ =` 仅用于：
- `mpsc::send()` — 接收端丢弃时忽略（正确）
- `std::process::Command::spawn()` — 系统命令 fire-and-forget（可接受）
- 测试中的清理操作

---

## 剩余 LOW 级别问题

| # | 文件 | 问题 |
|---|------|------|
| L1 | `engine/http.rs:111` | `expect()` 可改为 `unwrap_or_default()` |
| L2 | `engine/hls/mod.rs:117` | `expect()` 可改为 `unwrap_or_default()` |
| L3 | `plugin/loader.rs:123` | `expect()` 可改为 `unwrap_or_default()` |

---

*审查人：Claude Code Agent*
*最后更新：2026-06-04*
