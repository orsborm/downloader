# TODO — 迭代 32 计划

> 基于迭代 31 代码审查（REVIEW.md），按优先级排列

---

## 高优先级 — 安全修复

- [x] **WASM 沙箱路径遍历修复**：`plugin/loader.rs` 使用 `canonicalize()` 验证路径在允许目录内，拒绝绝对路径
- [x] **WASM 宿主函数死锁修复**：`plugin/loader.rs` 将 `block_on` 改为 `block_in_place` 避免 tokio 死锁
- [x] **7z 解压路径遍历修复**：`archive/mod.rs` 对 7z 条目执行 `validate_safe_path`，移除逃逸文件
- [x] **ed2k 无限递归修复**：`engine/ed2k/mod.rs` 改为循环处理 `OP_QUEUERANK`，最多重试 60 次
- [x] **API 认证默认启用**：`storage/config.rs` + `api/mod.rs` 首次启动自动生成 token，空 token 拒绝请求
- [x] **API CORS 收紧**：`api/mod.rs` 将 `CorsLayer::permissive()` 替换为 localhost 白名单
- [x] **SpeedChart XSS 修复**：`SpeedChart.tsx` 改用 DOM API 替代 `innerHTML`

## 中优先级 — 稳定性与质量

- [x] **DB 初始化 panic 修复**：`main.rs:64` 改为 `?` 传播 + 用户错误提示
- [x] **数据库错误日志**：`main.rs:188-200` 恢复 `let _ =` 为日志记录
- [x] **WASM http_get 修复**：`plugin/loader.rs` 将响应体写回 WASM 线性内存
- [ ] **前端 Error Boundary**：`App.tsx` 添加 React Error Boundary 包裹主内容
- [ ] **StatusBar 性能优化**：`StatusBar.tsx` 使用 `useMemo` 或 Zustand selector
- [ ] **Toolbar 性能优化**：`Toolbar.tsx` 缓存 `Array.from(tasks.values())`
- [ ] **键盘快捷键错误反馈**：`App.tsx` catch 中调用 `handleError`
- [ ] **Toolbar i18n 修复**：`Toolbar.tsx:241` 将 `t("rss.title")` 改为解压相关 key
- [ ] **DB Mutex 优化**：`storage/db.rs` 将 `tokio::sync::Mutex` 改为 `std::sync::Mutex`
- [ ] **resume_all_tasks 锁优化**：`commands/task.rs` 批量获取待恢复任务

## 低优先级 — 代码质量

- [ ] **Config 敏感信息加密**：`config.rs` api_token/proxy_password 至少 base64 编码
- [ ] **TaskList useMemo 依赖修复**：`TaskList.tsx:89` 将 searchQuery/statusFilter 加入依赖
- [ ] **PluginManager 类型去重**：从 `tauri-api.ts` 导入 PluginInfo
- [ ] **SettingSwitch ARIA**：添加 `role="switch"` 和 `aria-checked`
- [ ] **对话框焦点捕获**：所有对话框添加焦点捕获
- [ ] **SpeedChart rAF 优化**：无下载时停止 requestAnimationFrame 循环
- [ ] **右键菜单 ARIA**：添加 `role="menuitem"` 和键盘导航
- [ ] **App.tsx 状态简化**：9 个 useState 改为 reducer 或 openDialog 模式
- [ ] **useTaskEvents 清理优化**：将 notifiedTasks 清理移至 store action
- [ ] **allow_remote 死代码**：实现远程访问控制或移除字段

## 测试补全

- [ ] **Rust 核心模块测试**：`engine/task_manager.rs` 状态机、并发控制
- [ ] **Rust 存储层测试**：`storage/db.rs` CRUD、迁移、崩溃恢复
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
