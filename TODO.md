# TODO

> 基于代码审查（REVIEW.md），按优先级排列

---

## 安全加固

- [x] **Shell 命令注入加固**：`main.rs` 改用白名单方式过滤命令字符
- [x] **CORS 白名单**：`api/mod.rs` 仅允许 localhost 来源
- [x] **API Token 认证**：`api/mod.rs` 空 token 拒绝所有请求
- [x] **SpeedChart XSS**：`SpeedChart.tsx` 使用 DOM API 替代 innerHTML
- [x] **WASM 沙箱**：`plugin/loader.rs` 64MB 内存限制 + fuel 限制
- [x] **路径遍历防护**：`plugin/loader.rs` canonicalize + 目录白名单
- [x] **DB 初始化安全**：`main.rs` ? 传播，无 panic
- [x] **DB JSON 安全访问**：`storage/db.rs` match 替代 chain unwrap
- [x] **Plugin 域名白名单**：`plugin/loader.rs` http_get 检查 allowed_domains

## 版本管理

- [x] **版本管理脚本**：`scripts/version.ps1` 统一管理三处版本号

## 测试补全

- [x] **Rust API 模块测试**：`api/mod.rs` CORS 白名单、JSON-RPC 格式、事件序列化
- [x] **Rust 插件加载器测试**：`plugin/loader.rs` 路径遍历、WASM 验证、域名白名单
- [x] **Rust 存储层测试**：`storage/db.rs` 枚举 roundtrip、DB 创建、镜像 URL
- [ ] **Rust 核心模块测试**：`engine/task_manager.rs` 状态机、并发控制
- [ ] **Rust HTTP 引擎测试**：`engine/http.rs` 下载、断点续传、代理
- [ ] **前端组件渲染测试**：引入 React Testing Library
- [ ] **Windows 打包验证**：`npm run tauri build` 生成便携 ZIP + NSIS
- [ ] **依赖安全审计**：`npm audit` + `cargo audit`

## 已知限制

- BT 引擎核心下载待集成 librqbit API 完整流程
- ed2k KAD 实际网络连接待真实环境测试
- 浏览器扩展与 Rust 后端联调待完成
- RSS/Archive 模块需集成到 Tauri 命令层

---

*最后更新：2026-06-04*
