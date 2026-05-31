# TODO — 迭代 35 计划

> 基于迭代 35 代码审查（REVIEW.md），按优先级排列
> 安全修复（C1-C3, H1, M1-M3）已全部完成

---

## 高优先级 — 稳定性

- [ ] **前端 Error Boundary**：`App.tsx` 添加 React Error Boundary 包裹主内容，防止白屏
- [ ] **Config 敏感信息加密**：`config.rs` api_token/proxy_password 至少 base64 编码存储
- [ ] **DB Mutex 优化**：`storage/db.rs` 将 `tokio::sync::Mutex` 改为 `std::sync::Mutex`（rusqlite 是同步的）

## 中优先级 — 性能与体验

- [ ] **StatusBar 性能优化**：`StatusBar.tsx` 使用 `useMemo` 或 Zustand selector 避免全量遍历
- [ ] **Toolbar 性能优化**：`Toolbar.tsx` 缓存 `Array.from(tasks.values())`
- [ ] **键盘快捷键错误反馈**：`App.tsx` catch 中调用 `handleError` 替代静默吞没
- [ ] **resume_all_tasks 锁优化**：`commands/task.rs` 批量获取待恢复任务减少锁竞争
- [ ] **TaskList useMemo 依赖修复**：`TaskList.tsx:89` 将 searchQuery/statusFilter 加入依赖
- [ ] **SpeedChart rAF 优化**：无下载数据时停止 requestAnimationFrame 循环

## 低优先级 — 代码质量

- [ ] **Toolbar i18n 修复**：`Toolbar.tsx:241` 将 `t("rss.title")` 改为解压相关 key
- [ ] **PluginManager 类型去重**：从 `tauri-api.ts` 导入 PluginInfo
- [ ] **SettingSwitch ARIA**：添加 `role="switch"` 和 `aria-checked`
- [ ] **对话框焦点捕获**：所有对话框添加焦点捕获
- [ ] **右键菜单 ARIA**：添加 `role="menuitem"` 和键盘导航
- [ ] **App.tsx 状态简化**：9 个 useState 改为 reducer 或 openDialog 模式
- [ ] **useTaskEvents 清理优化**：将 notifiedTasks 清理移至 store action
- [ ] **allow_remote 死代码**：实现远程访问控制或移除字段

## 测试补全

- [x] **Rust API 模块测试**：`api/mod.rs` CORS 白名单、JSON-RPC 响应格式、事件序列化
- [x] **Rust 插件加载器测试**：`plugin/loader.rs` 路径遍历防护、WASM 魔数验证、内存限制
- [x] **Rust 存储层基础测试**：`storage/db.rs` TaskState/Protocol 枚举 roundtrip、数据库创建
- [x] **前端 XSS 防护测试**：`speedchart.test.ts` DOM API 安全验证、computeYTicks 输出安全
- [ ] **Rust 核心模块测试**：`engine/task_manager.rs` 状态机、并发控制
- [ ] **Rust 命令层测试**：`commands/task.rs` Tauri IPC 端到端
- [ ] **Rust HTTP 引擎测试**：`engine/http.rs` 下载、断点续传、代理
- [ ] **Rust 解压测试**：`archive/mod.rs` 各格式解压 + 路径安全
- [ ] **前端组件渲染测试**：引入 React Testing Library，覆盖 TaskList/AddTaskDialog/TaskDetail
- [ ] **Windows 打包验证**：`npm run tauri build` 生成便携 ZIP + NSIS
- [ ] **依赖安全审计**：`npm audit` + `cargo audit`

## 已知限制

- BT 引擎核心下载待集成 librqbit API 完整流程
- ed2k KAD 实际网络连接待真实环境测试
- 浏览器扩展与 Rust 后端联调待完成
- RSS/Archive 模块需集成到 Tauri 命令层

---

*最后更新：2026-05-31*
