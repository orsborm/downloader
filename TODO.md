# TODO — 迭代 37 完成

> 所有 MEDIUM + LOW 级别问题已修复

---

## 已完成

### 安全加固（10 项）
- [x] Shell 命令注入白名单
- [x] CORS 白名单
- [x] API Token 认证
- [x] SpeedChart XSS 防护
- [x] WASM 沙箱
- [x] 路径遍历防护
- [x] DB 初始化 panic
- [x] 敏感信息混淆
- [x] HTTP 客户端安全降级
- [x] DB JSON 安全访问

### 稳定性修复（8 项）
- [x] 前端 Error Boundary（4 个区域级）
- [x] App re-render 优化
- [x] TaskDetail 实时订阅 store
- [x] 任务参数持久化日志
- [x] TaskList useMemo 依赖优化（taskVersion 计数器）
- [x] resume_all_tasks 批量锁优化
- [x] Dead code 清理（Dialog.tsx, PasswordManager, HostApi, STORAGE_KEY）
- [x] 版本号管理机制

### 可访问性（12 项）
- [x] 所有对话框 role=dialog + aria-modal=true（10 个）
- [x] 所有对话框关闭按钮 aria-label（10 个）
- [x] 进度条 role=progressbar + aria-valuenow/min/max
- [x] 右键菜单键盘导航（ArrowUp/Down/Enter/Space/Escape）
- [x] Canvas 屏幕阅读器 aria-label
- [x] SectionErrorBoundary 双语错误文本

### 代码整洁（2 项）
- [x] main.tsx 清理多余 React import
- [x] lib/i18n.ts 清理未使用导出

## 测试补全

- [x] Rust API 模块测试（5 个）
- [x] Rust 插件加载器测试（7 个）
- [x] Rust 存储层基础测试（6 个）
- [x] 前端 XSS 防护测试（4 个）
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
- DB Mutex 类型优化（tokio::sync→std::sync）待大规模重构

---

*最后更新：2026-06-03*
