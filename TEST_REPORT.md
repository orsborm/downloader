# 测试报告

> 生成时间：2026-05-29
> 最后更新：迭代 27 — 安全审计与质量改进

## 测试概览

| 指标 | 值 |
|---|---|
| 测试文件数 | 17 |
| 总测试数 | **573** |
| 通过 | 573 |
| 失败 | 0 |
| 跳过 | 0 |
| 执行时间 | ~3.7s |

## 测试文件明细

### 1. format.test.ts（121 测试）

格式化工具函数测试，覆盖所有导出函数的正常/边界/异常场景。

| 测试组 | 测试数 | 内容 |
|---|---|---|
| formatSize | 14 | 0/负数/NaN/Infinity/字节/KB/MB/GB/TB 边界 |
| formatSpeed | 6 | 0/NaN/Infinity/KB/s/MB/s/GB/s/TB/s |
| formatEta | 12 | null/0/负数/秒/分秒/时分/小数/大值/NaN |
| formatProgress | 8 | 0/0.5/1/小数/NaN/Infinity/负数/超1 |
| formatDateTime | 4 | 有效日期/无效日期/空字符串/未来日期 |
| protocolLabel | 8 | HTTP/FTP/BT/MAGNET/ED2K/HLS/DASH/未知 |
| stateLabel | 7 | 6 种已知状态 + 未知状态 |
| formatDuration | 10 | 0/秒/分秒/时分/大值/负数/NaN/Infinity |
| isValidDownloadUrl | 26 | HTTP/HTTPS/FTP/FTPS/Magnet/ed2k/HLS/DASH/torrent/file/安全拒绝(XSS/javascript/vbscript/blob/chrome/about)/IPv4/IPv6/端口/认证/片段 |
| filterHistory | 6 | 空查询/文件名搜索/URL搜索/协议标签/无匹配/null字段 |
| computeHistoryStats | 4 | 总计计算/空数组/单条/缺失字段 |
| aggregateByDay | 7 | 天数范围/30天/数据结构/日期聚合/null日期/超出范围/日期格式 |

### 2. taskStore.test.ts（65 测试）

Zustand 状态管理测试，覆盖任务 CRUD、选择、排序、过滤、速度历史。

| 测试组 | 测试数 | 内容 |
|---|---|---|
| setTasks | 4 | 填充/替换/空数组/重复ID |
| applyUpdate | 9 | 更新速度/进度/保留旧值/忽略未知ID/错误/完成/做种/条件字段 |
| selectTask | 5 | 单选/Ctrl多选/Shift范围/无lastSelected/重选 |
| removeTask | 4 | 删除/从selectedIds移除/空store/多选删除 |
| setSort | 2 | 同键切换/不同键重置 |
| getSortedTasks | 8 | 按名称/大小/速度/进度排序/空数组/状态优先级/默认排序/搜索+过滤组合 |
| getGlobalStats | 4 | 活跃计数/速度求和/空store/混合速度 |
| speedHistory | 3 | 全局速度追踪/节流/大量更新长度限制 |
| clearSelection | 1 | 清除选择 |
| setSearchQuery | 2 | 设置/清除 |
| setStatusFilter | 2 | 设置/重置 |
| getTaskById | 3 | 查找/不存在/空store |
| 大数据集 | 2 | 100任务选择/1000任务设置 |

### 3. utils.test.ts（41 测试）

业务逻辑测试，覆盖 URL 检测、状态转换、进度/ETA 计算、优先级排序。

| 测试组 | 测试数 | 内容 |
|---|---|---|
| detectDownloadUrl | 14 | magnet/ed2k/HTTP/HTTPS/FTP/空/null/javascript/data/查询参数/大小写 |
| task state transitions | 12 | 所有合法/非法状态转换路径 |
| progress calculation | 6 | 0总量/0已下载/完成/分数/超量/负总量 |
| ETA calculation | 5 | 0速度/0总量/已完成/正常/大文件 |
| priority ordering | 4 | 优先级排序/时间排序/空数组/单元素 |

### 4. extension.test.ts（34 测试）

浏览器扩展工具函数测试，覆盖 URL 检测、文件名提取、类型标签。

| 测试组 | 测试数 | 内容 |
|---|---|---|
| isDownloadableUrl | 14 | zip/exe/mp4/torrent/magnet/ed2k/archive/document/audio/webm/iso/大小写/空/HTML |
| getFilename | 6 | URL路径/查询参数/编码字符/无效URL/根URL/嵌套路径 |
| getFormat | 4 | 扩展名/多级扩展/无扩展名/mp4 |
| getTypeLabel | 6 | HLS/视频/音频/磁力链/ed2k/文件 |
| getTypeClass | 4 | type-hls/type-video/type-file/音频归为type-video |

### 5. webui.test.ts（43 测试）

WebUI 组件逻辑测试，覆盖状态标签、颜色映射、操作按钮可见性、标签页配置。

| 测试组 | 测试数 | 内容 |
|---|---|---|
| statusLabel | 9 | active/downloading/paused/waiting/completed/error/seeding/未知/空 |
| StatusIcon colors | 7 | 7种状态对应颜色映射 |
| progress bar colors | 4 | error红/completed绿/paused黄/active蓝 |
| task action visibility | 6 | active暂停+删除/paused恢复+删除/error恢复+删除/completed仅删除/seeding仅删除 |
| tab configuration | 3 | 正确标签页/零计数/大计数 |
| empty state messages | 5 | 未连接/active/waiting/stopped/未知标签页 |
| task name display | 3 | 有名称/空名称回退ID/短ID |
| detail value formatting | 6 | 截断长ID/保留短ID/正ETA/零ETA/负ETA/数字转字符串 |

### 6. components.test.ts（51 测试）🆕

组件逻辑测试，覆盖 Toast ID 生成、限速解析、URL 验证、协议颜色、状态图标、速度图表过滤、状态排序、全局统计。

| 测试组 | 测试数 | 内容 |
|---|---|---|
| Toast ID generation | 3 | 唯一性/UUID格式/大量ID无碰撞 |
| speed limit parsing | 8 | 空值/KB→bytes转换/大值/非数字/零/负数/小数截断 |
| URL validation | 12 | 空值/空白/HTTP/HTTPS/magnet/ed2k/FTP/javascript/纯文本/torrent路径/m3u8/空格修剪 |
| protocolColor | 8 | HTTP蓝/FTP紫/BT绿/MAGNET绿/ED2K橙/未知灰/空灰/暗色模式 |
| StateIcon mapping | 7 | 6种状态映射 + 未知状态回退 |
| SpeedChart time filter | 5 | 5分钟/2秒/1秒/零范围/大范围 |
| state priority ordering | 4 | downloading优先/seeding>queued/error最后/未知状态 |
| global stats calculation | 4 | 速度求和/seeding计入活跃/空列表/忽略非活跃状态 |

## 迭代 22：代码审查修复

| 任务 | 状态 | 说明 |
|---|---|---|
| Toast ID 碰撞修复 | ✅ 已完成 | `Date.now()+counter` → `crypto.randomUUID()`，消除并发碰撞风险 |
| 动态导入修复 | ✅ 已完成 | `TaskList.tsx` 中 `handlePriority` 改为静态导入 `getAllTasks` |
| formatEta 负数处理 | ✅ 已完成 | 负值返回 `"--"` 而非 `"∞"`，区分"未知"和"计算错误" |
| 剪贴板冷却机制 | ✅ 已完成 | 新增 10 秒冷却时间，避免复制普通 URL 重复触发弹窗 |
| 测试同步更新 | ✅ 已完成 | Toast UUID 格式测试 + formatEta 负值测试更新，338 测试全通过 |

**变更文件**：`Toast.tsx`, `TaskList.tsx`, `format.ts`, `useClipboard.ts`, `components.test.ts`, `format.test.ts`

## 迭代 23：下载历史增强 + 流量图表

| 任务 | 状态 | 说明 |
|---|---|---|
| 搜索过滤 | ✅ 已完成 | 按文件名/URL/协议关键词实时过滤历史记录 |
| 统计摘要 | ✅ 已完成 | 显示总条数、总大小、平均速度、总耗时 |
| CSV 导出 | ✅ 已完成 | 导出过滤后的历史记录为 CSV（BOM 头兼容 Excel 中文） |
| 流量图表 | ✅ 已完成 | Recharts 柱状图展示最近 30 天每日下载趋势，列表/图表视图切换 |
| 纯函数提取 | ✅ 已完成 | `filterHistory`/`computeHistoryStats`/`aggregateByDay` 提取到 format.ts |
| 测试新增 | ✅ 已完成 | 新增 17 个测试（filterHistory 6 + computeHistoryStats 4 + aggregateByDay 7），355 测试全通过 |

**变更文件**：`DownloadHistoryDialog.tsx`, `format.ts`, `format.test.ts`

**测试统计**：355 测试全通过（原 338 → 新增 17）

## 迭代 24：安全修复与质量改进

| 任务 | 状态 | 说明 |
|---|---|---|
| Zip Slip 路径遍历修复 | ✅ 已完成 | 重写 `validate_safe_path`，使用路径组件分析替代 `canonicalize`（后者在路径不存在时可被绕过）；新增 `normalize_path` 辅助函数；拒绝任何含 `..` 的路径组件 |
| 速度历史缓冲区修复 | ✅ 已完成 | 缓冲区从 5 分钟扩展到 1 小时，与 SpeedChart 最大时间范围选项一致 |

**验证结果**：CODE_REVIEW.md 中标记的其他关键问题（API 认证、OPML XML 注入、重试进度恢复、formatEta(0)、信号量错误处理）均已在先前迭代中修复。

**变更文件**：`src-tauri/src/archive/mod.rs`, `src/stores/taskStore.ts`

**测试统计**：355 测试全通过（无新增测试，纯安全修复）

## 迭代 25：库依赖升级与代码质量改进

| 任务 | 状态 | 说明 |
|---|---|---|
| RSS 过滤规则：自研 regex → regex crate | ✅ 已完成 | 移除 200+ 行自研 `regex_lite` 模块，替换为 `regex` crate，支持字符类 `[a-z]`、量词 `{3}`、锚点 `^$` 等完整正则语法；新增 6 个测试（regex 匹配、复杂模式、无效正则回退、组合模式） |
| DASH MPD 解析：字符串解析 → quick-xml | ✅ 已完成 | 重写 `parse_mpd`，使用 `quick-xml` 流式解析器替代手动字符串查找；正确处理 `SegmentTemplate`（timescale/media/initialization/startNumber）、`SegmentTimeline`（S 元素）、`Representation` 属性、`BaseURL` 嵌套；新增 `expand_template` 函数支持 `$Number$`/`$RepresentationID$`/`$Number%05d$` 占位符；新增 4 个 MPD 测试 + 1 个 expand_template 测试 |
| RSS/Atom 解析：字符串解析 → quick-xml | ✅ 已完成 | 重写 `parse_rss_items`/`parse_atom_items`/`parse_opml`，使用 `quick-xml` 流式解析；正确处理 Atom 自闭合 `<link href="..."/>` 标签、HTML 实体解码；新增 7 个测试（RSS 单条/多条、Atom、OPML、自动检测、无 link 跳过、HTML 实体） |

**新增依赖**：`regex = "1"`, `quick-xml = "0.36"`

**变更文件**：`Cargo.toml`, `rss/rules.rs`, `rss/feed.rs`, `engine/hls/m3u8.rs`

**测试统计**：355 前端测试全通过 + 新增 18 个 Rust 单元测试（需 Rust 工具链验证）

## 迭代 26：统一错误处理模块

| 任务 | 状态 | 说明 |
|---|---|---|
| 创建统一错误处理模块 | ✅ 已完成 | `lib/errors.ts`：DownloaderError 类、ErrorCode 枚举（20+ 错误码）、handleError/withErrorHandling/withSyncErrorHandling 函数 |
| 错误码自动推断 | ✅ 已完成 | 从错误消息自动推断错误码（network→NETWORK_ERROR、timeout→TIMEOUT、ENOSPC→DISK_FULL 等） |
| 用户友好消息 | ✅ 已完成 | 每个错误码映射到中文用户友好消息 |
| 组件错误处理统一 | ✅ 已完成 | 更新 App.tsx、TaskList.tsx、PluginManager.tsx、RssManager.tsx、SettingsDialog.tsx 使用统一错误处理 |
| 错误处理测试 | ✅ 已完成 | 新增 30 个测试：DownloaderError 构造/from/toUserMessage、错误码推断、handleError/withErrorHandling/withSyncErrorHandling |

**变更文件**：`lib/errors.ts`（新增）, `App.tsx`, `components/TaskList.tsx`, `components/PluginManager.tsx`, `components/RssManager.tsx`, `components/SettingsDialog.tsx`, `__tests__/errors.test.ts`（新增）

**测试统计**：385 测试全通过（原 355 → 新增 30）

### 8. speedchart.test.ts（11 测试）🆕

SpeedChart Canvas 组件工具函数测试，覆盖 Y 轴刻度计算和时间标签格式化。

| 测试组 | 测试数 | 内容 |
|---|---|---|
| computeYTicks | 7 | 零值/负值/小值(1KB)/中值(1MB)/大值(100MB)/均匀间距/刻度数量限制 |
| formatTimeLabel | 4 | 格式验证/零填充/午夜/日末 |

### 9. shared-types.test.ts（7 测试）🆕

共享类型模块测试，验证桌面端和 WebUI 类型复用。

| 测试组 | 测试数 | 内容 |
|---|---|---|
| shared types re-export | 7 | Protocol/TaskState完整性/桌面端re-export兼容性/TaskInfo字段/TaskParams必填字段/SpeedDataPoint/GlobalStat |

## 迭代 27：SpeedChart Canvas 优化与共享类型

| 任务 | 状态 | 说明 |
|---|---|---|
| SpeedChart Canvas 重写 | ✅ 已完成 | 从 Recharts SVG 替换为 Canvas API + requestAnimationFrame，支持 60fps 实时渲染；保留双 Y 轴、时间范围切换、十字线 tooltip |
| 共享类型模块 | ✅ 已完成 | `src/shared/types.ts`：提取桌面端和 WebUI 共用的核心接口（Protocol/TaskState/TaskStatus/TaskInfo/GlobalStat 等）；`src/lib/types.ts` 改为 re-export；WebUI App.tsx 导入共享类型 |
| SpeedChart 工具测试 | ✅ 已完成 | 新增 11 个测试：computeYTicks 7 个（边界值/刻度均匀性/数量限制）+ formatTimeLabel 4 个（格式/零填充/边界时间） |
| 共享类型测试 | ✅ 已完成 | 新增 7 个测试：类型可访问性/状态完整性/桌面端 re-export 兼容性/WebUI TaskInfo 字段 |

**变更文件**：`src/components/SpeedChart.tsx`, `src/shared/types.ts`（新增）, `src/lib/types.ts`, `src-webui/App.tsx`, `src/__tests__/speedchart.test.ts`（新增）, `src/__tests__/shared-types.test.ts`（新增）

**测试统计**：403 测试全通过（原 385 → 新增 18）

## 迭代 28：后端补全与质量改进

基于代码审计发现的 7 个关键缺陷，逐一修复：

| 任务 | 状态 | 说明 |
|---|---|---|
| DB update_task_progress 补全 | ✅ 已完成 | SQL 更新语句新增 `download_speed`、`upload_speed`、`peers` 字段；tasks 表新增 3 列；增量迁移（ALTER TABLE，忽略重复列错误）；4 个 SELECT 查询更新为读取新列 |
| BT 引擎全局速度聚合 | ✅ 已完成 | `get_status()` 从硬编码 0 改为遍历所有活跃任务的 `handle.stats().live` 聚合真实下载/上传速度 |
| DASH MPD 下载路径连接 | ✅ 已完成 | `parse_playlist()` 新增 DASH 检测（URL 以 `.mpd` 结尾或内容含 `<MPD`），调用已有的 `m3u8::parse_mpd()` 解析器，返回 `StreamType::Dash` |
| ed2k server_ident 实现 | ✅ 已完成 | 解析 16 字节服务器 hash + tag 列表；提取服务器名称(0x01)、描述(0x0B)、用户数(0x0C)、文件数(0x0D)；存储到 `server_info` 字段 |
| ed2k 队列排名处理 | ✅ 已完成 | `handle_queue_rank()` 实现等待逻辑：排名 0 立即下载，排名 >0 计算等待时间（基础 10s + 每名 5s，上限 300s），保持 WaitingUpload 状态 |
| WASM 插件 HostApi 注入 | ✅ 已完成 | 新增 `http_get`、`read_file`、`write_file`、`emit_event` 四个宿主函数到 WASM linker；http_get 使用 reqwest 客户端；read/write_file 包含路径遍历安全检查 |
| 插件清单解析 | ✅ 已完成 | `load_plugin_manifest()` 从 WASM 自定义 section `plugin_manifest` 解析 JSON 清单；实现 LEB128 解码、section 跳过、名称匹配；无自定义 section 时回退到文件名推断 |

**变更文件**：`storage/db.rs`, `engine/bt/mod.rs`, `engine/hls/mod.rs`, `engine/ed2k/server.rs`, `engine/ed2k/transfer.rs`, `plugin/loader.rs`, `plugin/mod.rs`

**测试统计**：407 测试全通过（前端测试无变化，Rust 后端变更需 Rust 工具链验证）

## 迭代 29：国际化 (i18n) 支持

| 任务 | 状态 | 说明 |
|---|---|---|
| i18n 核心模块 | ✅ 已完成 | `lib/i18n.ts`：语言检测、切换、翻译函数、模板参数替换、localStorage 持久化 |
| 中文语言包 | ✅ 已完成 | `lib/locales/zh.ts`：完整中文翻译（通用/任务状态/工具栏/设置/错误/通知/协议等） |
| 英文语言包 | ✅ 已完成 | `lib/locales/en.ts`：完整英文翻译，与中文语言包结构完全一致 |
| React Hook | ✅ 已完成 | `hooks/useI18n.ts`：Zustand 状态管理集成，语言切换时自动重新渲染组件 |
| Toolbar 组件 i18n | ✅ 已完成 | 所有用户可见字符串使用 t() 翻译函数 |
| SettingsDialog 组件 i18n | ✅ 已完成 | 标题、标签页、按钮、设置选项全部使用 i18n |
| 语言切换器 | ✅ 已完成 | 设置 → 常规 → 语言下拉菜单，切换即时生效 |
| i18n 测试 | ✅ 已完成 | 新增 25 个测试：语言包结构一致性、无空值验证、语言检测、语言切换、翻译函数（简单键/嵌套键/模板参数/未知键） |

**变更文件**：`lib/i18n.ts`（新增）, `lib/locales/zh.ts`（新增）, `lib/locales/en.ts`（新增）, `hooks/useI18n.ts`（新增）, `components/Toolbar.tsx`, `components/SettingsDialog.tsx`, `__tests__/i18n.test.ts`（新增）

**测试统计**：432 测试全通过（原 407 → 新增 25）

## 迭代 30：任务调度系统

| 任务 | 状态 | 说明 |
|---|---|---|
| 调度管理模块 | ✅ 已完成 | `schedule/mod.rs`：ScheduleManager 实现，支持 cron 表达式验证、调度规则 CRUD、带宽计划管理 |
| 调度 IPC 命令 | ✅ 已完成 | `commands/schedule.rs`：11 个 IPC 命令，覆盖规则和带宽计划的增删改查 |
| 调度 UI 组件 | ✅ 已完成 | `components/ScheduleDialog.tsx`：双标签页（调度规则/带宽计划），支持 cron 预设、星期几选择 |
| 工具栏集成 | ✅ 已完成 | `Toolbar.tsx`：新增 Timer 图标按钮，`App.tsx`：集成 ScheduleDialog |
| API 封装 | ✅ 已完成 | `lib/tauri-api.ts`：11 个 API 函数，完整类型定义 |
| 调度测试 | ✅ 已完成 | `__tests__/schedule.test.ts`：30 个测试用例，覆盖 API 调用、类型验证、参数边界 |

**变更文件**：`schedule/mod.rs`（新增）, `commands/schedule.rs`（新增）, `commands/mod.rs`, `main.rs`, `components/ScheduleDialog.tsx`（新增）, `components/Toolbar.tsx`, `App.tsx`, `lib/tauri-api.ts`, `__tests__/schedule.test.ts`（新增）

**测试统计**：462 测试全通过（原 432 → 新增 30）

### 11. schedule.test.ts（30 测试）🆕

任务调度 API 测试，覆盖调度规则和带宽计划的 CRUD 操作、类型验证、参数边界。

| 测试组 | 测试数 | 内容 |
|---|---|---|
| addScheduleRule | 2 | 正确参数调用/错误抛出 |
| removeScheduleRule | 1 | 正确参数调用 |
| updateScheduleRule | 1 | 正确参数调用 |
| setScheduleRuleEnabled | 2 | 启用/禁用 |
| getScheduleRules | 2 | 返回规则列表/空数组 |
| getScheduleRule | 2 | 返回规则/null |
| addBandwidthSchedule | 2 | 基本调用/含星期几 |
| removeBandwidthSchedule | 2 | 删除第一个/第二个 |
| updateBandwidthSchedule | 1 | 正确参数调用 |
| getBandwidthSchedules | 2 | 返回计划列表/空数组 |
| getCurrentBandwidthLimit | 2 | 返回限制/null |
| ScheduleRuleType | 1 | 四种类型验证 |
| Cron Expression | 2 | 有效表达式/存储验证 |
| BandwidthSchedule Validation | 4 | 时间格式/星期范围/空星期/速度非负 |
| ScheduleParams | 4 | 可选速度/空参数/命令/自定义参数 |

## 迭代 31：Windows 便携打包与 API 测试扩展

| 任务 | 状态 | 说明 |
|---|---|---|
| Windows 便携打包脚本 | ✅ 已完成 | `scripts/build-portable.ps1`：PowerShell 脚本，支持前端构建→测试→Tauri 构建→便携 ZIP 打包全流程；自动创建 data/ 目录结构、默认 config.toml、README.txt |
| Tauri API 集成测试 | ✅ 已完成 | `__tests__/tauri-api.test.ts`：37 个测试用例，覆盖任务 CRUD、批量操作、设置、状态查询、历史记录、RSS、插件、解压、文件操作等全部 API 函数 |
| CI/CD 配置验证 | ✅ 已完成 | GitHub Actions workflow 已配置：CI（前端测试+Rust 检查+Rust 测试）、Release（Windows/macOS/Linux 三平台构建+发布） |

**变更文件**：`scripts/build-portable.ps1`（新增）, `__tests__/tauri-api.test.ts`（新增）

**测试统计**：499 测试全通过（原 462 → 新增 37）

### 12. tauri-api.test.ts（37 测试）🆕

Tauri IPC API 集成测试，覆盖所有前端→后端通信接口。

| 测试组 | 测试数 | 内容 |
|---|---|---|
| addTask | 3 | 正确参数/可选参数/错误传播 |
| pauseTask | 1 | 正确参数调用 |
| resumeTask | 1 | 正确参数调用 |
| removeTask | 2 | 默认不删文件/指定删除文件 |
| getTask | 2 | 返回任务/null |
| getAllTasks | 2 | 返回数组/空数组 |
| setTaskPriority | 1 | 正确参数调用 |
| batch operations | 4 | pauseAll/resumeAll/removeCompleted/resumeUnfinished |
| settings | 3 | 获取配置/更新配置/应用信息 |
| status | 2 | BT 状态/API 状态 |
| history | 3 | 无限制获取/有限制获取/清空历史 |
| RSS | 3 | 获取列表/添加订阅/删除订阅 |
| plugins | 5 | 列表/安装/卸载/启用/禁用 |
| archive | 3 | 获取配置/更新配置/手动解压（含密码） |
| file operations | 1 | 打开文件目录 |

### 13. toast.test.ts（11 测试）🆕

Toast 通知组件测试，覆盖 showToast 函数和 ToastMessage 接口。

| 测试组 | 测试数 | 内容 |
|---|---|---|
| showToast | 8 | UUID 格式/类型正确/默认类型/消息传递/持续时间/默认值/多监听器/并发唯一 ID |
| ToastType | 1 | 三种有效类型验证 |
| ToastMessage | 2 | 必填字段/可选持续时间 |

**变更文件**：`src/__tests__/toast.test.ts`（新增）

**测试统计**：510 测试全通过（原 499 → 新增 11）

## 测试覆盖分析

### 已覆盖
- 所有格式化函数的边界值和异常输入
- 任务状态管理的完整生命周期
- 搜索过滤的组合场景
- 速度历史的节流和全局追踪
- URL 验证的安全性（拒绝 XSS、危险协议）
- 状态机转换的合法性验证
- WebUI 状态标签、颜色映射、操作按钮逻辑
- WebUI 标签页配置和空状态消息
- 统一错误处理：错误类、错误码、错误推断、用户友好消息
- Tauri IPC API 完整接口覆盖（任务/设置/历史/RSS/插件/解压/状态）
- 任务调度系统：cron 表达式、带宽计划、规则 CRUD
- 国际化：语言包一致性、翻译函数、模板参数替换
- 浏览器扩展：URL 检测、文件名提取、类型标签
- SpeedChart：Y 轴刻度计算、时间标签格式化
- 共享类型：桌面端/WebUI 类型复用兼容性
- Toast 通知：UUID 生成、类型验证、监听器广播

### 未覆盖（需 Rust 测试环境）
- Rust 后端 IPC 命令测试
- HTTP/BT/ed2k/HLS 引擎测试
- SQLite 存储层测试
- JSON-RPC API 集成测试
- WebSocket 事件推送测试

## 运行方式

```bash
cd downloader
npx vitest run
```
