# 全协议下载器 - 使用手册

## 快速开始

### 环境要求

- **操作系统**: Windows 10+、macOS 12+、Ubuntu 22.04+
- **Node.js**: 18+
- **Rust**: 1.70+（仅编译需要，运行不需要）

### 安装依赖

```bash
cd downloader
npm install
```

### 开发模式运行

```bash
# 启动前端开发服务器 + Tauri 桌面窗口
npm run tauri dev
```

---

## 编译为便携 EXE

### 构建命令

```bash
# 完整构建（前端 + Rust 后端 → 便携安装包）
npm run tauri build
```

### 构建产物

构建完成后，产物位于：

```
src-tauri/target/release/bundle/nsis/
└── Downloader_1.0.0_x64-setup.exe    # 安装程序

src-tauri/target/release/
└── downloader.exe                      # 主程序
```

### 编译环境要求

| 依赖 | 版本 | 安装方式 |
|------|------|----------|
| Node.js | 18+ | https://nodejs.org |
| Rust | 1.70+ | `winget install Rustlang.Rustup` |
| VS Build Tools | 2022 | `winget install Microsoft.VisualStudio.2022.BuildTools` |

> VS Build Tools 安装时需勾选「C++ 桌面开发」工作负载。

### 便携模式特性

| 特性 | 说明 |
|------|------|
| 单文件分发 | 一个 EXE 文件，拷贝即用 |
| 无需管理员权限 | 安装到用户目录，不写 Program Files |
| 数据本地存储 | 配置、数据库、插件全部在 EXE 同级 `data/` 目录 |
| 关闭无残留 | 删除 EXE 所在目录即完全卸载，无注册表、无 AppData |
| 开箱即用 | 双击 EXE → 选择安装目录 → 立即可用 |

### 数据目录结构

安装后，EXE 所在目录结构如下：

```
安装目录/
├── Downloader.exe          # 主程序
├── data/                   # 所有用户数据（便携）
│   ├── downloader.db       # SQLite 数据库（任务、历史、RSS、调度、密码）
│   ├── config.toml         # 应用配置
│   ├── temp/               # 临时文件（HLS 分段等）
│   └── plugins/            # WASM 插件
└── resources/              # 内置资源（Tracker 列表等）
```

### 卸载方式

直接删除安装目录即可，**零残留**：
- 无注册表项
- 无 AppData 数据
- 无 Program Files 文件
- 无开始菜单快捷方式

---

## 功能说明

### 支持的协议

| 协议 | 说明 |
|------|------|
| HTTP/HTTPS | 多线程分块下载，支持断点续传、镜像加速、Basic/Digest 认证 |
| FTP | FTP 协议下载 |
| BitTorrent | .torrent 文件和 magnet 链接，DHT/PEX/UPnP/LSd |
| ed2k | eDonkey 协议，可配置服务器列表，KAD 网络，多源查找 |
| HLS/DASH | 流媒体下载，m3u8/mpd 解析，相对 URL 自动解析，TS 分段合并，AES-128 解密 |

### 任务管理

- **添加任务**: 点击工具栏「添加任务」按钮或按 `Ctrl+N`，粘贴下载链接
- **批量导入**: 支持文本粘贴、剪贴板导入、通配符生成（`file(1-100).zip`）
- **暂停/恢复**: 右键菜单或空格键切换选中任务状态
- **批量操作**: 全部暂停、全部开始、清除已完成任务
- **删除任务**: 右键菜单或 `Delete` 键，可选同时删除文件
- **优先级**: 右键菜单设置高/普通/低优先级
- **速度限制**: 右键菜单可设置单任务下载/上传速度限制（支持 >4GB/s）
- **拖拽**: 直接拖拽 .torrent 文件到窗口
- **镜像加速**: 任务详情中可添加镜像 URL，支持多源并行下载（认证/Cookie/Headers 同步应用到所有镜像）

### 快捷键

| 快捷键 | 功能 |
|--------|------|
| `Ctrl+N` | 添加任务 |
| `Delete` | 删除选中任务 |
| `Space` | 暂停/恢复选中任务 |
| `Ctrl+A` | 全选任务 |
| `Escape` | 取消选择/关闭对话框 |
| `Ctrl+S` | 保存设置（设置页内） |
| `Ctrl+F` | 聚焦搜索框 |

### 设置中心（7 个分类）

#### 常规
- **语言**: 中文 / English（完整双语，含系统通知）
- **主题**: 亮色 / 暗色 / 跟随系统（实时响应系统主题切换）
- **字体大小**: 10-32px 可调
- **关闭/最小化时缩小到托盘**: 合并开关，一个控制两个行为
- **开机自启动**: 系统启动时自动运行

#### 下载
- **默认下载目录**: 新任务的默认保存路径
- **完成目录**: 下载完成后自动移动到指定目录（可选）
- **临时目录**: 下载临时文件存放位置
- **最大并发任务数**: 1-32（运行时动态调整，无需重启）
- **单任务最大连接数**: 1-256
- **全局最大连接数**: 1-1000
- **自动重试次数**: 0-99
- **重试间隔**: 秒
- **全局最大下载/上传速度**: bytes/sec，0 表示不限速（支持 >4GB/s，自动饱和）

#### 连接
- **BT 监听端口**: 默认 6881
- **ed2k 监听端口**: 默认 4661
- **ed2k 服务器列表**: 可自定义，每行一个 `ip:port`，默认包含 2 个公共服务器
- **HTTP API 端口**: JSON-RPC 服务端口，0 禁用
- **API 认证 Token**: 留空表示不需要认证
- **端口映射**: UPnP + NAT-PMP 合并开关
- **代理**: 无 / HTTP / SOCKS5，支持用户名密码认证
- **连接超时 / 读取超时**: 秒，并排显示

#### BitTorrent
- **DHT 网络**: 分布式哈希表，无 Tracker 下载
- **PEX**: Peer 交换协议
- **LSD**: 局域网服务发现
- **加密策略**: 禁用 / 启用 / 强制
- **做种比率限制**: 达到后自动停止做种（默认 2.0）
- **做种时间限制**: 分钟，0 不限（默认 1440）
- **全局停止做种**: 开启后下载完成立即停止做种，不上传数据
- **Trackers 文件**: 自定义 Tracker 列表路径

#### HTTP
- **User-Agent**: HTTP 请求标识字符串
- **Cookie 策略**: 自动 / 手动 / 禁用
- **Referer 策略**: 严格 / 宽松 / 不发送
- **最大重定向次数**: 0-50
- **验证 SSL 证书**: 关闭可跳过证书验证

#### 通知
- **任务完成通知**: 开关
- **任务错误通知**: 开关
- **通知声音**: 开关
- **通知位置**: 右下角 / 左下角 / 右上角 / 左上角
- **通知持续时间**: 秒

#### 高级
- **日志级别**: DEBUG / INFO / WARN / ERROR
- **日志文件大小限制**: MB
- **日志保留天数**: 天
- **数据库备份间隔**: 小时，0 不自动备份
- **临时文件清理间隔**: 小时，0 不自动清理
- **内存限制**: MB，0 不限
- **下载完成后动作**: 无操作 / 关机 / 休眠 / 睡眠 / 执行命令

> 设置页底部有「取消」「应用」「保存」三个按钮。「应用」保存但不关闭，`Ctrl+S` 快捷键直接保存并关闭。

### 任务调度

- **定时规则**: 基于 cron 表达式的定时开始/暂停/带宽计划
- **带宽计划**: 按时间段自动调整下载/上传速度限制（重启后保留）
- **Cron 预设**: 每天 00:00、每天 08:00、工作日 09:00 等常用预设
- **按星期生效**: 带宽计划可指定星期几生效

### RSS 订阅

- 添加 RSS Feed URL，自动匹配下载
- 支持 OPML 导入/导出
- 过滤规则：正则表达式、通配符、关键词，按标题/链接/描述匹配
- 自动去重，防止重复下载（最多保留 10000 条记录）
- 所有订阅重启后保留

### 自动解压

支持 ZIP、TAR、TAR.GZ、TAR.BZ2、7Z、RAR 格式：
- 下载完成后自动解压
- 密码管理器，支持多组密码，按域名自动匹配
- 可配置解压后删除原文件
- **安全防护**: Zip Slip 路径遍历攻击防护（7z 使用临时目录解压 + 路径校验后再移动）

### ed2k 服务器配置

在设置 → 连接中可自定义 ed2k 服务器列表：
- 默认包含 2 个公共服务器（eMule、TV Underground）
- 每行一个服务器地址（`ip:port` 格式）
- 优先级：链接内嵌服务器 > 配置自定义服务器 > 默认服务器
- 重启后保留

### 浏览器扩展

Chrome MV3 扩展，位于 `extension/` 目录：
- 自动拦截下载链接
- 视频资源嗅探（m3u8/mpd/视频标签）
- 右键发送到下载器
- 批量下载当前页面资源

### WebUI

独立 Web 界面，通过 JSON-RPC 2.0 over WebSocket 通信：
- 默认端口: 6801
- aria2 兼容 API（`aria2.addUri`、`aria2.tellStatus` 等）
- Token 认证
- 响应式设计

### 下载历史

- 历史记录搜索（文件名、URL、协议）
- 统计摘要（总大小、平均速度、总耗时）
- 最近 30 天流量图表
- CSV 导出（英文表头，国际化兼容）
- 一键清空

### 速度图表

实时速度监控：
- 5 分钟 / 30 分钟 / 1 小时视图
- Canvas 高性能渲染（颜色/尺寸缓存，HiDPI 适配）
- 下载/上传速度双线显示
- 悬停查看详细数据

### 进度显示

- **任务列表进度条**: 按状态变色（蓝=下载中、绿=完成、红=错误、黄=暂停、灰=等待）
- **下载中动画**: 脉冲呼吸灯效果
- **百分比文字**: 嵌入进度条内部，带阴影保证可读性
- **全局进度条**: 底部状态栏显示所有下载任务的加权平均进度

### 界面交互

- **工具栏三区布局**: 左侧操作（添加/暂停/清理）→ 中间功能（搜索/过滤/RSS/插件等）→ 右侧固定（计数/设置），设置按钮始终可见不被挤压
- **全局 hover 态**: 所有可交互元素（按钮、标签页、菜单项、关闭按钮）均有悬停视觉反馈
- **主题实时切换**: 跟随系统模式下，操作系统切换明暗主题时应用即时响应
- **HiDPI 适配**: 速度图表 Canvas 渲染正确处理高 DPI 显示器，tooltip 定位准确
- **虚拟滚动**: 任务列表使用虚拟化渲染，支持数千任务无卡顿

### 下载性能优化

- **智能分块**: 根据文件大小自动调整分块数（小文件1块、大文件最多32块）
- **分块失败自动重试**: 每个分块最多重试3次，指数退避（1s/2s/4s）
- **连接池优化**: TCP keepalive、连接复用、每主机最大8个空闲连接
- **TCP_NODELAY**: 减少小数据包延迟
- **镜像加速**: 支持多源并行下载，认证/Cookie/Headers 同步应用到所有镜像
- **并发控制**: 全局信号量控制同时下载任务数，运行时动态调整（最小值保护为1）
- **HTTP 认证**: 支持 Basic/Digest 认证（需登录的资源）
- **自定义 Cookie**: 支持传递 Cookie 下载需登录的资源
- **自定义 Headers**: 支持设置 Referer 等防盗链 Header

### 数据持久化

所有用户数据在重启后自动保留：
- **RSS 订阅**: 订阅列表、过滤规则、去重记录全部持久化到 SQLite
- **调度规则**: 定时任务持久化
- **带宽计划**: 按时段限速计划持久化
- **解压密码**: 密码管理器数据持久化
- **ed2k 服务器**: 自定义服务器列表持久化到 config.toml
- **下载配置**: 通过 config.toml 持久化

### BT 做种管理

- **做种比率限制**: 达到设定比率后自动停止做种（默认 2.0）
- **做种时间限制**: 达到设定时间后自动停止做种（默认 1440 分钟）
- **实时做种状态**: 状态栏显示做种速度和上传量
- **DHT 状态**: 显示活跃 peer 数作为近似节点数

### 下载完成后动作

在设置 → 高级中配置，所有下载任务完成后自动执行：
- **无操作**（默认）
- **关机**（60 秒倒计时，跨平台支持）
- **休眠**
- **睡眠**
- **执行自定义命令**

### 国际化

完整中英文双语支持：
- 所有 UI 文本通过 i18n 系统管理
- 系统通知跟随语言设置
- CSV 导出使用英文表头
- 速度图表 Canvas 文字跟随语言
- 主题切换实时响应（含系统主题变化监听）

---

## 项目结构

```
downloader/
├── src/                    # React 前端源码
│   ├── components/         # UI 组件（20+）
│   ├── hooks/              # React Hooks（useI18n, useClipboard, useTaskEvents）
│   ├── lib/                # 工具库、API、i18n、错误处理
│   │   └── locales/        # 语言包（zh.ts, en.ts）
│   ├── stores/             # Zustand 状态管理
│   ├── shared/             # 共享类型定义
│   └── __tests__/          # 前端测试（17 个文件，617 个用例）
├── src-tauri/              # Rust 后端源码
│   ├── src/
│   │   ├── commands/       # Tauri IPC 命令（48 个）
│   │   ├── engine/         # 下载引擎（HTTP/BT/HLS/ed2k）
│   │   ├── storage/        # SQLite 数据库 + TOML 配置
│   │   ├── api/            # JSON-RPC 2.0 + WebSocket
│   │   ├── archive/        # 自动解压模块
│   │   ├── plugin/         # WASM 插件系统（Wasmtime 沙箱）
│   │   ├── rss/            # RSS 引擎（预编译正则匹配）
│   │   ├── schedule/       # 调度管理（cron + 带宽计划）
│   │   └── util/           # 工具函数
│   └── resources/          # BT Tracker 列表
├── extension/              # Chrome 浏览器扩展（MV3）
├── eslint.config.js        # ESLint 配置（flat config）
├── vite.config.ts          # Vite 构建配置（含分包策略）
└── .github/workflows/      # CI/CD（lint + test + build）
```

---

## 测试

```bash
# 运行前端测试
npm test

# 运行前端 lint
npm run lint

# 运行 Rust 测试
cd src-tauri && cargo test
```

当前测试覆盖：617 个前端测试（17 个文件）+ 125 个 Rust 测试全部通过。

---

## 内存管理与性能

### 已修复的内存泄漏

| 问题 | 修复 |
|------|------|
| BT 监控任务永不取消 | 添加 CancellationToken，删除任务时自动取消监控 |
| RSS 去重集合无限增长 | 自动清理超过 10000 条的旧记录 |
| 并发信号量替换导致计数错误 | 改用动态调整许可数，最小值保护为 1 |
| 剪贴板轮询窗口隐藏时不停止 | 添加 visibilitychange 监听，隐藏时暂停 |
| SpeedChart 每帧读取 CSS 变量 | 缓存颜色值，仅主题变化时刷新 |
| Canvas 每帧重新分配缓冲区 | 缓存尺寸，仅大小变化时重新分配 |
| DB 查询未使用语句缓存 | 统一使用 prepare_cached |

### 下载引擎优化

- **HTTP 客户端**: TCP keepalive (60s)、连接池空闲超时 (90s)、每主机最大 8 空闲连接
- **分块策略**: 小文件(<10MB) 1块、中文件 4-8块、大文件(100MB+) 8-32块
- **RSS 正则**: 预编译模式匹配，避免每次轮询重新编译
- **速度限制**: u64→u32 饱和转换，支持 >4GB/s 不溢出

### 安全修复

| 问题 | 修复 |
|------|------|
| Shell 命令注入 | 白名单方式，仅允许 `a-zA-Z0-9 .-_/:=@[],+~` |
| CORS 任意来源 | 白名单限制为 localhost:1420/3000 + tauri://localhost |
| API 无认证 | 空 token 拒绝所有请求（fail-closed） |
| SpeedChart XSS | DOM API + textContent 替代 innerHTML |
| WASM 沙箱逃逸 | 64MB 内存限制 + 路径遍历防护 + 域名白名单 |
| Plugin http_get 无域名检查 | enforced allowed_domains 白名单（精确+子域名匹配） |
| RPC addTask 无文件名验证 | 调用 validate_filename 清理非法字符 |
| DB JSON 链式 unwrap | match 安全访问替代 unwrap().unwrap() |
| DB 初始化 panic | ? 传播 + 用户可读错误提示 |
| 7z 路径遍历漏洞 | 改用临时目录解压 → 路径校验 → 移动到目标目录 |
| BT 文件名路径遍历 | `add_bt_task_with_files` 补充 `validate_filename` 校验 |
| mirror 下载未应用认证 | auth/cookie/headers 同步应用到所有镜像 URL |
| mirror HEAD 探测未应用认证 | 探测请求也应用 auth/cookie/headers |
| resume_task 静默成功 | 任务不存在时返回错误，DB 状态不再不一致 |
| ed2k 数据写入偏移错误 | 解析 SENDINGPART 返回的偏移量，seek 到正确位置 |
| ed2k 下载双计数 | 添加 completed_pieces 追踪，防止跨源重复计数 |
| HLS 相对 URL 不解析 | m3u8 解析时自动将相对路径解析为绝对 URL |
| DASH BaseURL 文本污染 | 仅处理 BaseURL 标签内的文本，忽略其他标签 |
| HLS hex_decode 静默错误 | 无效十六进制字符返回错误而非转为 0x00 |
| WebSocket 无认证 | 添加 token 认证（空 token 时拒绝所有连接） |
| 下载后动作误触发 | 暂停任务不再阻止关机/休眠等后置动作 |
| magnet/ed2k 文件名非法字符 | extract_filename 协议感知提取 + sanitize_filename 清理 |
| 任务进度崩溃丢失 | 下载中每 10 秒定期保存进度到 DB |
| BT 文件列表重启丢失 | 种子文件列表持久化到 task_files 表 |
| 归档历史缺少速度/时长 | 归档 SQL 计算 average_speed 和 duration |
| BT peer 数据伪造 | 改为真实聚合数据（不伪造单个 peer 条目） |
| 生产代码 expect/unwrap | 全部消除，改用 unwrap_or_default/match |
| 批量操作静默错误 | 12 处 `let _ =` 改为 `if let Err(e) = { warn! }` |
| resume_all_tasks TOCTOU | 批量获取 manager 锁，避免逐个锁定竞争 |
| RPC DB 操作静默错误 | delete/pause/resume 添加 warn 日志 |

### 功能修复

| 问题 | 修复 |
|------|------|
| 恢复任务丢失代理/认证/限速 | TaskParams 持久化到 DB metadata 列 |
| 恢复失败任务无效 | 检测已退出任务，自动重新添加 |
| 全部暂停不暂停 BT | pause_all 改为 async，支持 BT 引擎暂停 |
| 协议检测误判 | `.contains(".m3u8")` → `.ends_with(".m3u8")` 精确匹配 |
| 镜像下载无重试 | 添加 3 次指数退避重试 |
| 速度限制 >4GB/s 溢出 | `as u32` → `.min(u32::MAX) as u32` 饱和转换 |

### 性能优化

- **RPC 查询优化**: `tellActive`/`tellWaiting` 使用索引查询，`getGlobalStat` 使用 SQL 聚合
- **批量删除**: `purgeCompleted` 改用单条 `DELETE ... WHERE state IN (...)` SQL
- **DB 行映射**: 提取 `row_to_task()` 辅助函数，消除 4 处重复代码
- **SpeedChart**: 移除鼠标移动时的 React 重渲染（仅更新 ref，rAF 循环读取）
- **速度历史**: `splice(0, n)` 改为长度限制，O(1) 摊销

| 优化 | 说明 |
|------|------|
| RPC 查询优化 | `tellActive`/`tellWaiting` 使用 `get_tasks_by_state` 索引查询 |
| DB 行映射提取 | 消除 4 处重复的 TaskStatus 行映射代码 |
| ed2k 启动配置 | 启动时从 config 传递 ed2k 服务器列表 |
| db.rs CRUD 测试 | 新增 7 个数据库操作测试 |

### UI 修复

| 问题 | 修复 |
|------|------|
| 设置页文本溢出 | SettingRow 添加 `break-words` 自动换行 |
| ed2k 服务器列表无法多行输入 | textarea 改为独立布局块，支持垂直调整大小 |
| 任务名截断无提示 | TaskList/TaskDetail 添加 `title` 属性，hover 显示完整文本 |
| 插件描述截断无提示 | PluginManager 添加 `title` 属性 |
| Toast 定时器泄漏 | 组件卸载时清除所有 setTimeout |
| Dialog.tsx 死代码 | 删除未使用的 153 行共享对话框组件 |
| textarea 不可调整大小 | AddTaskDialog/BatchImportDialog 改为 `resize-y` |
| SpeedTracker unwrap | `unwrap()` → `match` 安全访问 |

### 版本管理

版本号统一管理三个文件：`package.json`、`tauri.conf.json`、`Cargo.toml`。

```powershell
# 查看当前版本
.\scripts\version.ps1

# 递增版本号
.\scripts\version.ps1 patch         # 1.0.0 -> 1.0.1
.\scripts\version.ps1 minor         # 1.0.0 -> 1.1.0
.\scripts\version.ps1 major         # 1.0.0 -> 2.0.0

# 直接设置版本号
.\scripts\version.ps1 set 1.2.3
```

---

## 已知限制

1. **Rust 环境**: 编译桌面版需要安装 Rust 工具链（https://rustup.rs）
2. **ed2k**: 需要连接 ed2k 服务器才能下载，默认服务器可能不稳定，建议自定义
3. **BT DHT 节点数**: 使用活跃 peer 数作为近似值（librqbit v8 不暴露路由表大小）
4. **集成测试**: 端到端下载流程测试尚未覆盖

---

## 故障排除

| 问题 | 解决方案 |
|------|----------|
| `cargo` 未找到 | 安装 Rust: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` |
| 前端构建失败 | 运行 `npm install` 后重试 |
| 下载无速度 | 检查代理设置、防火墙规则 |
| BT 下载慢 | 确保 DHT 已连接，检查监听端口是否被占用 |
| ed2k 连接失败 | 在设置中配置可靠的 ed2k 服务器，检查端口 4661 |
| HLS 下载失败 | 检查网络连接，部分直播流可能已结束，相对 URL 现已自动解析 |
| 设置不生效 | 部分设置需要重启应用（如代理、HTTP 配置） |
| 缺少 WebView2 | Windows 10 1803+ 自带，旧版需安装 [WebView2 Runtime](https://developer.microsoft.com/en-us/microsoft-edge/webview2/) |
