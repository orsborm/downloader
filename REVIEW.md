# 代码审查报告 — v1.1.3

> 日期：2026-06-05
> 版本：v1.1.3
> 审查范围：Rust 39 files + React 49 files（全量深度扫描）
> 测试：617 前端 + 125 Rust = 742 测试

---

## 审查总结

| 状态 | 数量 |
|---|---|
| 安全加固 | 14 项 ✅ |
| 稳定性修复 | 12 项 ✅ |
| UI/可访问性修复 | 18 项 ✅ |
| KAD 协议修复 | 4 项 ✅ |
| Bug 修复 | 3 项 ✅ |
| 剩余 LOW | 20 项 |

---

## 本轮修复

### KAD 协议修复（4 项）
| 项目 | 文件 | 说明 |
|------|------|------|
| 引导端口 | `ed2k/mod.rs` | UDP 4672 (TCP+11) 替代 TCP 4661 |
| 距离比较 | `ed2k/kad.rs` | 完整 128 位字节比较替代 leading_zeros |
| 源解析偏移 | `ed2k/kad.rs` | UDP+TCP 端口各 2 字节（共 4），非 4+4 |
| 协议验证 | `ed2k/kad.rs` | 验证 0xE4 标识和操作码 |

### Bug 修复（3 项）
| 项目 | 文件 | 说明 |
|------|------|------|
| BT 速度截断 | `bt/mod.rs` | `as u32` → `.min(u32::MAX) as u32` 饱和转换 |
| 错误显示 | 3 个对话框 | `String(e)` → `extractErrorMessage(e)` |
| KAD 引导状态 | `ed2k/kad.rs` | `bootstrap_attempted` 独立于节点数 |

### 可访问性修复（13 项）
| 项目 | 文件 | 说明 |
|------|------|------|
| 对话框 ARIA | 10 个文件 | `role="dialog"` + `aria-modal="true"` |
| 关闭按钮 | 10 个文件 | `aria-label="Close"` |

---

## 剩余 LOW 级别问题（20 项）

| # | 类别 | 文件 | 问题 |
|---|------|------|------|
| 1 | 性能 | `App.tsx` | 订阅整个 tasks Map，每秒 re-render |
| 2 | 性能 | `Toolbar.tsx` | taskArray 每次速度更新都重建 |
| 3 | 性能 | `TaskDetail.tsx` | LogsTab logs 数组未 memoize |
| 4 | 性能 | `ScheduleDialog.tsx` | CRON_PRESETS/WEEKDAYS 每次渲染重建 |
| 5 | 清理 | `TaskDetail.tsx` | LogsTab async fetch 无 mounted guard |
| 6 | i18n | `App.tsx` | ErrorBoundary 硬编码双语文本 |
| 7 | i18n | `App.tsx` | 拖拽提示中文 fallback |
| 8 | i18n | `TaskDetail.tsx` | URL 验证错误中文 fallback |
| 9 | 类型 | `PluginManager.tsx` | 使用 raw invoke 替代 typed wrapper |
| 10 | 类型 | `Toolbar.tsx` | `as StatusFilter` 类型断言 |
| 11 | CSS | `App.tsx` | 拖拽 z-40 低于对话框 z-50 |
| 12 | CSS | `TaskList.tsx` | 右键菜单 z-50 与对话框相同 |
| 13 | ARIA | `TaskList.tsx` | 排序头部缺少 aria-sort |
| 14 | ARIA | `TaskDetail.tsx` | Tab 导航缺少 ARIA tab 角色 |
| 15 | ARIA | `StatusBar.tsx` | 进度条缺少 role="progressbar" |
| 16 | ARIA | `Toast.tsx` | 单条 toast 缺少 role="alert" |
| 17 | 死代码 | `archive/password.rs` | PasswordManager 未使用 |
| 18 | 死代码 | `hls/mod.rs` | download_segment 未调用 |
| 19 | 死代码 | `rss/mod.rs` | matches_rules 实例方法未使用 |
| 20 | 性能 | `plugin/loader.rs` | WASM 模块每次重新编译 |

---

*审查人：Claude Code Agent*
*最后更新：2026-06-05*
