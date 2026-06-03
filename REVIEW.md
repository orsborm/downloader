# 代码审查报告

> 日期：2026-06-04
> 基线：b3504b7 (2026-06-03 03:04)
> 审查范围：Rust 后端 (39 files, ~15,380 LOC) + React 前端 (49 files, ~15,600 LOC)
> 测试：617 前端 + 125 Rust = 742 测试

---

## 审查总结

| 状态 | 数量 |
|---|---|
| 安全加固已完成 | 7 项 ✅ |
| 本次修复 | 3 项 |
| 新增功能 | 1 项（版本管理） |
| 剩余 LOW | 2 项 |

---

## 安全加固（已确认）

| 项目 | 文件 | 状态 |
|------|------|------|
| CORS 白名单 | `api/mod.rs:166-184` | ✅ localhost-only，有测试 |
| API Token 认证 | `api/mod.rs:212-251` | ✅ 空 token 拒绝所有请求 |
| SpeedChart XSS | `SpeedChart.tsx:239` | ✅ DOM API + textContent |
| WASM 沙箱 | `plugin/loader.rs` | ✅ 64MB 内存 + fuel 限制 |
| 路径遍历防护 | `plugin/loader.rs:15-64` | ✅ canonicalize + 目录白名单 |
| DB 初始化 | `main.rs:121-124` | ✅ ? 传播，无 panic |
| Shell 注入防护 | `main.rs:73-96` | ✅ 白名单方式（本次修复） |

---

## 本次修复

### 1. Shell 命令注入加固
- **文件**: `main.rs:73-96`
- **原问题**: 黑名单仅 9 个字符，缺少 `\n\r><!#'"` 等
- **修复**: 改用白名单方式，仅允许 `a-zA-Z0-9 .-_/:=@[],+~`
- **安全性**: 阻断所有 shell 元字符和控制字符

### 2. DB JSON 链式 unwrap 修复
- **文件**: `storage/db.rs:801-806`
- **原问题**: `meta.get_mut("mirrorUrls").unwrap().as_array_mut().unwrap()` 可能 panic
- **修复**: 改为 `match ... { Some(arr) => arr, None => return Ok(()) }`

### 3. Plugin http_get 域名白名单
- **文件**: `plugin/loader.rs:274-287`
- **原问题**: `LoaderConfig::allowed_domains` 字段存在但未在 http_get 中检查
- **修复**: 添加域名白名单检查，仅在配置了 allowed_domains 时生效
- **安全性**: 支持精确匹配和子域名匹配（`*.example.com`）

---

## 新增功能：版本管理

- **文件**: `scripts/version.ps1`
- **功能**: 统一管理 package.json + tauri.conf.json + Cargo.toml 三处版本号
- **用法**:
  ```
  .\scripts\version.ps1              # 显示当前版本
  .\scripts\version.ps1 patch         # 1.0.0 → 1.0.1
  .\scripts\version.ps1 minor         # 1.0.0 → 1.1.0
  .\scripts\version.ps1 major         # 1.0.0 → 2.0.0
  .\scripts\version.ps1 set 1.2.3     # 直接设置
  ```
- **编码**: 使用 UTF8NoBOM 避免 PowerShell 5.1 BOM 问题

---

## 生产代码 unwrap() 审计

| 位置 | 风险 | 说明 |
|------|------|------|
| `db.rs:806` | 无风险 | 已修复为 match 安全访问 |
| `http.rs:81-82` | 无风险 | 前置 `len < 2` 守卫 |
| `api/mod.rs:170-175` | 无风险 | 编译时常量 URI parse |
| `api/mod.rs:271,290` | 无风险 | Body::from(&str) 不可失败 |
| `main.rs:646` | 合理 | Tauri 启动失败无法恢复 |

---

## 剩余 LOW 级别问题

| # | 文件 | 问题 |
|---|------|------|
| L1 | `plugin/loader.rs:121` | `expect("failed to build HTTP client")` 可改为 `unwrap_or_default()` |
| L2 | `api/mod.rs` WebSocket | auth 仅检查 header，未支持 query 参数 |

---

## 架构优势

- **模块化清晰**: engine/storage/api/plugin/rss/schedule/archive 各司其职
- **错误处理一致**: `anyhow::Result` 内部 + `Result<T, String>` IPC 边界
- **测试覆盖良好**: 742 测试（617 前端 + 125 Rust）
- **无 unsafe 代码**: 全代码库无 unsafe 块
- **无 XSS 风险**: 无 dangerouslySetInnerHTML/innerHTML/eval
- **安全多层防御**: CORS + Token + WASM 沙箱 + 路径遍历 + 命令白名单 + 域名白名单

---

*审查人：Claude Code Agent*
*最后更新：2026-06-04*
