# 全协议下载器 PRD（Product Requirements Document）

> 版本：v4.0  |  日期：2026-05  |  状态：Draft
> 更新：基于 Tauri v2 / libtorrent / ed2k / UI-UX 四份调研报告，补充详细技术实现规范和步骤
> v4.0 更新：基于 IDM/FDM/JDownloader/Motrix/aria2/qBittorrent 等竞品调研，新增浏览器集成、远程管理、自动解压、RSS、插件系统、媒体流下载等需求，优化架构设计

---

## 1. 产品概述

### 1.1 产品定位

一款**绿色便携、全协议支持、零广告**的桌面下载器。覆盖日常所有下载场景：HTTP/HTTPS/FTP 基础下载、BitTorrent（种子/磁力链）、eDonkey2000（ed2k）P2P 协议。解压即用、关闭无残留、完全离线可用。

**核心价值**：不联网也能用，联网更强大。无广告、无弹窗、无捆绑、无遥测。

### 1.2 目标用户

| 用户群 | 典型场景 |
|---|---|
| 所有用户 | 日常文件下载、浏览器下载替代 |
| 高级用户 / 资源爱好者 | 大文件多协议下载、长时间挂机做种 |
| 开发者 / 技术人员 | 需要便携工具、不想污染系统环境 |
| 离线/弱网用户 | 断点续传、批量下载、离线管理 |

### 1.3 设计原则

- **Portable First**：零安装、零残留，所有数据写入程序所在目录
- **协议统一**：不同协议的任务在同一个界面管理，体验一致
- **稳定可靠**：断点续传、校验重下、自动做种
- **性能优先**：充分利用带宽，支持高并发任务
- **零广告零遥测**：不收集任何数据，不展示任何广告
- **离线可用**：核心功能不依赖网络服务，配置本地化
- **用户主权**：所有行为可配置，不强制任何在线功能

---

## 2. 功能需求

### 2.1 协议支持总览

| 协议 | 状态 | 说明 |
|---|---|---|
| HTTP/HTTPS | 核心 | 分块下载、断点续传、自动重定向 |
| FTP/FTPS | 核心 | 被动模式、目录列表、断点续传 |
| BitTorrent (种子) | 核心 | .torrent 文件解析、选择性下载 |
| BitTorrent (磁力链) | 核心 | magnet 链接、DHT 元数据获取 |
| ed2k | 核心 | ed2k 链接、服务器/KAD 网络 |
| Metalink | 增强 | .metalink 文件、多源下载 |
| HLS/DASH 流媒体 | 增强 | m3u8/mpd 流媒体下载、分片合并 |
| 直链批量 | 增强 | 文本导入、正则匹配、通配符 |

---

### 2.2 协议解析能力

#### 2.2.1 HTTP/HTTPS/FTP 下载

| 能力 | 说明 |
|---|---|
| 多线程分块 | 自动检测服务器 Range 支持，启用 4-16 线程分块下载 |
| 断点续传 | 支持 Range 请求，记录已下载字节偏移，重启后从断点继续 |
| 自动重定向 | 支持 301/302/303/307/308 重定向，最多跟随 10 层 |
| 文件名提取 | 从 Content-Disposition 头或 URL 路径提取文件名 |
| 大文件支持 | 支持 >4GB 文件（64 位文件指针） |
| HTTPS 证书 | 支持系统证书库，可配置忽略证书错误（仅调试用） |
| FTP 被动模式 | 默认使用被动模式（PASV），兼容防火墙环境 |
| FTP 目录列表 | 支持 LIST/NLST 命令，解析 Unix/Windows 风格目录格式 |
| Cookie 支持 | 自动处理 Set-Cookie，支持从浏览器导入 Cookie |
| User-Agent | 可配置 UA 字符串，默认模拟主流浏览器 |
| 限速控制 | 单任务/全局上传下载限速 |
| 超时设置 | 连接超时、读取超时、空闲超时均可配置 |
| 代理支持 | HTTP/HTTPS/SOCKS5 代理，支持代理认证 |

#### 2.2.2 BitTorrent — 种子文件解析

| 能力 | 说明 |
|---|---|
| 支持格式 | .torrent 文件（单文件 / 多文件 / 包含 BEP 编码） |
| 解析内容 | 文件列表、文件大小、piece 长度、info_hash、announce / tracker 列表、创建者、注释 |
| 选择性下载 | 用户可在文件树中勾选 / 取消子文件 |
| 加密连接 | 支持 MSE/PE（Message Stream Encryption / Protocol Encryption） |
| DHT / PEX | 支持 DHT（BEP-5）、PEX（BEP-11）、LSD（Local Service Discovery） |
| 做种（Seeding） | 下载完成后自动做种，可设置做种比率 / 时间上限 |

#### 2.1.2 BitTorrent — 磁力链解析

| 能力 | 说明 |
|---|---|
| 格式 | magnet:?xt=urn:btih:<hash>&dn=...&tr=... |
| Metadata 获取 | 通过扩展协议（BEP-9）从 DHT 网络获取 .torrent 元数据 |
| Tracker 补充 | 自动合并磁力链中携带的 tracker 与内置 tracker 列表 |
| 在线预览 | 元数据获取后展示文件列表，供用户选择 |

#### 2.1.3 eDonkey2000 (ed2k) 协议

| 能力 | 说明 |
|---|---|
| 格式 | ed2k://|file|<name>|<size>|<hash>|/ |
| 连接方式 | 支持连接 ed2k 服务器（server.met）与 Kademlia (KAD) 网络 |
| 分块下载 | 基于 ed2k 分块（9.28 MB per chunk）进行校验和重传 |
| 源交换 | 支持 Source Exchange（SUI）以增加资源来源 |
| 队列机制 | 遵守 ed2k 队列规则，支持 HighID / LowID 识别 |

#### 2.2.4 通用解析

- **剪贴板监听**：自动识别剪贴板中的 magnet / ed2k / http(s) 链接并弹出添加任务对话框
- **拖拽支持**：拖拽 .torrent 文件到主窗口直接添加任务
- **批量导入**：支持从文本文件批量导入链接（每行一个）
- **协议自动识别**：粘贴或输入链接时自动识别协议类型，无需手动选择

#### 2.2.5 批量下载能力

| 能力 | 说明 |
|---|---|
| 文本批量导入 | 从 .txt 文件导入链接列表（每行一个 URL），自动创建任务 |
| 通配符匹配 | 支持通配符模式，如 `file(1-100).zip` 自动生成 100 个下载链接 |
| 正则替换 | 使用正则表达式批量生成链接，如 `(\d+)` 替换为序号 |
| 序列号填充 | 支持数字序号（1,2,3...）、零填充（001,002,003...）、字母序号（a,b,c...） |
| 批量设置 | 导入时统一设置保存目录、文件名模板、并发数 |
| 任务分组 | 批量导入的任务自动归为一组，支持整组操作（暂停/开始/删除） |
| 去重检测 | 自动检测重复链接，避免重复下载 |
| 链接有效性预检 | 导入前可选 HEAD 请求检测链接是否有效，过滤无效链接 |

---

### 2.3 下载管理能力

#### 2.3.1 任务生命周期

`
[创建] → [等待/排队] → [下载中] → [校验中] → [做种中(BT)] → [完成]
                         ↕
                      [暂停] → [恢复]
                         ↕
                      [错误/重试]
`

#### 2.3.2 断点续传

- 所有协议均支持**断点续传**
- 任务状态（已下载分块位图、连接信息）持久化到本地数据库
- 程序异常退出后重启自动恢复未完成任务
- BT 任务：保留 piece bitmap，校验已有数据后从断点继续
- ed2k 任务：保留已下载 chunk 的 hash 校验记录

#### 2.3.3 并发下载

| 参数 | 默认值 | 可配置范围 | 说明 |
|---|---|---|---|
| 最大同时下载任务数 | 3 | 1 ~ 20 | 全局并行任务上限 |
| 单任务最大连接数 | 64 | 8 ~ 256 | 每个任务可建立的最大 peer 连接数 |
| 全局最大连接数 | 200 | 50 ~ 500 | 所有任务共享的连接池上限 |
| 最大上传速度 | 不限 | 0（不限）~ 自定义 | 全局上传限速 |
| 最大下载速度 | 不限 | 0（不限）~ 自定义 | 全局下载限速 |
| 排队策略 | FIFO | FIFO / 优先级 | 等待队列中的任务调度方式 |

#### 2.3.4 下载目录管理

- 首次使用时提示用户选择默认下载目录
- 添加任务时可临时修改该任务的保存目录
- 支持“下载中”与“完成”分目录存放
- 所有目录配置保存在程序目录内（便携化）

#### 2.3.5 文件校验

- BT：每个 piece 下载完成后立即做 SHA-1 校验
- ed2k：每个 chunk 下载完成后做 MD4/ed2k hash 校验
- HTTP/FTP：支持 MD5/SHA1/SHA256 校验和验证
- 完整下载后可手动触发全文件校验

#### 2.3.6 任务优先级

| 优先级 | 说明 | 行为 |
|---|---|---|
| 高 | 紧急任务 | 优先分配带宽和连接，可抢占低优先级任务资源 |
| 普通 | 默认优先级 | 正常排队，按 FIFO 或创建时间排序 |
| 低 | 后台任务 | 仅在高/普通任务空闲时分配资源 |
| 暂停 | 手动暂停 | 不参与排队，需手动恢复 |

- 支持拖拽调整任务顺序
- 支持批量设置优先级
- 优先级变更实时生效

#### 2.3.7 定时下载

| 能力 | 说明 |
|---|---|
| 定时开始 | 设置任务在指定时间自动开始下载 |
| 定时暂停 | 设置任务在指定时间自动暂停 |
| 计划任务 | 支持 cron 表达式定义复杂时间规则 |
| 带宽计划 | 不同时段使用不同的速度限制（如夜间不限速） |
| 做种计划 | 设置做种时间段，非指定时间暂停做种 |

#### 2.3.8 下载完成后动作

| 动作 | 说明 |
|---|---|
| 关机 | 下载完成后自动关机（所有任务完成后） |
| 休眠 | 下载完成后自动休眠 |
| 睡眠 | 下载完成后自动睡眠 |
| 锁屏 | 下载完成后自动锁屏 |
| 关闭程序 | 下载完成后自动关闭下载器 |
| 执行命令 | 下载完成后执行自定义命令/脚本 |
| 发送通知 | 下载完成后发送系统通知（默认开启） |
| 打开文件 | 下载完成后自动打开文件 |
| 打开目录 | 下载完成后自动打开文件所在目录 |

#### 2.3.9 任务搜索与过滤

| 能力 | 说明 |
|---|---|
| 关键词搜索 | 按任务名称搜索，支持实时过滤 |
| 状态过滤 | 按状态筛选：下载中/暂停/完成/错误/做种 |
| 协议过滤 | 按协议类型筛选：HTTP/BT/ed2k |
| 大小过滤 | 按文件大小范围筛选 |
| 时间过滤 | 按添加时间/完成时间筛选 |
| 排序方式 | 按名称/大小/进度/速度/剩余时间/添加时间排序 |
| 标签系统 | 为任务添加自定义标签，按标签分类管理 |

#### 2.3.10 文件管理

| 能力 | 说明 |
|---|---|
| 自动重命名 | 下载时自动处理文件名冲突，支持添加序号后缀 |
| 文件预览 | BT 任务支持在下载前预览文件列表和大小 |
| 打开文件 | 下载完成后直接打开文件 |
| 打开目录 | 打开文件所在的文件夹 |
| 移动文件 | 将已完成的文件移动到其他目录 |
| 删除文件 | 删除任务时可选同时删除已下载的文件 |
| 文件关联 | 双击 .torrent 文件自动添加到下载器 |

---

### 2.4 界面需求 (GUI)

#### 2.4.1 主窗口布局

`
┌──────────────────────────────────────────────────────────────┐
│  菜单栏：文件 | 任务 | 工具 | 帮助                           │
│  工具栏：[添加链接] [添加种子] [暂停全部] [开始全部] [设置]    │
├──────────────────────────────────────────────────────────────┤
│  任务列表（表格视图）                                         │
│  ┌────┬────────┬──────┬────────┬──────┬──────┬─────────────┐│
│  │ 状态│ 文件名  │ 大小 │ 进度   │ 速度 │ 来源 │ 剩余时间    ││
│  ├────┼────────┼──────┼────────┼──────┼──────┼─────────────┤│
│  │ ▶  │ movie…  │ 4.2G │ 67.3%  │ 5.2M │ 32   │ 12min       ││
│  │ ⏸  │ docs…   │ 200M │ 100%   │ --   │ --   │ 完成        ││
│  └────┴────────┴──────┴────────┴──────┴──────┴─────────────┘│
├──────────────────────────────────────────────────────────────┤
│  详情面板（选中任务时展开）                                    │
│  ┌─ 概要 ──┬─ 文件 ──┬─ 连接 ──┬─ Tracker ──┬─ 日志 ──────┐│
│  │ 基本信息  │ 文件树  │ Peer列表│ tracker状态│ 实时日志    ││
│  └─────────┴────────┴────────┴───────────┴────────────┘│
│                                                             │
│  状态栏：全局速度 ▼ 5.2MB/s ▲ 1.1MB/s │ 活跃: 3 │ DHT: OK │
└──────────────────────────────────────────────────────────────┘
`

#### 2.4.2 核心界面元素

**任务列表**
- 表格展示所有任务，支持按列排序（名称、大小、进度、速度等）
- 进度条内嵌在单元格中，带百分比文字
- 状态图标：下载中 ▶、暂停 ⏸、等待 ⏳、完成 ✅、错误 ❌、做种 🌱
- 右键菜单：开始 / 暂停 / 删除 / 重新校验 / 打开文件 / 打开目录 / 属性

**详情面板 — 概要 Tab**
- 任务名称、保存路径、总大小、已下载、分享比率
- Info Hash / ed2k Hash
- 创建时间、完成时间

**详情面板 — 连接 Tab**
- Peer 列表：IP、客户端、上传/下载速度、已交换数据量、连接状态
- 支持封禁单个 Peer

**详情面板 — Tracker Tab**
- Tracker 地址、状态（工作 / 超时 / 错误）、Peer 数量
- 手动添加 / 删除 Tracker

**详情面板 — 日志 Tab**
- 按时间戳滚动显示该任务的事件日志（连接、校验、错误等）

**全局状态栏**
- 全局实时下载速度（每秒更新）
- 全局实时上传速度
- 活跃任务数 / 任务总数
- DHT 网络状态（节点数 / 已连接 / 未连接）
- ed2k 服务器连接状态

#### 2.4.3 添加任务对话框

```
┌─────────────────────────────────────────────────────────┐
│  添加下载任务                                              │
├─────────────────────────────────────────────────────────┤
│  链接地址：                                               │
│  ┌─────────────────────────────────────────────────────┐│
│  │ https://example.com/file.zip                        ││
│  └─────────────────────────────────────────────────────┘│
│  支持：HTTP/HTTPS/FTP/magnet/ed2k/种子文件                │
│                                                         │
│  保存到：                                                │
│  ┌─────────────────────────────────────────────────────┐│
│  │ C:\Downloads\                              [浏览...] ││
│  └─────────────────────────────────────────────────────┘│
│                                                         │
│  文件名：                                                │
│  ┌─────────────────────────────────────────────────────┐│
│  │ file.zip                                            ││
│  └─────────────────────────────────────────────────────┘│
│                                                         │
│  □ 添加后立即开始下载                                     │
│  □ 使用代理                                              │
│  □ 限速：[    ] KB/s                                     │
│                                                         │
│  [高级选项]                                    [取消] [确定]│
└─────────────────────────────────────────────────────────┘
```

#### 2.4.4 批量导入对话框

```
┌─────────────────────────────────────────────────────────┐
│  批量导入下载任务                                          │
├─────────────────────────────────────────────────────────┤
│  导入方式：                                               │
│  ○ 从文本文件导入                                         │
│  ○ 从剪贴板导入                                          │
│  ○ 使用通配符生成                                         │
│                                                         │
│  链接列表：                                               │
│  ┌─────────────────────────────────────────────────────┐│
│  │ https://example.com/file1.zip                       ││
│  │ https://example.com/file2.zip                       ││
│  │ https://example.com/file3.zip                       ││
│  │                                                     ││
│  └─────────────────────────────────────────────────────┘│
│  共 3 个链接，预计总大小：150 MB                           │
│                                                         │
│  保存到：C:\Downloads\                                    │
│                                                         │
│  □ 添加后立即开始下载                                     │
│  □ 去重检测                                              │
│  □ 链接有效性预检                                         │
│                                                         │
│  [从文件导入]  [从剪贴板]                    [取消] [确定]│
└─────────────────────────────────────────────────────────┘
```

#### 2.4.5 任务属性对话框

```
┌─────────────────────────────────────────────────────────┐
│  任务属性 - movie.mkv                                     │
├─────────────────────────────────────────────────────────┤
│  ┌─ 常规 ──┬─ 文件 ──┬─ 连接 ──┬─ Tracker ──┬─ 日志 ──┐│
│  │         │         │         │            │          ││
│  │ 任务ID:  │ 文件列表 │ Peer列表│ Tracker列表│ 事件日志 ││
│  │ abc123   │         │         │            │          ││
│  │ 协议: BT │ 1/5 文件│ 32 连接 │ 5/8 工作   │          ││
│  │ 大小:4.2G│         │         │            │          ││
│  │ 已下载:  │         │         │            │          ││
│  │ 2.8G    │         │         │            │          ││
│  │ 分享率:  │         │         │            │          ││
│  │ 1.5     │         │         │            │          ││
│  └─────────┴─────────┴─────────┴────────────┴──────────┘│
│                                                         │
│                                            [关闭]        │
└─────────────────────────────────────────────────────────┘
```

#### 2.4.6 速度图表

- 主窗口底部可展开速度历史图表
- 时间轴：最近 5 分钟 / 30 分钟 / 1 小时 可切换
- 双 Y 轴：下载速度（蓝色）、上传速度（绿色）
- 鼠标悬停显示具体数值

#### 2.4.7 通知

- 任务完成时系统托盘气泡通知
- 任务出错时系统托盘气泡通知
- 可在设置中关闭通知
- 支持自定义通知声音

#### 2.4.5 快捷键

| 快捷键 | 功能 |
|---|---|
| Ctrl+N | 添加新任务 |
| Ctrl+V | 从剪贴板添加链接 |
| Ctrl+O | 打开种子文件 |
| Delete | 删除选中任务 |
| Ctrl+A | 全选任务 |
| Space | 暂停/恢复选中任务 |
| Ctrl+S | 开始选中任务 |
| Ctrl+P | 暂停选中任务 |
| F2 | 重命名任务 |
| F5 | 刷新任务状态 |
| Ctrl+, | 打开设置 |
| Esc | 最小化/关闭对话框 |

#### 2.4.6 拖拽支持

- 拖拽 .torrent 文件到窗口添加任务
- 拖拽 magnet/ed2k/http 链接到窗口添加任务
- 拖拽文本文件（.txt）批量导入链接
- 拖拽调整任务顺序
- 拖拽文件到其他目录移动文件

#### 2.4.7 主题定制

| 能力 | 说明 |
|---|---|
| 亮色主题 | 默认浅色主题 |
| 暗色主题 | 深色主题，护眼模式 |
| 跟随系统 | 自动跟随系统主题设置 |
| 自定义颜色 | 可自定义进度条、状态图标、背景色等 |
| 字体设置 | 可调整界面字体大小和字体族 |

#### 2.4.8 系统托盘

- 最小化到系统托盘
- 托盘右键菜单：显示主窗口、开始全部、暂停全部、退出
- 托盘图标显示全局下载速度状态
- 双击托盘图标显示/隐藏主窗口

#### 2.4.9 下载历史与统计

| 能力 | 说明 |
|---|---|
| 下载历史 | 记录所有已完成的下载任务，支持按时间/名称/大小排序 |
| 历史搜索 | 按文件名、URL、日期搜索历史记录 |
| 统计信息 | 总下载量、总上传量、平均速度、完成任务数 |
| 流量图表 | 按天/周/月统计下载流量趋势图 |
| 清理历史 | 支持按时间范围清理历史记录 |
| 导出历史 | 支持导出历史记录为 CSV/JSON 格式 |

#### 2.4.10 工具栏自定义

- 支持自定义工具栏按钮显示/隐藏
- 支持调整按钮顺序
- 支持添加自定义快捷操作
- 支持工具栏位置（顶部/底部/左侧/右侧）

#### 2.4.11 浏览器集成（新增）

> 对标 IDM 浏览器扩展、JDownloader LinkGrabber，解决"从浏览器捕获下载链接"的核心场景

| 能力 | 说明 |
|---|---|
| 浏览器扩展 | 提供 Chrome / Firefox / Edge 扩展，自动将浏览器下载请求转发至本下载器 |
| 视频嗅探 | 扩展自动嗅探页面中的视频/音频资源（mp4/webm/mp3/m3u8），显示下载按钮 |
| 链接捕获 | 可配置拦截规则：全部链接 / 特定文件类型 / 特定域名 |
| 右键菜单 | 浏览器右键"使用下载器下载此链接/此视频" |
| 扩展通信 | 扩展通过 Native Messaging 或本地 HTTP API（localhost:port）与下载器通信 |
| 批量捕获 | 支持批量捕获页面中所有可下载链接，用户勾选后批量添加 |
| 智能过滤 | 自动过滤广告、追踪像素、小图标等非下载资源 |

**扩展架构**：
```
浏览器扩展 (content script + background script)
    │
    │  Native Messaging / HTTP POST localhost:port
    ▼
下载器本地 API 接收层
    │
    │  解析链接 → 添加任务
    ▼
Task Engine
```

#### 2.4.12 远程管理 WebUI（新增）

> 对标 qBittorrent WebUI、FDM 远程控制、JDownloader MyJDownloader，解决"远程管理下载任务"的需求

| 能力 | 说明 |
|---|---|
| Web 控制台 | 内置轻量 Web 服务器，浏览器访问 `http://localhost:port` 管理任务 |
| 任务管理 | 远程查看/添加/暂停/删除任务，查看进度和速度 |
| 实时更新 | WebSocket 推送任务状态变化，无需手动刷新 |
| 认证保护 | 用户名/密码认证，防止未授权访问 |
| 响应式设计 | 适配手机/平板浏览器，支持移动端管理 |
| HTTPS | 可选启用自签名 HTTPS，保护远程通信安全 |
| 局域网发现 | mDNS/SSDP 广播，方便局域网内快速发现设备 |

**WebUI 技术方案**：
- 复用前端 React 组件，打包为独立的 Web 版本
- 内嵌轻量 HTTP 服务器（axum），端口可配置
- WebSocket 实时推送任务状态
- 与桌面版共享同一份 Rust Core 逻辑

#### 2.4.13 下载完成自动解压（新增）

> 对标 JDownloader 自动提取、FDM ZIP 预览，解决"下载压缩包后手动解压"的痛点

| 能力 | 说明 |
|---|---|
| 自动解压 | 下载完成后自动解压 .zip/.rar/.7z/.tar.gz 等格式到指定目录 |
| 解压密码管理 | 支持保存常用解压密码，下载时自动尝试匹配 |
| 密码记忆 | 按域名/路径关联密码，下次自动应用 |
| 嵌套解压 | 支持压缩包内再包含压缩包的递归解压 |
| 解压后删除 | 可选解压成功后自动删除原始压缩包 |
| 格式支持 | .zip、.rar、.7z、.tar、.tar.gz、.tar.bz2、.tar.xz |
| 进度显示 | 解压进度在任务详情中显示 |
| 错误处理 | 解压失败（密码错误/文件损坏）时通知用户 |

#### 2.4.14 RSS 自动下载（新增）

> 对标 qBittorrent RSS、JDownloader RSS 系列，解决"自动订阅并下载更新内容"的需求

| 能力 | 说明 |
|---|---|
| RSS 订阅 | 添加 RSS/Atom feed URL，自动解析条目 |
| 过滤规则 | 支持正则表达式过滤标题/描述，匹配则自动创建下载任务 |
| 下载规则 | 按过滤规则指定保存目录、文件名模板、优先级 |
| 更新频率 | 可配置 RSS 轮询间隔（默认 30 分钟） |
| 去重 | 自动记录已处理的 RSS 条目，避免重复下载 |
| 通知 | 新匹配项下载时发送通知 |
| 导入/导出 | 支持 OPML 格式导入/导出订阅列表 |

**典型场景**：
- 订阅 Linux 发行版 ISO 更新
- 订阅播客/视频系列更新
- 订阅软件发布更新

#### 2.4.15 插件系统（新增）

> 对标 JDownloader 插件生态、qBittorrent 搜索引擎插件，解决"功能可扩展性"问题

| 能力 | 说明 |
|---|---|
| 插件接口 | 定义标准化插件 API（Rust trait + WASM 沙箱） |
| 插件类型 | 下载引擎插件、解析器插件、通知插件、后处理插件 |
| 插件管理 | 内置插件管理器：安装/卸载/启用/禁用/更新 |
| 插件市场 | 可选在线插件仓库，浏览和一键安装社区插件 |
| 安全隔离 | 插件运行在 WASM 沙箱中，无法直接访问系统 |
| 热加载 | 插件安装/更新无需重启程序 |

**插件接口示例**：
```rust
trait DownloaderPlugin {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn supported_protocols(&self) -> Vec<String>;
    fn parse_url(&self, url: &str) -> Option<TaskParams>;
    fn on_complete(&self, task: &TaskStatus) -> Result<()>;
}
```

#### 2.4.16 媒体流下载（新增）

> 对标 IDM 视频下载、yt-dlp，解决"HLS/DASH 流媒体下载"的新兴需求

| 能力 | 说明 |
|---|---|
| HLS 下载 | 解析 .m3u8 播放列表，下载所有 .ts 分片并合并为完整文件 |
| DASH 下载 | 解析 .mpd 清单，下载视频+音频轨并合并 |
| 分辨率选择 | 展示可用分辨率列表（360p/720p/1080p/4K），用户选择后下载 |
| 合并输出 | 使用 ffmpeg（可选依赖）将分片合并为 mp4/mkv |
| DRM 检测 | 检测到 DRM 保护时提示用户 |
| 直播录制 | 支持 HLS 直直播流录制 |

#### 2.4.17 JSON-RPC API（新增）

> 对标 aria2 JSON-RPC、qBittorrent WebAPI，解决"第三方工具集成"和"自动化"需求

| 能力 | 说明 |
|---|---|
| JSON-RPC 2.0 | 标准 JSON-RPC 协议，支持 HTTP 和 WebSocket 传输 |
| 认证 | Token 或用户名/密码认证 |
| 完整 API | 覆盖所有任务操作：添加/删除/暂停/恢复/查询状态 |
| 事件订阅 | WebSocket 连接可订阅任务事件（完成/错误/速度变化） |
| 限流 | API 请求限流，防止滥用 |
| 文档 | 内置 API 文档页面（/api/docs） |

**API 示例**：
```json
// 添加任务
{
  "jsonrpc": "2.0",
  "method": "aria2.addUri",
  "params": [["https://example.com/file.zip"], {"dir": "/downloads"}],
  "id": 1
}

// 查询状态
{
  "jsonrpc": "2.0",
  "method": "downloader.tellStatus",
  "params": ["task_id_123"],
  "id": 2
}
```

---

### 2.5 绿色便携能力

| 要求 | 实现方式 |
|---|---|
| 零安装 | 单目录解压即用，不写注册表，不修改系统环境变量 |
| 零残留 | 所有配置、日志、数据库均存储在程序目录下 data/ 子目录 |
| 单实例运行 | 通过目录锁文件（data/.lock）保证，第二次启动激活已有窗口 |
| 自动更新 | 可选功能：检查 GitHub Release 下载增量包到 update/ 目录，用户确认后替换 |
| 卸载 | 直接删除程序目录即可，无任何系统残留 |

---

### 2.6 设置中心

#### 2.6.1 常规设置
- 语言（中文 / English）
- 开机自启动（仅当用户显式启用时写入注册表，便携模式下默认关闭）
- 最小化到系统托盘
- 关闭时最小化 / 直接退出
- 主题（亮色 / 暗色 / 跟随系统）
- 字体大小
- 界面缩放

#### 2.6.2 下载设置
- 默认下载目录
- 完成目录
- 临时文件目录（默认 data/temp/）
- 最大并发任务数
- 单任务最大连接数
- 全局最大连接数
- 速度限制（上传 / 下载）
- 任务完成后动作（关机 / 休眠 / 无操作）
- 自动重试次数
- 重试间隔

#### 2.6.3 连接设置
- 监听端口（BT / ed2k / HTTP）
- UPnP / NAT-PMP 自动端口映射
- 代理设置（HTTP / HTTPS / SOCKS5）
- 代理认证（用户名/密码）
- 代理规则（全部/仅P2P/仅HTTP）
- IP 过滤（支持加载 ipfilter.dat）
- 连接超时
- 读取超时

#### 2.6.4 BT 设置
- DHT 开关
- PEX 开关
- LSD 开关
- 加密策略（禁用 / 启用 / 强制）
- 做种比率 / 时间限制
- 内置 Tracker 列表管理
- Peer 交换
- 做种优先级

#### 2.6.5 ed2k 设置
- 用户名（ed2k 客户端标识）
- 服务器列表导入（server.met）
- KAD 网络开关
- 信用系统（Credit System）
- 来源交换
- 队列大小

#### 2.6.6 HTTP 设置
- User-Agent 字符串
- Cookie 管理
- Referer 策略
- 重定向策略
- 证书验证
- 自定义请求头

#### 2.6.7 通知设置
- 任务完成通知开关
- 任务出错通知开关
- 通知声音
- 通知位置
- 通知持续时间

#### 2.6.8 高级设置
- 日志级别（DEBUG/INFO/WARN/ERROR）
- 日志文件大小限制
- 日志保留天数
- 数据库自动备份
- 临时文件清理策略
- 内存使用限制

---

## 3. 技术架构

### 3.1 整体架构图

```
┌─────────────────────────────────────────────────────────────────────────┐
│                          接入层 (Access Layer)                           │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌────────────┐  │
│  │ GUI (Tauri   │  │ WebUI        │  │ JSON-RPC API │  │ 浏览器扩展  │  │
│  │  WebView)    │  │ (axum HTTP)  │  │ (HTTP/WS)    │  │ (Native    │  │
│  │              │  │              │  │              │  │  Messaging)│  │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘  └─────┬──────┘  │
│         └──────────────────┴──────────────────┴────────────────┘         │
│                              │ IPC / HTTP / WS                           │
├──────────────────────────────┼──────────────────────────────────────────┤
│                       Rust Core 层                                       │
│  ┌──────────────────────────────────────────────────────────────────┐   │
│  │                    任务调度引擎 (Task Engine)                       │   │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐         │   │
│  │  │ BT Engine │  │ ed2k Eng │  │ HTTP Eng │  │ HLS/DASH │         │   │
│  │  └──────────┘  └──────────┘  └──────────┘  └──────────┘         │   │
│  └──────────────────────────────────────────────────────────────────┘   │
│  ┌───────────┐  ┌─────────────┐  ┌──────────────┐  ┌──────────────┐   │
│  │ 文件 I/O   │  │ 持久化存储   │  │ 网络层       │  │ 插件系统     │   │
│  │ + 自动解压  │  │ (SQLite)    │  │ (连接池/代理) │  │ (WASM 沙箱) │   │
│  └───────────┘  └─────────────┘  └──────────────┘  └──────────────┘   │
│  ┌──────────────────────────────┐  ┌──────────────────────────────┐   │
│  │ RSS 引擎                     │  │ 浏览器扩展通信层              │   │
│  │ (Feed 解析 / 规则匹配)        │  │ (Native Messaging Server)    │   │
│  └──────────────────────────────┘  └──────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────┘
```

### 3.2 技术选型

| 层级 | 技术 | 选型理由 |
|---|---|---|
| **桌面框架** | **Tauri v2** | 轻量（打包体 ~5 MB）、Rust 后端高性能、WebView 渲染前端、跨平台（Windows / macOS / Linux）、便携模式天然支持。v2 新增：Channel API 实时数据流、capabilities 细粒度权限、插件系统重构 |
| **前端 UI** | **React 18 + TypeScript + Tailwind CSS** | 生态成熟、组件丰富、配合 Recharts 绘制速度图表 |
| **前端状态** | **Zustand** | 轻量级状态管理，适合高频更新（下载速度每秒推送） |
| **前端组件** | **@tanstack/react-virtual + @tanstack/react-table + @radix-ui** | 虚拟滚动处理大量任务、表格排序筛选、无障碍对话框/右键菜单 |
| **BT 协议引擎** | **librqbit** (纯 Rust) | 纯 Rust 实现（1.7K stars），无 C++ 依赖，构建简单。支持 DHT/PEX/LSD/uTP/IPv6，已有 Tauri 桌面应用。备选：IronTide（功能更全但 RC 阶段） |
| **ed2k 协议引擎** | **自研 Rust 实现**（参考 libed2k 架构） | ed2k 生态无成熟 Rust 库，核心需自研。参考 libed2k（C++ Boost.Asio 架构），模块化实现：ed2k-hash / ed2k-proto / ed2k-server / ed2k-kad / ed2k-transfer |
| **HTTP/HTTPS 下载** | **reqwest + tokio 分块** | 异步 HTTP 客户端，支持 Range 请求断点续传、代理、Cookie |
| **持久化** | **SQLite (via rusqlite)** | 任务列表、配置、分块位图、peer 缓存等结构化数据存储 |
| **配置管理** | **TOML（程序目录下 config.toml）** | 人类可读、便于手动编辑 |
| **日志** | **tracing + tracing-subscriber** | Rust 生态标准日志框架，支持结构化日志和文件滚动输出 |
| **动画** | **framer-motion** | 轻量级 React 动画库，用于主题切换、列表项动画 |
| **图标** | **lucide-react** | 轻量、一致的图标库 |
| **Tauri 插件** | **tauri-plugin-notification / tauri-plugin-updater / tauri-plugin-fs / tauri-plugin-clipboard** | 官方插件，跨平台系统通知、自动更新、文件系统、剪贴板 |
| **Web 服务器** | **axum** | 轻量异步 HTTP 框架，用于内置 WebUI 和 JSON-RPC API 服务 |
| **WebSocket** | **tokio-tungstenite** | 异步 WebSocket，用于 WebUI 实时推送和 JSON-RPC WebSocket 传输 |
| **HLS 解析** | **hls-demux / m3u8-rs** | HLS m3u8 播放列表解析，ts 分片下载与合并 |
| **DASH 解析** | **mpd-parser (自研)** | DASH mpd 清单解析 |
| **压缩解压** | **zip / rar / 7z / tar** | 下载完成后自动解压，支持多种压缩格式 |
| **RSS 解析** | **feed-rs** | RSS/Atom feed 解析 |
| **WASM 运行时** | **wasmtime / wasmer** | 插件系统 WASM 沙箱运行时 |
| **浏览器扩展** | **Chrome Extension MV3 / Firefox WebExtension** | 浏览器下载捕获和视频嗅探扩展 |
| **FFmpeg 集成** | **ffmpeg-next (可选)** | 媒体流分片合并，可选依赖 |

### 3.3 核心模块设计

#### 3.3.1 Task Engine（任务调度引擎）

`ust
// 核心 trait\ trait DownloadEngine {
    async fn add_task(&self, params: TaskParams) -> Result<TaskId>;
    async fn pause_task(&self, id: TaskId) -> Result<()>;
    async fn resume_task(&self, id: TaskId) -> Result<()>;
    async fn remove_task(&self, id: TaskId, delete_files: bool) -> Result<()>;
    async fn get_status(&self, id: TaskId) -> Result<TaskStatus>;
    async fn get_peers(&self, id: TaskId) -> Result<Vec<PeerInfo>>;
}

// 统一任务状态
struct TaskStatus {
    id: TaskId,
    name: String,
    protocol: Protocol,        // BT | Magnet | Ed2k | Http
    state: TaskState,          // Queued | Downloading | Paused | Seeding | Done | Error
    total_size: u64,
    downloaded: u64,
    uploaded: u64,
    download_speed: u64,       // bytes/sec
    upload_speed: u64,
    progress: f32,             // 0.0 ~ 1.0
    peers: u32,
    seeds: u32,
    eta: Option<Duration>,
    save_path: PathBuf,
    files: Vec<FileInfo>,
    error: Option<String>,
}
`

**并发模型**：
- 使用 **Tokio** 异步运行时
- 每个下载任务为一个独立的 	okio::task
- 全局信号量控制并发任务数
- 任务间通过 mpsc channel 向 UI 层推送状态更新（100ms 节流）

#### 3.3.2 BT Engine

- 封装 libtorrent-rasterbar 的核心 API
- 管理 session（全局唯一）、torrent_handle（每个任务一个）
- 监听 libtorrent alert 通道，转换为统一的 TaskStatus 推送给 Task Engine
- 独立线程运行 libtorrent session（因其内部有自己的线程模型）

#### 3.3.3 ed2k Engine

模块划分：

| 模块 | 职责 |
|---|---|
| ed2k_hash | ed2k hash 计算（MD4-based ICH） |
| ed2k_server | ed2k 服务器连接、登录、搜索、队列管理 |
| ed2k_kad | Kademlia DHT 实现（UDP） |
| ed2k_source | 源交换协议（SUI / OBFU） |
| ed2k_transfer | 文件分块下载、请求队列、上传管理 |
| ed2k_credit | 信用系统（eMule 兼容） |

#### 3.3.4 持久化层

SQLite 表结构概要：

`sql
-- 任务表
CREATE TABLE tasks (
    id          TEXT PRIMARY KEY,
    name        TEXT NOT NULL,
    protocol    TEXT NOT NULL,
    magnet_uri  TEXT,
    torrent     BLOB,            -- .torrent 原始数据
    ed2k_uri    TEXT,
    save_path   TEXT NOT NULL,
    total_size  INTEGER,
    state       TEXT NOT NULL,
    added_at    DATETIME DEFAULT CURRENT_TIMESTAMP,
    completed_at DATETIME,
    metadata    TEXT              -- JSON 扩展字段
);

-- 文件表（多文件任务的子文件）
CREATE TABLE task_files (
    task_id     TEXT REFERENCES tasks(id),
    index       INTEGER,
    path        TEXT NOT NULL,
    size        INTEGER NOT NULL,
    priority    INTEGER DEFAULT 1,  -- 0=跳过 1=正常 2=高
    PRIMARY KEY (task_id, index)
);

-- 分块位图（断点续传核心）
CREATE TABLE piece_bitmaps (
    task_id     TEXT PRIMARY KEY REFERENCES tasks(id),
    bitmap      BLOB NOT NULL     -- bitfield, 每 bit 对应一个 piece
);

-- 配置表
CREATE TABLE settings (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

-- 下载历史表
CREATE TABLE download_history (
    id          TEXT PRIMARY KEY,
    task_id     TEXT,
    name        TEXT NOT NULL,
    url         TEXT,
    protocol    TEXT NOT NULL,
    save_path   TEXT NOT NULL,
    file_path   TEXT,
    total_size  INTEGER,
    downloaded  INTEGER,
    average_speed INTEGER,
    duration    INTEGER,           -- 下载耗时（秒）
    completed_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    tags        TEXT,              -- JSON 数组
    notes       TEXT
);

-- 标签表
CREATE TABLE tags (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    name        TEXT NOT NULL UNIQUE,
    color       TEXT DEFAULT '#3B82F6',
    created_at  DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- 任务标签关联表
CREATE TABLE task_tags (
    task_id     TEXT REFERENCES tasks(id),
    tag_id      INTEGER REFERENCES tags(id),
    PRIMARY KEY (task_id, tag_id)
);

-- 流量统计表
CREATE TABLE traffic_stats (
    date        DATE PRIMARY KEY,
    download    INTEGER DEFAULT 0,
    upload      INTEGER DEFAULT 0,
    tasks_completed INTEGER DEFAULT 0
);
`

#### 3.3.5 前端数据流

`
[Tauri Rust Core]
       │
       │  tauri::emit("task-update", payload)   // 每 100ms 推送
       ▼
[React Event Listener]
       │
       │  useTaskStore.getState().updateTasks(data)
       ▼
[Zustand Store]  ──→  [Task List Component] (虚拟滚动优化)
                   ──→  [Detail Panel]
                   ──→  [Speed Chart] (Recharts, 1s 数据点)
`

### 3.4 目录结构

`
downloader/
├── src-tauri/                   # Rust 后端
│   ├── Cargo.toml
│   ├── tauri.conf.json          # Tauri 配置（窗口、权限、端口等）
│   ├── src/
│   │   ├── main.rs              # 入口
│   │   ├── lib.rs               # 模块导出
│   │   ├── commands/            # Tauri IPC 命令
│   │   │   ├── task.rs          # add/pause/resume/remove 任务
│   │   │   ├── settings.rs      # 读写配置
│   │   │   └── system.rs        # 托盘、通知、更新
│   │   ├── engine/              # 下载引擎
│   │   │   ├── mod.rs
│   │   │   ├── task_manager.rs  # 任务调度器
│   │   │   ├── bt/              # BitTorrent
│   │   │   │   ├── mod.rs
│   │   │   │   ├── session.rs   # libtorrent session 封装
│   │   │   │   └── magnet.rs    # magnet 链接解析
│   │   │   ├── ed2k/            # ed2k
│   │   │   │   ├── mod.rs
│   │   │   │   ├── server.rs
│   │   │   │   ├── kad.rs
│   │   │   │   ├── transfer.rs
│   │   │   │   └── hash.rs
│   │   │   ├── http.rs          # HTTP 分块下载
│   │   │   └── hls.rs           # HLS/DASH 流媒体下载
│   │   ├── api/                 # JSON-RPC API 服务
│   │   │   ├── mod.rs
│   │   │   ├── rpc.rs           # JSON-RPC 2.0 处理
│   │   │   ├── websocket.rs     # WebSocket 实时推送
│   │   │   └── auth.rs          # API 认证
│   │   ├── webui/               # 内置 WebUI 服务
│   │   │   ├── mod.rs
│   │   │   └── static/          # WebUI 静态资源
│   │   ├── browser/             # 浏览器扩展通信
│   │   │   ├── mod.rs
│   │   │   └── native_msg.rs    # Native Messaging Server
│   │   ├── rss/                 # RSS 自动下载
│   │   │   ├── mod.rs
│   │   │   ├── feed.rs          # Feed 解析
│   │   │   └── rules.rs         # 过滤规则引擎
│   │   ├── archive/             # 自动解压
│   │   │   ├── mod.rs
│   │   │   └── password.rs      # 解压密码管理
│   │   ├── plugin/              # 插件系统
│   │   │   ├── mod.rs
│   │   │   ├── loader.rs        # WASM 插件加载器
│   │   │   └── api.rs           # 插件 API 定义
│   │   ├── storage/             # 持久化
│   │   │   ├── db.rs            # SQLite 操作
│   │   │   └── config.rs        # TOML 配置
│   │   └── util/                # 工具
│   │       ├── bandwidth.rs     # 带宽统计
│   │       ├── clipboard.rs     # 剪贴板监听
│   │       └── portable.rs      # 便携模式检测
│   └── resources/               # 内置资源
│       ├── trackers_best.txt    # 公共 tracker 列表
│       └── server.met           # ed2k 默认服务器列表
│
├── src/                         # React 前端（桌面版）
│   ├── main.tsx
│   ├── App.tsx
│   ├── components/
│   │   ├── TaskList.tsx         # 任务列表（虚拟滚动）
│   │   ├── TaskDetail.tsx       # 详情面板（Tabs）
│   │   ├── SpeedChart.tsx       # 速度图表
│   │   ├── AddTaskDialog.tsx    # 添加任务对话框
│   │   ├── SettingsDialog.tsx   # 设置中心
│   │   ├── StatusBar.tsx        # 底部状态栏
│   │   ├── TrayManager.tsx      # 托盘逻辑
│   │   ├── RssManager.tsx       # RSS 订阅管理
│   │   ├── PluginManager.tsx    # 插件管理
│   │   └── ArchiveDialog.tsx    # 解压密码管理
│   ├── stores/
│   │   ├── taskStore.ts         # Zustand store
│   │   ├── rssStore.ts          # RSS 订阅状态
│   │   └── pluginStore.ts       # 插件状态
│   ├── hooks/
│   │   ├── useTaskEvents.ts     # Tauri event 订阅
│   │   ├── useClipboard.ts      # 剪贴板 hook
│   │   └── useRss.ts            # RSS 管理 hook
│   └── lib/
│       ├── tauri-api.ts         # Tauri invoke 封装
│       └── format.ts            # 速度/大小格式化
│
├── src-webui/                   # React 前端（WebUI 版，复用组件）
│   ├── main.tsx
│   ├── App.tsx
│   └── lib/
│       ├── rpc-api.ts           # JSON-RPC API 封装
│       └── ws.ts                # WebSocket 连接管理
│
├── extension/                   # 浏览器扩展
│   ├── manifest.json            # Chrome MV3 / Firefox WebExtension
│   ├── background.js            # Service Worker
│   ├── content.js               # 内容脚本（视频嗅探）
│   ├── popup.html               # 弹出窗口
│   ├── popup.js
│   └── icons/
│
├── data/                        # 运行时数据（gitignore）
│   ├── downloader.db            # SQLite 数据库
│   ├── config.toml              # 用户配置
│   ├── logs/                    # 日志文件
│   └── temp/                    # 下载临时文件
│
├── package.json
├── tailwind.config.ts
├── tsconfig.json
└── README.md
`

### 3.5 构建与分发

| 项目 | 方案 |
|---|---|
| 构建工具 | tauri build 统一打包 |
| 输出格式 | **Windows**: 解压即用的 .zip（免安装），可选 .msi 安装包  
**macOS**: .dmg 或 .app  
**Linux**: .AppImage（便携）或 .deb |
| 包体大小 | 预估 ~10-15 MB（含 WebView2 Runtime 除外） |
| CI/CD | GitHub Actions：push tag → 构建全平台 → 发布 Release |
| 代码签名 | Windows: Authenticode；macOS: Apple Notarization（可选） |

### 3.6 配置文件规范

**config.toml 示例**

```toml
[general]
language = "zh-CN"
theme = "system"  # light | dark | system
minimize_to_tray = true
close_to_tray = false
auto_start = false
font_size = 14

[download]
default_dir = "C:/Downloads"
complete_dir = "C:/Downloads/Complete"
temp_dir = "./data/temp"
max_concurrent_tasks = 3
max_connections_per_task = 64
max_global_connections = 200
max_upload_speed = 0  # 0 = unlimited, bytes/sec
max_download_speed = 0
auto_retry_count = 3
auto_retry_interval = 5  # seconds

[connection]
bt_port = 6881
ed2k_port = 4661
http_port = 0  # 0 = disabled
upnp = true
nat_pmp = true
proxy_type = "none"  # none | http | socks5
proxy_host = ""
proxy_port = 0
proxy_username = ""
proxy_password = ""
connection_timeout = 30  # seconds
read_timeout = 60

[bt]
dht = true
pex = true
lsd = true
encryption = "enabled"  # disabled | enabled | forced
seed_ratio_limit = 2.0
seed_time_limit = 1440  # minutes, 0 = unlimited
trackers_file = "./resources/trackers_best.txt"

[ed2k]
username = "Downloader"
server_met = "./resources/server.met"
kad = true
credit_system = true

[http]
user_agent = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36"
cookie_policy = "auto"  # auto | manual | disabled
referer_policy = "strict"  # strict | unsafe | none
max_redirects = 10
verify_ssl = true

[notification]
task_complete = true
task_error = true
sound = true
position = "bottom-right"
duration = 5  # seconds

[advanced]
log_level = "info"  # debug | info | warn | error
log_max_size = 10  # MB
log_retain_days = 7
db_backup_interval = 24  # hours
temp_cleanup_interval = 1  # hours
memory_limit = 0  # MB, 0 = unlimited

[schedule]
bandwidth_schedule = false
# bandwidth_rules = [
#   { start = "00:00", end = "08:00", download = 0, upload = 0 },
#   { start = "08:00", end = "24:00", download = 10485760, upload = 5242880 }
# ]
```

### 3.7 Tauri v2 配置规范

**tauri.conf.json**

```json
{
  "productName": "Downloader",
  "version": "1.0.0",
  "identifier": "com.downloader.app",
  "build": {
    "frontendDist": "../dist",
    "devUrl": "http://localhost:5173",
    "removeUnusedCommands": true
  },
  "app": {
    "windows": [
      {
        "title": "全协议下载器",
        "width": 1200,
        "height": 800,
        "minWidth": 800,
        "minHeight": 600,
        "resizable": true,
        "fullscreen": false,
        "decorations": true,
        "transparent": false
      }
    ],
    "security": {
      "capabilities": [
        {
          "identifier": "main-capability",
          "windows": ["main"],
          "permissions": [
            "core:default",
            "fs:default",
            "notification:default",
            "clipboard:default",
            "updater:default",
            {
              "identifier": "fs:scope",
              "allow": [
                { "path": "$DOWNLOAD" },
                { "path": "$DOWNLOAD/**/*" },
                { "path": "$DOCUMENT" },
                { "path": "$DOCUMENT/**/*" },
                { "path": "$APPDATA" },
                { "path": "$APPDATA/**/*" },
                { "path": "$RESOURCE" },
                { "path": "$RESOURCE/**/*" }
              ]
            }
          ]
        }
      ]
    },
    "trayIcon": {
      "iconPath": "icons/icon.png",
      "iconAsTemplate": true
    }
  },
  "bundle": {
    "active": true,
    "targets": "all",
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/128x128@2x.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ],
    "createUpdaterArtifacts": true,
    "windows": {
      "nsis": {
        "displayLanguageSelector": true,
        "installMode": "currentUser"
      }
    }
  },
  "plugins": {
    "updater": {
      "pubkey": "",
      "endpoints": [
        "https://github.com/user/downloader/releases/latest/download/latest.json"
      ]
    }
  }
}
```

**capabilities/default.json**

```json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "main-capability",
  "description": "Main window capabilities",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "fs:allow-read-file",
    "fs:allow-write-file",
    "fs:allow-read-dir",
    "fs:allow-exists",
    "fs:allow-mkdir",
    "fs:allow-remove",
    "fs:allow-rename",
    "notification:allow-send-notification",
    "notification:allow-request-permission",
    "notification:allow-is-permission-granted",
    "clipboard:allow-read-text",
    "clipboard:allow-write-text"
  ]
}
```

### 3.8 ed2k 协议实现规范

#### 3.8.1 协议包格式

```
TCP 包头 (6 字节):
+----------+----------+----------+
| Protocol | Size     | Opcode   |
| 1 byte   | 4 bytes  | 1 byte   |
| (0xE3)   | (LE)     |          |
+----------+----------+----------+
| Payload (Size - 1 bytes)        |
+---------------------------------+

协议标识:
  0xE3 - 标准 eDonkey 协议
  0xC5 - eMule 扩展协议
  0xD4 - 压缩协议 (zlib)
  0xE4 - Kademlia DHT
  0xE5 - 压缩 Kademlia
```

#### 3.8.2 核心操作码

**服务器通信 (C2S)**:
| 操作码 | 十六进制 | 说明 |
|--------|----------|------|
| OP_LOGINREQUEST | 0x01 | 登录 |
| OP_GETSERVERLIST | 0x14 | 请求服务器列表 |
| OP_SEARCHREQUEST | 0x16 | 文件搜索 |
| OP_GETSOURCES | 0x19 | 查找文件源 |

**服务器响应 (S2C)**:
| 操作码 | 十六进制 | 说明 |
|--------|----------|------|
| OP_SERVERIDENT | 0x41 | 服务器标识 |
| OP_IDCHANGE | 0x40 | 客户端 ID 分配 |
| OP_SEARCHRESULT | 0x33 | 搜索结果 |
| OP_FOUNDSOURCES | 0x42 | 找到的源 |

**客户端通信 (C2C)**:
| 操作码 | 十六进制 | 说明 |
|--------|----------|------|
| OP_HELLO | 0x01 | 客户端问候 |
| OP_REQUESTFILENAME | 0x58 | 文件名请求 |
| OP_SETREQFILEID | 0x4F | 设置请求文件 |
| OP_HASHSETREQUEST | 0x51 | 请求 hashset |
| OP_STARTUPLOADREQ | 0x54 | 上传请求 |
| OP_REQUESTPARTS | 0x47 | 请求分片 (3个偏移) |
| OP_SENDINGPART | 0x46 | 发送数据分片 |

#### 3.8.3 ed2k Hash 算法

```rust
const ED2K_CHUNK_SIZE: usize = 9_728_000; // 9.28 MB

fn ed2k_hash(data: &[u8]) -> [u8; 16] {
    if data.len() <= ED2K_CHUNK_SIZE {
        // 单块文件：直接 MD4
        return md4(data);
    }
    // 多块文件：先计算每个块的 MD4，再对拼接结果 MD4
    let chunk_hashes: Vec<u8> = data.chunks(ED2K_CHUNK_SIZE)
        .flat_map(|chunk| md4(chunk))
        .collect();
    md4(&chunk_hashes)
}
```

#### 3.8.4 实现模块划分

```
src-tauri/src/engine/ed2k/
├── mod.rs              # 模块导出
├── hash.rs             # ed2k hash 计算 (MD4-based)
├── proto.rs            # 协议定义 (操作码、包结构、序列化)
├── server.rs           # 服务器通信 (TCP)
├── kad.rs              # Kademlia DHT (UDP)
├── transfer.rs         # 文件传输 (分片下载/上传)
├── source.rs           # 源交换协议
├── credit.rs           # 信用系统
├── tag.rs              # Tag 系统 (元数据编码)
└── obfuscation.rs      # 协议混淆 (RC4)
```

### 3.9 UI 设计规范

#### 3.9.1 主题系统 (CSS 变量)

```css
:root, [data-theme="light"] {
  --bg-primary: #ffffff;
  --bg-secondary: #f8fafc;
  --bg-tertiary: #f1f5f9;
  --text-primary: #0f172a;
  --text-secondary: #475569;
  --text-muted: #94a3b8;
  --border: #e2e8f0;
  --accent: #3b82f6;
  --accent-hover: #2563eb;
  --success: #22c55e;
  --warning: #f59e0b;
  --error: #ef4444;
  --progress-bg: #e2e8f0;
  --progress-fill: #3b82f6;
}

[data-theme="dark"] {
  --bg-primary: #0f172a;
  --bg-secondary: #1e293b;
  --bg-tertiary: #334155;
  --text-primary: #f1f5f9;
  --text-secondary: #94a3b8;
  --text-muted: #64748b;
  --border: #334155;
  --accent: #60a5fa;
  --accent-hover: #93bbfd;
  --success: #4ade80;
  --warning: #fbbf24;
  --error: #f87171;
  --progress-bg: #334155;
  --progress-fill: #60a5fa;
}
```

#### 3.9.2 Tailwind CSS 配置

```typescript
// tailwind.config.ts
export default {
  darkMode: ['selector', '[data-theme="dark"]'],
  theme: {
    extend: {
      colors: {
        primary: 'var(--bg-primary)',
        secondary: 'var(--bg-secondary)',
        tertiary: 'var(--bg-tertiary)',
        'text-primary': 'var(--text-primary)',
        'text-secondary': 'var(--text-secondary)',
        'text-muted': 'var(--text-muted)',
        border: 'var(--border)',
        accent: 'var(--accent)',
        success: 'var(--success)',
        warning: 'var(--warning)',
        error: 'var(--error)',
      }
    }
  }
}
```

#### 3.9.3 设计 Token

```
圆角:
  - 按钮/输入框: 6px
  - 卡片/对话框: 8-12px
  - 进度条: 4px

间距:
  - 列表行高: 36-40px (紧凑) / 48px (舒适)
  - 列表行内边距: 8px 12px
  - 对话框内边距: 24px
  - 表单项间距: 16px

字体:
  - 正文: 14px (可配置 12-18px)
  - 标题: 16-18px
  - 状态栏: 12px
  - 等宽数字: 用于速度/大小显示

动画:
  - 进度条更新: 300ms ease-out
  - 状态切换: 200ms ease
  - 对话框弹出: 200ms scale + fade
  - 主题切换: 200ms background-color/color/border-color
```

---

## 4. 非功能需求

### 4.1 性能

| 指标 | 目标 |
|---|---|
| 冷启动时间 | < 1.5 秒（SSD） |
| 内存占用（空闲） | < 50 MB |
| 内存占用（满载 20 任务） | < 300 MB |
| UI 帧率 | 60 fps（列表滚动、图表刷新不卡顿） |
| 速度上报延迟 | < 200ms（从底层采样到 UI 刷新） |

### 4.2 可靠性

- 程序异常退出后重启自动恢复所有任务（通过 SQLite 持久化）
- 磁盘满时自动暂停任务并提示用户
- 校验失败自动重下对应 piece / chunk
- Crash 自动收集 minidump，用户可选择上传

### 4.3 安全

- 不收集任何用户数据（无遥测，除非用户主动开启）
- 支持 SOCKS5 代理，保护用户隐私
- ed2k 连接支持混淆（Protocol Obfuscation）
- 内置 IP 过滤功能

### 4.4 兼容性

| 平台 | 最低版本 |
|---|---|
| Windows | Windows 10 (1809+) |
| macOS | macOS 12 Monterey |
| Linux | Ubuntu 20.04+ / 等效 |

### 4.5 错误处理

| 错误类型 | 处理策略 |
|---|---|
| 网络连接失败 | 自动重试（可配置次数），指数退避策略 |
| 磁盘空间不足 | 自动暂停任务，显示警告通知，用户清理后可恢复 |
| 文件校验失败 | 自动重新下载失败的 piece/chunk |
| 协议错误 | 记录详细日志，显示用户友好的错误信息 |
| 代理连接失败 | 自动回退直连（可配置），显示代理状态 |
| 任务损坏 | 自动检测并提示用户重新下载或校验 |
| 数据库错误 | 自动备份损坏数据库，创建新数据库 |
| 配置文件损坏 | 使用默认配置，提示用户重新设置 |

### 4.6 测试策略

| 测试类型 | 覆盖范围 | 工具 |
|---|---|---|
| 单元测试 | 核心模块：任务管理、协议解析、文件校验 | Rust: cargo test; JS: vitest |
| 集成测试 | 引擎与存储层交互、Tauri IPC 通信 | Tauri test utilities |
| E2E 测试 | 用户完整流程：添加任务→下载→完成 | Playwright (via Tauri) |
| 性能测试 | 大量任务并发、大文件下载、内存泄漏 | 自定义基准测试 |
| 跨平台测试 | Windows/macOS/Linux 三平台兼容性 | GitHub Actions matrix |
| 回归测试 | 每次发布前全量测试套件 | CI/CD 自动触发 |

### 4.7 日志与监控

| 能力 | 说明 |
|---|---|
| 结构化日志 | 使用 tracing 框架，支持 JSON/文本格式输出 |
| 日志级别 | DEBUG/INFO/WARN/ERROR，可在设置中调整 |
| 日志轮转 | 按大小（默认 10MB）或天数（默认 7 天）自动轮转 |
| 错误上报 | 可选的匿名错误上报（默认关闭），用户可手动上传日志 |
| 性能指标 | 记录下载速度、连接数、内存使用等关键指标 |

---

## 5. 里程碑计划

| 阶段 | 周期 | 交付内容 |
|---|---|---|
| **M1 - 基础框架** | 4 周 | Tauri 项目搭建、UI 骨架、任务管理 CRUD、SQLite 存储、HTTP/HTTPS/FTP 下载引擎 |
| **M2 - BT 引擎** | 3 周 | libtorrent 集成、.torrent 解析、磁力链元数据获取、DHT |
| **M3 - ed2k 引擎** | 4 周 | ed2k 服务器通信、KAD、文件下载、源交换 |
| **M4 - 完善体验** | 3 周 | 速度图表、通知、设置中心、暗色主题、批量导入、定时下载、任务搜索过滤 |
| **M5 - 扩展能力** | 4 周 | 自动解压、RSS 自动下载、JSON-RPC API、WebUI、浏览器扩展、HLS/DASH 流媒体下载 |
| **M6 - 优化发布** | 2 周 | 性能优化、跨平台测试、打包分发、CI/CD、用户文档 |

---

## 6. 风险与应对

| 风险 | 影响 | 应对策略 |
|---|---|---|
| BT 引擎选型 | 功能完整性 | **首选 librqbit**（纯 Rust，1.7K stars，活跃维护）；备选 IronTide（RC 阶段，功能更全）；最后备选 PeterDing/libtorrent-rasterbar-rs（需 Boost+C++ 编译） |
| ed2k 协议自研工作量大 | M3 周期延长 | 参考 libed2k（C++ Boost.Asio 架构）模块化实现；优先实现核心下载功能，KAD/信用系统后续迭代；复用 md4 crate 计算 hash |
| WebView 兼容性 | 不同系统渲染差异 | Windows 自带 WebView2；macOS/Linux 使用系统 WebView；UI 回归测试覆盖三平台 |
| 做种/分享合法性 | 法律风险 | 产品定位为通用下载工具，不内置任何内容索引；用户协议明确使用责任 |
| ed2k 协议混淆实现 | 兼容性问题 | 实现 RC4 加密，支持三种模式（禁用/启用/强制），参考 eMule CryptoRC4.cpp |
| 大文件内存占用 | 性能问题 | 流式处理，避免一次性加载整个文件；分片校验而非全文件校验 |
| 浏览器扩展兼容性 | 不同浏览器 API 差异 | Chrome MV3 + Firefox WebExtension 双版本维护；使用 WebExtension Polyfill 兼容层 |
| HLS/DASH 合并依赖 ffmpeg | 用户需额外安装 | 内置轻量 ts→mp4 合并（不依赖 ffmpeg）；ffmpeg 仅作为高级可选依赖 |
| WASM 插件沙箱性能 | 插件执行效率 | 限制插件权限和执行时间；轻量操作直接 Rust 实现，仅复杂逻辑走 WASM |
| WebUI 安全 | 远程访问风险 | 默认仅监听 localhost；启用远程访问时强制认证；支持 HTTPS |

---

## 6.5 架构优化建议（v4.0 新增）

> 基于竞品调研（IDM/FDM/JDownloader/aria2/qBittorrent），以下为架构层面的优化建议

### 6.5.1 引入接入层抽象（多入口架构）

**现状**：原架构仅考虑 Tauri IPC 单一入口
**优化**：引入统一的接入层，支持多种客户端接入

```
┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐
│  Tauri GUI  │  │    WebUI    │  │ JSON-RPC    │  │ 浏览器扩展   │
│  (IPC)      │  │  (HTTP)     │  │ (HTTP/WS)   │  │ (Native Msg)│
└──────┬──────┘  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘
       │                │                │                │
       └────────────────┴────────────────┴────────────────┘
                              │
                    ┌─────────▼─────────┐
                    │   统一 API 层      │
                    │   (Service Layer)  │
                    └─────────┬─────────┘
                              │
                    ┌─────────▼─────────┐
                    │   Task Engine      │
                    └───────────────────┘
```

**收益**：
- WebUI、JSON-RPC API、浏览器扩展共享同一套业务逻辑
- 新增接入方式只需实现协议适配，无需修改核心引擎
- 便于未来扩展（CLI 工具、移动端 App 等）

### 6.5.2 引擎插件化

**现状**：BT/ed2k/HTTP 引擎硬编码在 Task Engine 中
**优化**：引擎层采用 trait + 注册制，支持动态加载

```rust
// 引擎注册表
struct EngineRegistry {
    engines: HashMap<Protocol, Box<dyn DownloadEngine>>,
}

impl EngineRegistry {
    fn register(&mut self, protocol: Protocol, engine: Box<dyn DownloadEngine>) { ... }
    fn get(&self, protocol: Protocol) -> Option<&dyn DownloadEngine> { ... }
}
```

**收益**：
- 新协议（如 FTPS、SFTP）可通过插件添加，无需修改核心代码
- 引擎可独立测试和迭代
- 社区可通过插件系统贡献新协议支持

### 6.5.3 事件驱动架构优化

**现状**：100ms 固定节流推送
**优化**：基于事件总线的发布-订阅模式

```rust
enum DownloadEvent {
    Progress { task_id: TaskId, downloaded: u64, speed: u64 },
    StateChanged { task_id: TaskId, state: TaskState },
    Error { task_id: TaskId, error: String },
    Completed { task_id: TaskId },
    PeerConnected { task_id: TaskId, peer: PeerInfo },
}
```

**收益**：
- 不同消费者（GUI、WebUI、API、日志）可独立订阅感兴趣的消息
- 推送频率可按消费者需求独立配置
- 便于添加新的事件类型（如 RSS 匹配、插件触发）

### 6.5.4 存储层优化

**现状**：SQLite 单一存储
**优化**：分离元数据存储与下载状态存储

| 存储 | 内容 | 技术 |
|---|---|---|
| 元数据库 | 任务配置、历史记录、标签 | SQLite（持久化） |
| 状态缓存 | 实时速度、peer 列表、piece bitmap | 内存 + 定期快照到磁盘 |
| 配置文件 | 用户设置 | TOML（已存在） |

**收益**：
- 高频更新的状态数据不写 SQLite，减少 I/O 压力
- 元数据库查询不受状态更新影响
- 状态缓存崩溃后可从 piece bitmap 重建

### 6.5.5 WebUI 架构复用

**现状**：无 WebUI
**优化**：桌面端前端组件复用于 WebUI

```
src/
├── components/          # 共享 UI 组件
│   ├── TaskList.tsx     # 桌面 + WebUI 复用
│   ├── TaskDetail.tsx
│   └── ...
├── desktop/             # 桌面特有逻辑
│   └── tauri-api.ts
├── webui/               # WebUI 特有逻辑
│   └── rpc-api.ts
└── shared/              # 共享逻辑
    ├── stores/
    ├── hooks/
    └── lib/
```

**收益**：
- 一套组件代码，两种部署形态
- 减少维护成本
- 统一的用户体验

---

## 7. 附录

### 7.1 相关协议文档

- [BEP-3: BitTorrent Protocol](https://www.bittorrent.org/beps/bep_0003.html)
- [BEP-5: DHT Protocol](https://www.bittorrent.org/beps/bep_0005.html)
- [BEP-9: Extension for Peers to Send Metadata Files](https://www.bittorrent.org/beps/bep_0009.html)
- [ed2k Protocol Specification](http://www.amule.org/wiki/index.php/Ed2k-protocol)
- [Kademlia: A Peer-to-Peer Information System Based on the XOR Metric](https://pdos.csail.mit.edu/~petar/papers/maymounkov-kademlia-lncs.pdf)

### 7.2 竞品参考

| 竞品 | 优势 | 不足 | 本产品差异 |
|---|---|---|---|
| qBittorrent | 开源、BT 功能完善、WebUI、RSS 自动下载、搜索引擎插件 | 仅 BT，不支持 ed2k；C++/Qt 体大 | 全协议 + 轻量 Tauri + 插件系统 |
| eMule | ed2k 经典客户端 | 界面老旧、仅 Windows、已停止维护 | 现代 UI + 跨平台 |
| Motrix | aria2 引擎 + BT，UI 美观 | BT 实现弱、不支持 ed2k、无浏览器集成 | 完整 BT + ed2k + 便携 + 浏览器集成 |
| IDM | HTTP 下载强（32 线程）、浏览器集成、视频嗅探、动态分段加速 | 无 P2P 协议支持、仅 Windows、付费 | P2P 全覆盖 + 免费 + 跨平台 |
| FDM | 全协议支持、免费、远程控制、ZIP 预览 | 广告多、体大、非便携 | 零广告 + 便携 + 轻量 |
| JDownloader 2 | 插件生态丰富、CAPTCHA 自动求解、DLC/CCF 容器支持、链接抓取器、自动解压、远程管理 | Java 体大、内存占用高、广告 | 轻量 Rust + 零广告 + 插件系统 |
| aria2 | JSON-RPC API、WebSocket 实时推送、多协议、守护进程模式、极轻量 | 无 GUI、配置复杂、BT 功能弱 | 内置 GUI + 全协议 + 易用 |

#### 7.2.1 竞品功能矩阵对比

| 功能 | IDM | FDM | JDownloader | Motrix | aria2 | qBittorrent | **本产品** |
|---|---|---|---|---|---|---|---|
| HTTP 多线程 | ✅ 32线程 | ✅ | ✅ | ✅ 64连接 | ✅ | ❌ | ✅ |
| BT/磁力链 | ❌ | ✅ | ❌ | ✅ | ✅ | ✅ | ✅ |
| ed2k | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ |
| 浏览器集成 | ✅ | ❌ | ✅ | ❌ | ❌ | ❌ | ✅ **新增** |
| 视频嗅探 | ✅ | ❌ | ✅ | ❌ | ❌ | ❌ | ✅ **新增** |
| 远程管理 WebUI | ❌ | ✅ | ✅ | ❌ | ✅ | ✅ | ✅ **新增** |
| RSS 自动下载 | ❌ | ❌ | ✅ | ❌ | ❌ | ✅ | ✅ **新增** |
| 自动解压 | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ✅ **新增** |
| 插件系统 | ❌ | ❌ | ✅ | ❌ | ❌ | ✅ | ✅ **新增** |
| JSON-RPC API | ❌ | ❌ | ❌ | ✅ | ✅ | ✅ | ✅ **新增** |
| 媒体流下载 | ✅ | ❌ | ✅ | ❌ | ❌ | ❌ | ✅ **新增** |
| 便携模式 | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ✅ |
| 零广告 | ❌ | ❌ | ❌ | ✅ | ✅ | ✅ | ✅ |
| 跨平台 | ❌ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| CAPTCHA 求解 | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | 🔜 v2 |

### 7.3 术语表

| 术语 | 说明 |
|---|---|
| BT | BitTorrent，点对点文件分享协议 |
| DHT | Distributed Hash Table，分布式哈希表，用于无 Tracker 的 BT 下载 |
| PEX | Peer Exchange，Peer 交换协议，用于发现更多下载源 |
| LSD | Local Service Discovery，局域网服务发现 |
| MSE/PE | Message Stream Encryption / Protocol Encryption，BT 加密协议 |
| ed2k | eDonkey2000，另一种 P2P 文件分享协议 |
| KAD | Kademlia，ed2k 的分布式网络实现 |
| magnet | 磁力链接，基于内容哈希的下载链接 |
| piece | BT 下载的最小校验单位 |
| chunk | ed2k 下载的最小校验单位（9.28 MB） |
| seeder | 做种者，拥有完整文件的下载者 |
| leecher | 下载者，正在下载文件的用户 |
| tracker | BT 服务器，用于协调下载者之间的连接 |
| info_hash | BT 种子的唯一标识符 |
| SHA-1 | 安全哈希算法，用于 BT 文件校验 |
| MD4 | 消息摘要算法，用于 ed2k 文件校验 |

### 7.4 参考资源

**开源项目**
- [librqbit](https://github.com/ikatson/rqbit) - 纯 Rust BT 客户端（推荐）
- [IronTide](https://codeberg.org/alan090/irontide) - 纯 Rust BT 引擎（备选）
- [libtorrent-rasterbar](https://github.com/arvidn/libtorrent) - C++ BT 库（参考）
- [libed2k](https://github.com/qmule/libed2k) - ed2k C++ 库（参考架构）
- [aMule](https://github.com/amule-project/amule) - ed2k 开源客户端
- [eMule](https://sourceforge.net/projects/emule/) - ed2k 协议参考实现
- [Tauri](https://tauri.app/) - 跨平台桌面应用框架

**技术文档**
- [Tauri v2 官方文档](https://tauri.app/v2/guides/)
- [React 文档](https://react.dev/)
- [Tailwind CSS 文档](https://tailwindcss.com/)
- [Zustand 文档](https://github.com/pmndrs/zustand)
- [SQLite 文档](https://www.sqlite.org/docs.html)
- [librqbit API](https://docs.rs/librqbit) - BT 引擎 Rust API

**协议规范**
- [BitTorrent 协议规范](https://www.bittorrent.org/beps/bep_0003.html)
- [ed2k 协议规范](http://www.amule.org/wiki/index.php/Ed2k-protocol)
- [Kademlia 论文](https://pdos.csail.mit.edu/~petar/papers/maymounkov-kademlia-lncs.pdf)
- [libed2k packet_struct.hpp](https://github.com/qmule/libed2k/blob/master/include/libed2k/packet_struct.hpp) - ed2k 操作码定义

---

## 8. 实现步骤（详细任务清单）

### 8.1 M1 - 基础框架（4 周）

#### 第 1 周：项目初始化与环境搭建

**Day 1-2: Tauri v2 项目创建**
- [ ] `npm create tauri-app@latest downloader -- --template react-ts`
- [ ] 配置 `tauri.conf.json`（窗口、权限、标识符）
- [ ] 配置 `capabilities/default.json`（文件系统、通知、剪贴板权限）
- [ ] 安装 Tauri 插件：`tauri-plugin-notification`、`tauri-plugin-fs`、`tauri-plugin-clipboard`
- [ ] 配置 Tailwind CSS + dark mode
- [ ] 配置 Zustand、React Router
- [ ] 创建基础目录结构（见 3.4 节）

**Day 3-4: Rust 后端基础架构**
- [ ] 创建 `src-tauri/src/engine/mod.rs` - 引擎模块入口
- [ ] 创建 `src-tauri/src/storage/db.rs` - SQLite 初始化
- [ ] 创建 `src-tauri/src/storage/config.rs` - TOML 配置读写
- [ ] 实现数据库迁移脚本（tasks、task_files、piece_bitmaps、settings 表）
- [ ] 创建 `src-tauri/src/commands/mod.rs` - Tauri IPC 命令入口

**Day 5: 前端基础架构**
- [ ] 创建 `src/stores/taskStore.ts` - Zustand 任务状态管理
- [ ] 创建 `src/lib/tauri-api.ts` - Tauri invoke 封装
- [ ] 创建 `src/lib/format.ts` - 速度/大小格式化工具
- [ ] 创建 `src/hooks/useTaskEvents.ts` - Tauri event 订阅 hook

#### 第 2 周：任务管理 CRUD

**Day 1-2: Rust 端任务管理**
- [ ] 实现 `src-tauri/src/commands/task.rs`:
  - `add_task(params: TaskParams) -> TaskId`
  - `pause_task(id: TaskId)`
  - `resume_task(id: TaskId)`
  - `remove_task(id: TaskId, delete_files: bool)`
  - `get_task(id: TaskId) -> TaskStatus`
  - `get_all_tasks() -> Vec<TaskStatus>`
- [ ] 实现 `src-tauri/src/engine/task_manager.rs` - 任务调度器
- [ ] 实现任务状态机：Queued → Downloading → Paused → Done → Error

**Day 3-4: 前端任务列表**
- [ ] 创建 `src/components/TaskList.tsx`:
  - 使用 `@tanstack/react-virtual` 虚拟滚动
  - 使用 `@tanstack/react-table` 表格排序
  - 列定义：状态、文件名、协议、大小、进度、速度、来源、ETA
  - 进度条组件（分段式）
  - 状态图标组件
- [ ] 创建 `src/components/StatusBar.tsx` - 全局状态栏
- [ ] 实现任务列表与 Tauri 后端的数据绑定

**Day 5: 任务操作 UI**
- [ ] 创建 `src/components/AddTaskDialog.tsx` - 添加任务对话框
- [ ] 实现右键菜单（开始/暂停/删除/属性/打开目录）
- [ ] 实现拖拽排序（`@dnd-kit/sortable`）
- [ ] 实现批量选择（Ctrl+Click、Shift+Click、Ctrl+A）

#### 第 3 周：HTTP/HTTPS/FTP 下载引擎

**Day 1-3: HTTP 下载实现**
- [ ] 创建 `src-tauri/src/engine/http.rs`:
  - 实现 `reqwest` 异步 HTTP 客户端
  - 实现 Range 请求断点续传
  - 实现多线程分块下载（4-16 线程）
  - 实现自动重定向跟随（301/302/303/307/308）
  - 实现文件名提取（Content-Disposition / URL 路径）
  - 实现 Cookie 处理
  - 实现代理支持（HTTP/SOCKS5）
- [ ] 实现下载进度回调（每 100ms 推送到前端）
- [ ] 实现速度限制（令牌桶算法）

**Day 4-5: FTP 下载实现**
- [ ] 实现 FTP 被动模式（PASV）
- [ ] 实现 FTP 目录列表（LIST/NLST）
- [ ] 实现 FTP 断点续传（REST 命令）
- [ ] 集成到 HTTP 引擎统一管理

#### 第 4 周：持久化与断点续传

**Day 1-2: 断点续传实现**
- [ ] 实现 `piece_bitmaps` 表的读写
- [ ] 实现 HTTP 下载的字节偏移记录
- [ ] 实现程序重启后自动恢复未完成任务
- [ ] 实现文件校验（MD5/SHA1/SHA256）

**Day 3-4: 设置系统**
- [ ] 创建 `src/components/SettingsDialog.tsx`:
  - 左侧分类导航 + 右侧表单
  - 常规设置、下载设置、连接设置、通知设置、高级设置
- [ ] 实现 `src-tauri/src/commands/settings.rs` - 配置读写
- [ ] 实现配置文件热更新

**Day 5: 集成测试与修复**
- [ ] 测试 HTTP/HTTPS 下载（小文件、大文件、断点续传）
- [ ] 测试 FTP 下载
- [ ] 测试任务 CRUD 操作
- [ ] 测试设置保存/加载
- [ ] 修复发现的 Bug

---

### 8.2 M2 - BT 引擎（3 周）

#### 第 5 周：librqbit 集成

**Day 1-2: 依赖配置**
- [ ] 在 `Cargo.toml` 添加 `librqbit` 依赖
- [ ] 配置 librqbit feature flags（dht、upnp、peer-info 等）
- [ ] 创建 `src-tauri/src/engine/bt/mod.rs` - BT 引擎入口
- [ ] 创建 `src-tauri/src/engine/bt/session.rs` - librqbit session 封装

**Day 3-4: 核心 API 封装**
- [ ] 实现 `BtEngine` trait:
  - `add_torrent(torrent_path: &str, save_path: &str) -> TaskId`
  - `add_magnet(magnet_uri: &str, save_path: &str) -> TaskId`
  - `pause_task(id: TaskId)`
  - `resume_task(id: TaskId)`
  - `remove_task(id: TaskId, delete_files: bool)`
  - `get_status(id: TaskId) -> TaskStatus`
  - `get_peers(id: TaskId) -> Vec<PeerInfo>`
- [ ] 实现 librqbit 状态到 TaskStatus 的转换
- [ ] 实现进度回调（100ms 节流推送）

**Day 5: .torrent 文件解析**
- [ ] 实现 .torrent 文件拖拽添加
- [ ] 实现文件树展示（多文件 torrent）
- [ ] 实现选择性下载（勾选/取消子文件）
- [ ] 实现 magnet 链接解析和元数据获取

#### 第 6 周：BT 功能完善

**Day 1-2: DHT/PEX/LSD**
- [ ] 验证 DHT 网络连接
- [ ] 验证 PEX（Peer Exchange）
- [ ] 验证 LSD（Local Service Discovery）
- [ ] 实现 DHT 节点数显示
- [ ] 实现 Tracker 管理（添加/删除/状态显示）

**Day 3-4: 做种与限速**
- [ ] 实现做种比率/时间限制
- [ ] 实现单任务限速
- [ ] 实现全局限速
- [ ] 实现加密策略（禁用/启用/强制）

**Day 5: Peer 管理**
- [ ] 实现 Peer 列表显示（IP、客户端、速度、已交换数据）
- [ ] 实现单个 Peer 封禁
- [ ] 实现 Peer 来源标识（DHT/PEX/Tracker/Incoming）

#### 第 7 周：BT 测试与优化

**Day 1-2: 功能测试**
- [ ] 测试 .torrent 文件下载
- [ ] 测试 magnet 链接下载
- [ ] 测试多文件 torrent 选择性下载
- [ ] 测试做种功能
- [ ] 测试断点续传

**Day 3-4: 性能优化**
- [ ] 优化内存占用（大量任务场景）
- [ ] 优化 UI 响应速度（虚拟滚动）
- [ ] 优化速度计算准确性
- [ ] 优化日志输出

**Day 5: Bug 修复与文档**
- [ ] 修复测试发现的 Bug
- [ ] 编写 BT 引擎使用文档
- [ ] 编写 BT 相关配置说明

---

### 8.3 M3 - ed2k 引擎（4 周）

#### 第 8 周：ed2k 基础模块

**Day 1-2: ed2k hash 实现**
- [ ] 创建 `src-tauri/src/engine/ed2k/hash.rs`
- [ ] 实现 MD4 hash 计算（使用 `md4` crate）
- [ ] 实现 ed2k 分块 hash（9.28 MB 分块）
- [ ] 实现 AICH hash（180 KB 子块）
- [ ] 编写 hash 计算单元测试

**Day 3-4: 协议定义**
- [ ] 创建 `src-tauri/src/engine/ed2k/proto.rs`
- [ ] 定义协议标识（0xE3、0xC5、0xD4、0xE4、0xE5）
- [ ] 定义 TCP 包头结构（6 字节）
- [ ] 定义服务器通信操作码
- [ ] 定义客户端通信操作码
- [ ] 实现包序列化/反序列化

**Day 5: Tag 系统**
- [ ] 创建 `src-tauri/src/engine/ed2k/tag.rs`
- [ ] 实现 Tag 类型定义（Hash、String、Integer、Float、Bool）
- [ ] 实现 Tag 序列化/反序列化
- [ ] 实现常见文件 Tag（FT_FILENAME、FT_FILESIZE 等）

#### 第 9 周：ed2k 服务器通信

**Day 1-2: 服务器连接**
- [ ] 创建 `src-tauri/src/engine/ed2k/server.rs`
- [ ] 实现 TCP 连接管理
- [ ] 实现登录流程（OP_LOGINREQUEST → OP_SERVERIDENT → OP_IDCHANGE）
- [ ] 实现 HighID/LowID 识别
- [ ] 实现服务器列表管理（server.met 解析）

**Day 3-4: 搜索与源查找**
- [ ] 实现文件搜索（OP_SEARCHREQUEST → OP_SEARCHRESULT）
- [ ] 实现源查找（OP_GETSOURCES → OP_FOUNDSOURCES）
- [ ] 实现 UDP 查询（OP_GLOBSEARCHREQ3、OP_GLOBGETSOURCES2）
- [ ] 实现搜索结果解析

**Day 5: 文件传输**
- [ ] 创建 `src-tauri/src/engine/ed2k/transfer.rs`
- [ ] 实现客户端问候（OP_HELLO → OP_HELLOANSWER）
- [ ] 实现文件请求流程
- [ ] 实现分片下载（OP_REQUESTPARTS → OP_SENDINGPART）
- [ ] 实现队列管理（OP_QUEUERANK）

#### 第 10 周：ed2k KAD 网络

**Day 1-2: KAD 基础**
- [ ] 创建 `src-tauri/src/engine/ed2k/kad.rs`
- [ ] 实现 128 位 ID 空间
- [ ] 实现 k-bucket 路由表（k=10）
- [ ] 实现 XOR 距离度量
- [ ] 实现引导流程（KADEMLIA2_BOOTSTRAP_REQ/RES）

**Day 3-4: KAD 操作**
- [ ] 实现迭代式节点查找（α=3 并发）
- [ ] 实现关键词搜索（KADEMLIA2_SEARCH_KEY_REQ）
- [ ] 实现源搜索（KADEMLIA2_SEARCH_SOURCE_REQ）
- [ ] 实现发布操作（KADEMLIA2_PUBLISH_KEY_REQ）

**Day 5: KAD 集成**
- [ ] 实现 KAD 网络状态显示
- [ ] 实现 KAD 节点数统计
- [ ] 实现防火墙检测（KADEMLIA_FIREWALLED2_REQ）
- [ ] 集成到 ed2k 引擎

#### 第 11 周：ed2k 完善与测试

**Day 1-2: 源交换与信用系统**
- [ ] 创建 `src-tauri/src/engine/ed2k/source.rs`
- [ ] 实现源交换协议（OP_REQUESTSOURCES → OP_ANSWERSOURCES）
- [ ] 创建 `src-tauri/src/engine/ed2k/credit.rs`
- [ ] 实现基础信用系统（上传/下载比率）

**Day 3: 协议混淆**
- [ ] 创建 `src-tauri/src/engine/ed2k/obfuscation.rs`
- [ ] 实现 Diffie-Hellman 密钥交换
- [ ] 实现 RC4 流加密
- [ ] 支持三种模式（禁用/启用/强制）

**Day 4-5: 集成测试**
- [ ] 测试 ed2k 链接下载
- [ ] 测试服务器连接
- [ ] 测试 KAD 网络
- [ ] 测试源交换
- [ ] 修复发现的 Bug

---

### 8.4 M4 - 完善体验（3 周）

#### 第 12 周：UI 完善

**Day 1-2: 详情面板**
- [ ] 创建 `src/components/TaskDetail.tsx`:
  - 概要 Tab（任务信息、Hash、时间）
  - 文件 Tab（文件树、优先级）
  - 连接 Tab（Peer 列表、封禁）
  - Tracker Tab（状态、手动添加）
  - 日志 Tab（事件日志）
- [ ] 实现详情面板展开/收起
- [ ] 实现详情面板高度拖拽调整

**Day 3-4: 速度图表**
- [ ] 创建 `src/components/SpeedChart.tsx`:
  - 使用 Recharts 绘制
  - 双 Y 轴（下载/上传速度）
  - 时间轴切换（5分钟/30分钟/1小时）
  - 鼠标悬停显示数值
- [ ] 实现速度数据存储（环形缓冲区）
- [ ] 实现图表主题适配

**Day 5: 通知系统**
- [ ] 实现系统通知（tauri-plugin-notification）
- [ ] 实现任务完成通知
- [ ] 实现出错通知
- [ ] 实现通知设置

#### 第 13 周：批量与定时

**Day 1-2: 批量导入**
- [ ] 创建 `src/components/BatchImportDialog.tsx`:
  - 文本文件导入
  - 剪贴板导入
  - 通配符生成
  - 正则替换
- [ ] 实现去重检测
- [ ] 实现链接有效性预检
- [ ] 实现任务分组

**Day 3-4: 定时下载**
- [ ] 实现定时开始/暂停
- [ ] 实现带宽计划（cron 表达式）
- [ ] 实现做种计划
- [ ] 实现下载完成后动作（关机/休眠/锁屏/执行命令）

**Day 5: 搜索与过滤**
- [ ] 实现任务搜索（关键词、状态、协议、大小、时间）
- [ ] 实现排序（名称、大小、进度、速度、ETA、添加时间）
- [ ] 实现标签系统
- [ ] 实现侧边栏分类（全部/下载中/已完成/做种/错误/标签）

#### 第 14 周：主题与历史

**Day 1-2: 主题系统**
- [ ] 实现 CSS 变量主题系统
- [ ] 实现亮色/暗色/跟随系统切换
- [ ] 实现主题切换动画
- [ ] 实现自定义强调色
- [ ] 实现字体大小设置

**Day 3-4: 下载历史**
- [ ] 创建 `src/components/DownloadHistory.tsx`:
  - 历史列表
  - 搜索过滤
  - 统计信息
  - 流量图表
- [ ] 实现历史记录导出（CSV/JSON）
- [ ] 实现历史记录清理

**Day 5: 工具栏与托盘**
- [ ] 实现工具栏自定义
- [ ] 实现系统托盘功能
- [ ] 实现托盘右键菜单
- [ ] 实现托盘图标状态显示

---

### 8.5 M5 - 扩展能力（4 周，v4.0 新增）

#### 第 15 周：自动解压 + RSS 自动下载

**Day 1-2: 自动解压功能**
- [ ] 创建 `src-tauri/src/archive/mod.rs` - 解压引擎
- [ ] 创建 `src-tauri/src/archive/password.rs` - 密码管理
- [ ] 支持 .zip/.rar/.7z/.tar.gz 解压
- [ ] 实现解压密码记忆（按域名/路径关联）
- [ ] 实现递归解压（嵌套压缩包）
- [ ] 创建 `src/components/ArchiveDialog.tsx` - 密码管理 UI

**Day 3-4: RSS 自动下载**
- [ ] 创建 `src-tauri/src/rss/mod.rs` - RSS 引擎
- [ ] 创建 `src-tauri/src/rss/feed.rs` - Feed 解析
- [ ] 创建 `src-tauri/src/rss/rules.rs` - 过滤规则引擎
- [ ] 实现 RSS/Atom feed 解析（使用 feed-rs）
- [ ] 实现正则表达式过滤规则
- [ ] 实现自动创建下载任务
- [ ] 实现去重和 OPML 导入/导出
- [ ] 创建 `src/components/RssManager.tsx` - RSS 管理 UI

**Day 5: RSS 集成测试**
- [ ] 测试 RSS feed 添加/删除/编辑
- [ ] 测试过滤规则匹配
- [ ] 测试自动下载触发
- [ ] 测试 OPML 导入/导出

#### 第 16 周：JSON-RPC API + WebUI

**Day 1-2: JSON-RPC API**
- [ ] 创建 `src-tauri/src/api/mod.rs` - API 服务入口
- [ ] 创建 `src-tauri/src/api/rpc.rs` - JSON-RPC 2.0 处理
- [ ] 创建 `src-tauri/src/api/websocket.rs` - WebSocket 实时推送
- [ ] 创建 `src-tauri/src/api/auth.rs` - Token/密码认证
- [ ] 实现 aria2 兼容 API（aria2.addUri, aria2.tellStatus 等）
- [ ] 实现自定义 API（downloader.addTask, downloader.getStats 等）
- [ ] 实现 WebSocket 事件订阅

**Day 3-4: WebUI**
- [ ] 创建 `src-webui/` 目录，复用桌面端组件
- [ ] 创建 `src-webui/lib/rpc-api.ts` - JSON-RPC 客户端
- [ ] 创建 `src-webui/lib/ws.ts` - WebSocket 连接管理
- [ ] 实现响应式布局（适配手机/平板）
- [ ] 实现 WebUI 静态资源内嵌到 Rust 二进制
- [ ] 配置 axum 路由（/api, /ws, /）

**Day 5: API 测试**
- [ ] 测试 JSON-RPC 命令执行
- [ ] 测试 WebSocket 实时推送
- [ ] 测试 WebUI 任务管理
- [ ] 测试认证机制

#### 第 17 周：浏览器扩展

**Day 1-2: Chrome 扩展**
- [ ] 创建 `extension/` 目录
- [ ] 创建 `manifest.json`（Chrome MV3）
- [ ] 实现 background.js（Service Worker）
- [ ] 实现 content.js（视频嗅探 - m3u8/mp4/webm 检测）
- [ ] 实现 popup.html/js（弹出窗口 UI）
- [ ] 实现与下载器的通信（Native Messaging 或 HTTP）

**Day 3-4: Firefox 扩展 + 批量捕获**
- [ ] 适配 Firefox WebExtension API
- [ ] 实现右键菜单"使用下载器下载"
- [ ] 实现批量链接捕获
- [ ] 实现智能过滤（排除广告/追踪资源）
- [ ] 测试 Chrome + Firefox 兼容性

**Day 5: 扩展集成测试**
- [ ] 测试视频嗅探准确性
- [ ] 测试链接捕获和转发
- [ ] 测试批量下载
- [ ] 修复兼容性问题

#### 第 18 周：HLS/DASH + 插件系统

**Day 1-2: HLS/DASH 流媒体下载**
- [ ] 创建 `src-tauri/src/engine/hls.rs` - HLS 引擎
- [ ] 实现 m3u8 播放列表解析
- [ ] 实现 ts 分片下载和合并
- [ ] 实现分辨率选择
- [ ] 实现 mpd 清单解析（DASH）
- [ ] 实现 DRM 检测和提示

**Day 3-4: 插件系统基础**
- [ ] 创建 `src-tauri/src/plugin/mod.rs` - 插件系统
- [ ] 创建 `src-tauri/src/plugin/loader.rs` - WASM 加载器
- [ ] 创建 `src-tauri/src/plugin/api.rs` - 插件 API 定义
- [ ] 实现插件生命周期管理（加载/卸载/启用/禁用）
- [ ] 创建 `src/components/PluginManager.tsx` - 插件管理 UI

**Day 5: 集成测试**
- [ ] 测试 HLS 下载（直播/点播）
- [ ] 测试 DASH 下载
- [ ] 测试插件加载和执行
- [ ] 修复发现的 Bug

---

### 8.6 M6 - 优化发布（2 周）

#### 第 19 周：性能优化

**Day 1-2: Rust 端优化**
- [ ] 优化 Cargo.toml 编译配置（LTO、codegen-units、strip）
- [ ] 优化内存使用（减少克隆、使用 Arc/Mutex）
- [ ] 优化异步任务调度
- [ ] 优化数据库查询（索引、预编译语句）

**Day 3-4: 前端优化**
- [ ] 优化虚拟滚动性能
- [ ] 优化图表渲染（节流、Canvas）
- [ ] 优化状态更新（减少不必要的重渲染）
- [ ] 优化打包体积（代码分割、Tree Shaking）

**Day 5: 跨平台测试**
- [ ] Windows 10/11 测试
- [ ] macOS 12+ 测试
- [ ] Ubuntu 20.04+ 测试
- [ ] 修复平台特定问题

#### 第 20 周：打包发布

**Day 1-2: CI/CD 配置**
- [ ] 创建 `.github/workflows/release.yml`:
  - Windows: NSIS 安装包 + 便携 ZIP
  - macOS: DMG
  - Linux: AppImage + DEB
- [ ] 配置代码签名（Windows Authenticode）
- [ ] 配置自动更新（tauri-plugin-updater）

**Day 3-4: 文档编写**
- [ ] 编写 README.md（功能介绍、截图、下载链接）
- [ ] 编写用户文档（安装、配置、使用指南）
- [ ] 编写开发者文档（构建、贡献指南）
- [ ] 编写 CHANGELOG.md

**Day 5: 发布准备**
- [ ] 最终功能测试
- [ ] 性能基准测试
- [ ] 创建 GitHub Release
- [ ] 发布公告

---

### 8.7 依赖清单

**Rust 依赖 (Cargo.toml)**
```toml
[dependencies]
tauri = { version = "2", features = ["tray-icon"] }
tauri-plugin-notification = "2"
tauri-plugin-fs = "2"
tauri-plugin-clipboard = "2"
tauri-plugin-updater = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["full"] }
reqwest = { version = "0.12", features = ["stream", "socks"] }
rusqlite = { version = "0.31", features = ["bundled"] }
toml = "0.8"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
tracing-appender = "0.2"
uuid = { version = "1", features = ["v4"] }
md4 = "0.10"  # ed2k hash
rand = "0.8"
bytes = "1"
anyhow = "1"
thiserror = "2"

# BT 引擎
librqbit = { version = "9", features = ["dht", "upnp", "peer-info"] }

# WebUI / API 服务
axum = "0.7"
tokio-tungstenite = "0.21"
tower-http = { version = "0.5", features = ["cors", "fs"] }

# RSS 解析
feed-rs = "1.3"

# 压缩解压
zip = "0.6"
tar = "0.4"
flate2 = "1.0"
sevenz-rust = "0.5"

# HLS/DASH 解析
m3u8-rs = "5.0"

# 插件系统 WASM 运行时
wasmtime = "22"

# JSON-RPC
jsonrpsee = { version = "0.23", features = ["server"] }

[dependencies.tauri]
version = "2"
features = ["tray-icon"]
```

**前端依赖 (package.json)**
```json
{
  "dependencies": {
    "react": "^18.3",
    "react-dom": "^18.3",
    "@tauri-apps/api": "^2",
    "@tauri-apps/plugin-notification": "^2",
    "@tauri-apps/plugin-fs": "^2",
    "@tauri-apps/plugin-clipboard": "^2",
    "zustand": "^4.5",
    "@tanstack/react-virtual": "^3.8",
    "@tanstack/react-table": "^8.19",
    "@radix-ui/react-dialog": "^1.1",
    "@radix-ui/react-context-menu": "^2.2",
    "@radix-ui/react-dropdown-menu": "^2.1",
    "recharts": "^2.12",
    "lucide-react": "^0.400",
    "framer-motion": "^11.3",
    "@dnd-kit/core": "^6.1",
    "@dnd-kit/sortable": "^8.0",
    "clsx": "^2.1",
    "tailwind-merge": "^2.4",
    "date-fns": "^3.6"
  },
  "devDependencies": {
    "@tauri-apps/cli": "^2",
    "@types/react": "^18.3",
    "@types/react-dom": "^18.3",
    "@vitejs/plugin-react": "^4.3",
    "autoprefixer": "^10.4",
    "postcss": "^8.4",
    "tailwindcss": "^3.4",
    "typescript": "^5.5",
    "vite": "^5.4",
    "vitest": "^2.0",
    "web-ext": "^7.8",
    "webextension-polyfill": "^0.10"
  }
}
```

---

*— End of PRD v3.0 —*

**更新日志**
- v4.0 (2026-05): 基于 IDM/FDM/JDownloader/Motrix/aria2/qBittorrent 等竞品调研，新增浏览器集成（视频嗅探）、远程管理 WebUI、自动解压、RSS 自动下载、插件系统、HLS/DASH 流媒体下载、JSON-RPC API 等功能需求；优化架构设计（接入层抽象、引擎插件化、事件驱动、存储层分离）；新增竞品功能矩阵对比；更新里程碑计划（M5 扩展能力 4 周 + M6 优化发布 2 周，总计 20 周）；更新依赖清单
- v3.0 (2026-05): 基于四份调研报告，补充 Tauri v2 配置、BT 引擎选型（librqbit）、ed2k 协议实现规范、UI 设计规范、详细实现步骤（16 周任务清单）
- v2.0 (2026-05): 补充 HTTP/HTTPS/FTP 下载能力、批量下载、任务管理增强、界面设计细节、错误处理、测试策略、术语表、参考资源
- v1.0 (2025-07): 初始版本
