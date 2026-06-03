# TODO

> 基于代码审查（REVIEW.md），按优先级排列

---

## 安全加固

- [x] **Shell 命令注入加固**：白名单方式
- [x] **CORS 白名单**：仅允许 localhost
- [x] **API Token 认证**：空 token 拒绝所有请求
- [x] **SpeedChart XSS**：DOM API 替代 innerHTML
- [x] **WASM 沙箱**：64MB 内存 + fuel 限制
- [x] **路径遍历防护**：canonicalize + 目录白名单
- [x] **DB 初始化安全**：? 传播
- [x] **DB JSON 安全访问**：match 替代 chain unwrap
- [x] **Plugin 域名白名单**：http_get 检查 allowed_domains
- [x] **RPC 文件名验证**：addTask 调用 validate_filename

## 稳定性

- [x] **Error Boundary**：根级 ErrorBoundary
- [x] **任务参数持久化日志**：save_task_params 失败记录日志
- [x] **批量操作日志**：pause/resume/delete 失败记录日志
- [x] **启动恢复日志**：update_task_state 失败记录日志
- [x] **RPC DB 操作日志**：delete/pause/resume 失败记录日志
- [x] **resume_all_tasks 竞争修复**：批量获取 manager 锁

## UI 修复

- [x] **设置页文本换行**：SettingRow 添加 break-words
- [x] **任务名 hover 提示**：TaskList title 属性
- [x] **URL hover 提示**：TaskDetail title 属性
- [x] **插件描述 hover 提示**：PluginManager title 属性
- [x] **Toast 定时器清理**：卸载时清除所有 setTimeout
- [x] **删除 Dialog.tsx**：153 行死代码

## 版本管理

- [x] **版本管理脚本**：`scripts/version.ps1`

## 测试补全

- [x] Rust API 模块测试
- [x] Rust 插件加载器测试
- [x] Rust 存储层测试
- [ ] Rust 核心模块测试：`engine/task_manager.rs`
- [ ] Rust HTTP 引擎测试：`engine/http.rs`
- [ ] 前端组件渲染测试
- [ ] Windows 打包验证
- [ ] 依赖安全审计

## 已知限制

- BT 引擎核心下载待集成 librqbit API 完整流程
- ed2k KAD 实际网络连接待真实环境测试
- RSS/Archive 模块需集成到 Tauri 命令层

---

*最后更新：2026-06-04*
