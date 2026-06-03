# TODO — 迭代 37 计划

> 基于迭代 36 修复后审查（REVIEW.md），按优先级排列

---

## 高优先级 — 安全与稳定性

所有 HIGH 级别问题已在迭代 36 修复。

## 中优先级 — 性能与质量

- [ ] **StatusBar BT 轮询优化**：`StatusBar.tsx:68` 仅在 BT 引擎激活时启动 `getBtStatus()` 轮询
- [ ] **DB Mutex 类型优化**：`storage/db.rs` 将 `tokio::sync::Mutex` 改为 `std::sync::Mutex`（rusqlite 是同步的）
- [ ] **resume_all_tasks 锁优化**：`commands/task.rs` 批量获取待恢复任务减少锁竞争
- [ ] **PasswordManager 连接**：`archive/password.rs` 连接到 ArchiveManager 或删除死代码
- [ ] **HostApi trait**：`plugin/api.rs` 实现或删除未使用的 trait 定义
- [ ] **TaskList useMemo 依赖修复**：`TaskList.tsx:109` 优化依赖，避免每次 Map 引用变化都重算
- [ ] **对话框 ARIA**：所有自定义对话框添加 `role="dialog"`、`aria-modal="true"`、关闭按钮 `aria-label`

## 低优先级 — 可访问性与代码整洁

- [ ] **main.tsx 清理**：删除多余的 `import React`
- [ ] **进度条 ARIA**：`TaskList.tsx` 添加 `role="progressbar"`、`aria-valuenow/min/max`
- [ ] **右键菜单键盘导航**：`TaskList.tsx` 添加箭头键导航
- [ ] **Canvas 屏幕阅读器**：`SpeedChart.tsx` 添加 aria-label 或 sr-only 文本
- [ ] **SectionErrorBoundary i18n**：`SectionErrorBoundary.tsx` 硬编码中文改为双语

## 测试补全

- [x] **Rust API 模块测试**：`api/mod.rs` CORS 白名单、JSON-RPC 响应格式、事件序列化
- [x] **Rust 插件加载器测试**：`plugin/loader.rs` 路径遍历防护、WASM 魔数验证、内存限制
- [x] **Rust 存储层基础测试**：`storage/db.rs` TaskState/Protocol 枚举 roundtrip、数据库创建
- [x] **前端 XSS 防护测试**：`speedchart.test.ts` DOM API 安全验证
- [ ] **Rust 核心模块测试**：`engine/task_manager.rs` 状态机、并发控制
- [ ] **Rust 命令层测试**：`commands/task.rs` Tauri IPC 端到端
- [ ] **Rust HTTP 引擎测试**：`engine/http.rs` 下载、断点续传、代理
- [ ] **前端组件渲染测试**：引入 React Testing Library，覆盖 TaskList/AddTaskDialog/TaskDetail
- [ ] **Windows 打包验证**：`npm run tauri build` 生成便携 ZIP + NSIS
- [ ] **依赖安全审计**：`npm audit` + `cargo audit`

## 已知限制

- BT 引擎核心下载待集成 librqbit API 完整流程
- ed2k KAD 实际网络连接待真实环境测试
- 浏览器扩展与 Rust 后端联调待完成
- RSS/Archive 模块需集成到 Tauri 命令层

---

*最后更新：2026-06-03*
