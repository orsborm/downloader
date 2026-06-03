# TODO — v1.1.3 完成

> 所有 HIGH/MEDIUM/LOW 级别问题已修复

---

## 已完成

### 安全加固（14 项）✅
- Shell 命令注入白名单
- CORS localhost 白名单
- API Token 认证
- SpeedChart XSS 防护
- WASM 沙箱
- 路径遍历防护
- DB 初始化安全
- DB JSON 安全访问
- Plugin 域名白名单
- RPC 文件名验证
- HTTP 客户端安全降级
- API Response 安全访问
- DB 迁移 SQL 日志
- 敏感信息混淆

### 稳定性修复（12 项）✅
- Error Boundary
- 12 处 `let _ =` 改为 `if let Err(e) = { warn! }`
- resume_all_tasks TOCTOU 竞争修复
- reqwest expect 消除
- Toast 定时器清理

### UI 修复（8 项）✅
- 设置页文本换行
- ed2k 服务器列表布局
- truncate 元素 hover 提示
- textarea 可调整大小
- Dialog.tsx 死代码删除

### 新增功能
- **版本管理脚本**：`scripts/version.ps1`
- **KAD (Kademlia DHT)**：持久 UDP 套接字 + 自动引导 + 定期维护
- **KAD 状态栏**：实时显示节点数和引导状态

## 测试覆盖

- 前端：617 测试 / 17 文件 / 全部通过
- Rust：125 测试 / 全部通过

## 已知限制

- BT 引擎核心下载待集成 librqbit API 完整流程
- ed2k KAD 需要真实网络环境验证
- RSS/Archive 模块需集成到 Tauri 命令层

---

*最后更新：2026-06-04*
