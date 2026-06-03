# 代码审查报告 — v1.1.3

> 日期：2026-06-04
> 版本：v1.1.3
> 审查范围：Rust 39 files (~15,380 LOC) + React 49 files (~15,600 LOC)
> 测试：617 前端 + 125 Rust = 742 测试

---

## 审查总结

| 状态 | 数量 |
|---|---|
| 安全加固 | 14 项 ✅ |
| 稳定性修复 | 12 项 ✅ |
| UI 修复 | 8 项 ✅ |
| 新增功能 | 2 项（版本管理、KAD） |
| 生产代码 unwrap/expect | **0 个** ✅ |
| 生产代码 `let _ =`（非 channel send） | **0 个** ✅ |

---

## v1.1.3 新增

### KAD (Kademlia DHT)
- 持久 UDP 套接字（Arc\<UdpSocket\>），替代每请求创建新套接字
- 启动时使用 ed2k 服务器列表作为引导节点
- 每 5 分钟定期维护（清理过期节点、刷新路由表）
- 状态栏实时显示 KAD 节点数和引导状态

### 版本管理
- `scripts/version.ps1` 统一管理 package.json + tauri.conf.json + Cargo.toml
- 支持 show/major/minor/patch/set 操作

---

## 安全加固（14 项）

| 项目 | 文件 | 状态 |
|------|------|------|
| Shell 命令注入白名单 | `main.rs` | ✅ |
| CORS localhost 白名单 | `api/mod.rs` | ✅ |
| API Token 认证 | `api/mod.rs` | ✅ |
| SpeedChart XSS 防护 | `SpeedChart.tsx` | ✅ |
| WASM 沙箱 | `plugin/loader.rs` | ✅ |
| 路径遍历防护 | `plugin/loader.rs` + `util/mod.rs` | ✅ |
| DB 初始化安全 | `main.rs` | ✅ |
| DB JSON 安全访问 | `storage/db.rs` | ✅ |
| Plugin 域名白名单 | `plugin/loader.rs` | ✅ |
| RPC 文件名验证 | `api/rpc.rs` | ✅ |
| HTTP 客户端安全降级 | 3 处引擎文件 | ✅ |
| API Response 安全访问 | `api/mod.rs` | ✅ |
| DB 迁移 SQL 日志 | `storage/db.rs` | ✅ |
| 敏感信息混淆 | `storage/config.rs` | ✅ |

---

## 稳定性修复（12 项）

| 项目 | 文件 | 状态 |
|------|------|------|
| Error Boundary | `App.tsx` | ✅ |
| save_task_params 日志 | `commands/task.rs` | ✅ |
| 暂停/恢复批量日志 | `commands/task.rs` | ✅ |
| 删除任务日志 | `commands/task.rs` | ✅ |
| 文件列表保存日志 | `commands/task.rs` | ✅ |
| 恢复任务状态日志 | `commands/task.rs` | ✅ |
| 启动恢复日志 | `main.rs` | ✅ |
| RPC DB 操作日志 | `api/rpc.rs` | ✅ |
| resume_all_tasks 竞争修复 | `commands/task.rs` | ✅ |
| SpeedTracker unwrap 消除 | `engine/http.rs` | ✅ |
| reqwest expect 消除 | 3 处引擎文件 | ✅ |
| Toast 定时器清理 | `Toast.tsx` | ✅ |

---

## UI 修复（8 项）

| 项目 | 文件 | 状态 |
|------|------|------|
| 设置页文本换行 | `SettingsDialog.tsx` | ✅ |
| ed2k 服务器列表布局 | `SettingsDialog.tsx` | ✅ |
| 任务名 hover 提示 | `TaskList.tsx` | ✅ |
| URL hover 提示 | `TaskDetail.tsx` | ✅ |
| 文件路径/peer/tracker/mirror hover | 多个组件 | ✅ |
| 插件描述 hover 提示 | `PluginManager.tsx` | ✅ |
| textarea 可调整大小 | `AddTaskDialog.tsx` + `BatchImportDialog.tsx` | ✅ |
| 删除 Dialog.tsx 死代码 | `components/Dialog.tsx` | ✅ |

---

*审查人：Claude Code Agent*
*最后更新：2026-06-04*
