# TODO — 下轮迭代计划

> 基于迭代 36 审查结果，按优先级排列

---

## 环境前置条件

- [ ] **安装 Rust 工具链**：`winget install Rustlang.Rustup` 或访问 https://rustup.rs/
- [ ] **安装 React Testing Library**：`npm i -D @testing-library/react @testing-library/jest-dom jsdom`

## 高优先级

- [ ] **Rust 后端编译验证**：`cd src-tauri && cargo build` 确认无编译错误（需 Rust 工具链）
- [ ] **Rust 后端测试**：`cd src-tauri && cargo test` 验证后端单元测试（需 Rust 工具链）
- [ ] **Windows 打包**：执行 `npm run tauri build` 生成便携 ZIP + NSIS 安装包（需 Rust 工具链）
- [ ] **集成测试**：Tauri IPC 通信端到端测试（需 Rust 编译环境）
- [x] **依赖安全审计**：`npm audit` 完成，5 个 moderate 漏洞（开发依赖），`cargo audit` 待 Rust 工具链

## 安全修复（迭代 33）

- [x] **WASM 沙箱路径遍历修复**：`plugin/loader.rs` 使用 `canonicalize()` 验证路径在允许目录内
- [x] **WASM 宿主函数死锁修复**：`plugin/loader.rs` 将 `block_on` 改为 `block_in_place`
- [x] **7z 解压路径遍历修复**：`archive/mod.rs` 对 7z 条目执行 `validate_safe_path`
- [x] **ed2k 无限递归修复**：`engine/ed2k/mod.rs` 改为循环处理 `OP_QUEUERANK`
- [x] **API 认证默认启用**：`storage/config.rs` + `api/mod.rs` 首次启动自动生成 token

## 功能改进（迭代 36）

- [x] **全局下载速度限制**：`task_manager.rs` 分离下载/上传限速器
- [x] **全局上传速度限制**：`task_manager.rs` 新增 `set_global_upload_speed_limit()`
- [x] **BT 引擎速度限制**：`bt/mod.rs` 集成 `librqbit::limits::LimitsConfig`
- [x] **设置界面速度配置**：`SettingsDialog.tsx` 添加全局下载/上传速度输入框
- [x] **i18n 翻译**：`locales/zh.ts` + `locales/en.ts` 添加速度限制相关翻译

## 功能改进（迭代 37）

- [x] **单任务下载限速**：`task.rs` + `db.rs` 支持独立任务速度限制
- [x] **单任务上传限速**：`task.rs` + `db.rs` 支持独立任务速度限制
- [x] **打开文件功能**：`task.rs` 新增 `open_file` 命令
- [x] **右键菜单增强**：`TaskList.tsx` 新增打开文件、速度限制子菜单
- [x] **速度限制显示**：`TaskList.tsx` 速度列显示当前限制
- [x] **复制链接提示**：`TaskList.tsx` 复制成功后显示提示

## 功能改进（迭代 38）

- [x] **多链接识别**：`AddTaskDialog.tsx` 支持输入多个链接（每行一个）
- [x] **批量添加任务**：`task.rs` 新增 `batch_add_tasks` 命令
- [x] **BT/Magnet 文件选择**：`FileSelectDialog.tsx` 支持全选/反选/单选
- [x] **获取种子文件列表**：`bt/mod.rs` 使用 `list_only` 模式获取文件列表
- [x] **带文件选择的 BT 任务**：`task.rs` 新增 `add_bt_task_with_files` 命令

## 代码审查与修复（迭代 39）

- [x] **批量任务空保存路径修复**：`task.rs` 在循环外获取默认路径
- [x] **i64 到 u64 安全转换**：`db.rs` 使用 `.unwrap_or(0).max(0)` 防止负值溢出
- [x] **BT 配置完整传递**：`bt/mod.rs` + `task_manager.rs` + `main.rs` 添加 lsd/encryption 配置
- [x] **useCallback 依赖修复**：`AddTaskDialog.tsx` 使用 useMemo 缓存 urls 计算
- [x] **硬编码中文字符串修复**：`TaskList.tsx` 替换为 t() 调用
- [x] **数组复制优化**：`taskStore.ts` 移入节流条件内，使用 splice 替代 shift
- [x] **clipboard 错误处理**：`TaskList.tsx` 使用 .then()/.catch() 正确处理

## P2P 功能计划

- [x] **P2P 调研完成**：分析现有能力和扩展方案
- [x] **BT 配置传递修复**：LSD/加密配置字段已添加
- [ ] **mDNS 局域网发现**：添加 mdns-sd 依赖，实现 discovery.rs
- [ ] **P2P 状态面板**：显示节点数、速度、分享文件
- [ ] **自定义 P2P 协议**：实现 protocol.rs/peer.rs/swarm.rs

## 镜像加速（迭代 40）

- [x] **多源下载引擎**：`http.rs` 新增 `download_with_mirrors()` 方法
- [x] **镜像URL管理API**：`task.rs` 新增 `add_mirror_url`/`remove_mirror_url`/`get_mirror_urls`
- [x] **镜像URL存储**：`db.rs` 使用 metadata JSON 字段存储
- [x] **镜像源管理UI**：`TaskDetail.tsx` 新增镜像源 Tab
- [x] **任务管理器集成**：`task_manager.rs` 支持多源下载

## 缺陷全面修复（迭代 40）

- [x] **路径遍历防护**：`util/mod.rs` 新增 `validate_filename()` 函数
- [x] **HttpEngine panic 修复**：使用 `unwrap_or_else` 降级处理
- [x] **目录创建错误日志**：`http.rs` 记录警告而非静默忽略
- [x] **open_file 命令注入修复**：Windows 改用 `explorer` 打开
- [x] **重试逻辑异步IO**：改用 `tokio::fs::metadata`
- [x] **文件 flush 修复**：下载完成后调用 `sync_all()`
- [x] **SettingsDialog 空状态**：添加错误状态 UI
- [x] **clipboard 错误处理**：已在迭代 39 修复

## 中优先级

- [x] **React 组件逻辑测试**：新增 47 个测试覆盖 TaskDetail/TaskList/ContextMenu（总计 617 测试）
- [ ] **React 组件渲染测试**：使用 React Testing Library 测试渲染和交互
- [ ] **CSS 类名统一**：使用 clsx 替代模板字符串拼接 Tailwind 类
- [ ] **日期格式化集成**：将硬编码 "zh-CN" 接入 i18n 设置系统
- [ ] **Zustand store 拆分**：分离 selectionStore + speedStore，降低 taskStore 复杂度
- [ ] **macOS/Linux 打包配置**：跨平台构建脚本和 CI/CD

## 低优先级

- [ ] **性能基准测试**：启动时间、内存占用、1000+ 任务压力测试
- [ ] **E2E 测试**：Playwright 覆盖核心下载流程
- [ ] **浏览器扩展测试**：Puppeteer 测试 content script 视频嗅探
- [ ] **插件开发 SDK 文档**：downloader-plugin-sdk 使用指南

## 已知限制

- ed2k KAD 实际网络连接待真实环境测试
- 浏览器扩展与 Rust 后端联调待完成
- RSS/Archive 模块需集成到 Tauri 命令层
- BT 引擎速度限制需重新初始化会话才能生效

---

*最后更新：2026-05-31*
