# 测试报告 — 全协议下载器

> 日期：2026-05-29
> 测试框架：Vitest 2.1.9
> 总测试数：193 通过 / 0 失败

---

## 测试概览

| 测试文件 | 测试数 | 状态 |
|---|---|---|
| `format.test.ts` | 87 | ✅ 全部通过 |
| `taskStore.test.ts` | 65 | ✅ 全部通过 |
| `utils.test.ts` | 41 | ✅ 全部通过 |

---

## 测试覆盖详情

### 1. format.test.ts（87 测试）

| 模块 | 测试内容 |
|---|---|
| `formatSize` | 0B、负值、B/KB/MB/GB/TB、NaN、Infinity、边界值 |
| `formatSpeed` | 0、KB/s、MB/s、TB/s、NaN、Infinity、小数 |
| `formatEta` | null、0、负值、秒、分秒、时分、小数秒向下取整、大值、NaN |
| `formatProgress` | 0%、50%、100%、NaN、Infinity、负值、>1、极小值 |
| `formatDateTime` | ISO 日期、无效日期、空字符串、epoch、未来日期 |
| `formatDuration` | 秒、分秒、时分、0、负值、NaN、Infinity、大时长 |
| `protocolLabel` | HTTP/FTP/BT/MAGNET/ED2K/HLS/DASH、未知协议 |
| `stateLabel` | queued/downloading/paused/seeding/done/error、未知状态 |
| `isValidDownloadUrl` | HTTP/HTTPS/FTP/magnet/ed2k/FTPS/file/m3u8/mpd/torrent、空值、无效协议、大小写、空白、查询参数 |

### 2. taskStore.test.ts（65 测试）

| 模块 | 测试内容 |
|---|---|
| `setTasks` | 填充、替换、空数组、重复 ID |
| `applyUpdate` | 状态更新、速度更新、进度保留、未知 ID 忽略、错误状态、完成状态、做种状态、条件字段更新 |
| `selectTask` | 单选、Ctrl 多选、Shift 范围选择、无 lastSelectedId、同任务重选、大数据集选择 |
| `clearSelection` | 清除所有选中 |
| `removeTask` | 删除任务、清除选中、空 store、多选删除 |
| `setSort` | 切换方向、重置方向 |
| `getSortedTasks` | 按名称/大小/进度/速度排序、空数组、状态优先级、默认排序 |
| `getGlobalStats` | 活跃计数、速度求和、空 store、混合状态 |
| `speedHistory` | 全局速度追踪、节流、大量更新 |
| `setSearchQuery` | 设置/清除 |
| `setStatusFilter` | 设置/重置 |
| 搜索+过滤组合 | 名称搜索、URL 搜索、大小写、空白、状态过滤、组合过滤 |
| `getTaskById` | 按 ID 查找、不存在 ID、空 store |

### 3. utils.test.ts（41 测试）

| 模块 | 测试内容 |
|---|---|
| `detectDownloadUrl` | magnet/ed2k/HTTP/HTTPS/FTP、非 URL、空字符串、空白、大小写、javascript/data 协议、复杂查询字符串、多参数 magnet |
| 任务状态转换 | queued→downloading、downloading→paused/done/error/seeding、paused→downloading、error→downloading、done 不可转换、seeding→done |
| 进度计算 | 0 total、0 downloaded、完成、分数、超过 total、负值 total |
| ETA 计算 | 0 speed、0 total、已完成、正常计算、大文件 |
| 优先级排序 | 高优先级优先、同优先级按时间、空数组、单元素 |

---

## 性能优化记录（迭代 16）

### Cargo.toml Release Profile
- `strip = true`：去除调试符号，减小二进制体积
- `lto = true`：链接时优化，提升运行性能
- `codegen-units = 1`：单编译单元，更好的优化
- `opt-level = "s"`：体积优先优化
- `panic = "abort"`：panic 时直接 abort，减小体积

### SQLite 性能优化
- WAL 模式：并发读写性能提升
- `synchronous=NORMAL`：WAL 模式下足够安全
- `cache_size=-8000`：8MB 页缓存
- `temp_store=MEMORY`：临时表存内存
- `mmap_size=268435456`：256MB mmap 加速大数据库读取
- `busy_timeout=5000`：锁等待超时 5 秒

### 数据库索引
- `idx_tasks_state`：任务状态索引
- `idx_tasks_added_at`：添加时间索引
- `idx_tasks_state_priority`：状态+优先级复合索引
- `idx_tasks_protocol`：协议类型索引
- `idx_history_completed_at`：历史完成时间索引
- `idx_history_protocol`：历史协议类型索引
- `idx_task_files_task_id`：文件表任务 ID 索引

### 新增数据库方法
- `prepare_cached`：缓存预编译语句，减少重复编译开销
- `get_tasks_by_state`：按状态查询（利用复合索引）
- `get_task_stats`：快速统计各状态任务数
- `batch_update_state`：批量更新任务状态
- `archive_completed_tasks`：归档已完成任务到历史表
- `maintenance`：数据库维护（清理孤立记录、VACUUM）
- `get_db_size`：获取数据库文件大小

---

## 运行命令

```bash
cd downloader && npm test
```
