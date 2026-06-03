// SQLite 数据库模块
// 负责任务、配置、分块位图等数据的持久化存储

use anyhow::Result;
use rusqlite::{params, Connection};
use std::path::Path;

/// 任务状态枚举（与前端同步）
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum TaskState {
    Queued,
    Downloading,
    Paused,
    Seeding,
    Done,
    Error,
}

impl TaskState {
    pub fn as_str(&self) -> &'static str {
        match self {
            TaskState::Queued => "queued",
            TaskState::Downloading => "downloading",
            TaskState::Paused => "paused",
            TaskState::Seeding => "seeding",
            TaskState::Done => "done",
            TaskState::Error => "error",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "queued" => TaskState::Queued,
            "downloading" => TaskState::Downloading,
            "paused" => TaskState::Paused,
            "seeding" => TaskState::Seeding,
            "done" => TaskState::Done,
            "error" => TaskState::Error,
            _ => TaskState::Queued,
        }
    }
}

/// 协议类型
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum Protocol {
    Http,
    Ftp,
    Bt,
    Magnet,
    Ed2k,
    Hls,
    Dash,
}

impl Protocol {
    pub fn as_str(&self) -> &'static str {
        match self {
            Protocol::Http => "HTTP",
            Protocol::Ftp => "FTP",
            Protocol::Bt => "BT",
            Protocol::Magnet => "MAGNET",
            Protocol::Ed2k => "ED2K",
            Protocol::Hls => "HLS",
            Protocol::Dash => "DASH",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "HTTP" | "HTTPS" => Protocol::Http,
            "FTP" | "FTPS" => Protocol::Ftp,
            "BT" => Protocol::Bt,
            "MAGNET" => Protocol::Magnet,
            "ED2K" => Protocol::Ed2k,
            "HLS" => Protocol::Hls,
            "DASH" | "MPD" => Protocol::Dash,
            _ => Protocol::Http,
        }
    }
}

/// 文件信息（多文件任务的子文件）
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileInfo {
    pub index: i32,
    pub path: String,
    pub size: u64,
    pub priority: i32, // 0=跳过 1=正常 2=高
}

/// 任务状态信息（完整）
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskStatus {
    pub id: String,
    pub name: String,
    pub protocol: Protocol,
    pub state: TaskState,
    pub url: String,
    pub save_path: String,
    pub total_size: u64,
    pub downloaded: u64,
    pub uploaded: u64,
    pub download_speed: u64,
    pub upload_speed: u64,
    pub progress: f32,
    pub peers: u32,
    pub seeds: u32,
    pub eta: Option<u64>, // 剩余秒数
    pub files: Vec<FileInfo>,
    pub error: Option<String>,
    pub added_at: String,
    pub completed_at: Option<String>,
    pub priority: i32,
    pub download_limit: u64, // 单任务下载速度限制 bytes/sec，0=不限速
    pub upload_limit: u64,   // 单任务上传速度限制 bytes/sec，0=不限速
}

/// 添加任务参数
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskParams {
    pub url: String,
    pub save_path: String,
    pub file_name: Option<String>,
    pub proxy: Option<String>,
    pub speed_limit: Option<u64>,
    pub start_immediately: bool,
    pub only_files: Option<Vec<usize>>,  // BT/Magnet 文件选择
    pub mirror_urls: Option<Vec<String>>, // 镜像URL列表
    /// HTTP 认证（Basic/Digest），格式 "user:pass"
    #[serde(default)]
    pub http_auth: Option<String>,
    /// 自定义 Cookie
    #[serde(default)]
    pub http_cookie: Option<String>,
    /// 自定义 HTTP Headers（key:value 对）
    #[serde(default)]
    pub http_headers: Option<Vec<(String, String)>>,
}

/// 下载历史记录
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadHistory {
    pub id: String,
    pub task_id: Option<String>,
    pub name: String,
    pub url: Option<String>,
    pub protocol: String,
    pub save_path: String,
    pub file_path: Option<String>,
    pub total_size: u64,
    pub downloaded: u64,
    pub average_speed: u64,
    pub duration: u64,
    pub completed_at: String,
}

/// 标签
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tag {
    pub id: i64,
    pub name: String,
    pub color: String,
    pub created_at: String,
}

/// 创建标签参数
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTagParams {
    pub name: String,
    pub color: Option<String>,
}

/// SQLite 数据库
pub struct Database {
    conn: Connection,
}

/// 将数据库行转换为 TaskStatus（消除重复的行映射代码）
/// 列顺序: id, name, protocol, url, save_path, total_size, downloaded, uploaded,
///          download_speed, upload_speed, peers, state, error, priority, added_at,
///          completed_at, download_limit, upload_limit
fn row_to_task(row: &rusqlite::Row) -> rusqlite::Result<TaskStatus> {
    let total_size = row.get::<_, i64>(5).unwrap_or(0).max(0) as u64;
    let downloaded = row.get::<_, i64>(6).unwrap_or(0).max(0) as u64;
    Ok(TaskStatus {
        id: row.get(0)?,
        name: row.get(1)?,
        protocol: Protocol::from_str(&row.get::<_, String>(2).unwrap_or_default()),
        url: row.get(3)?,
        save_path: row.get(4)?,
        total_size,
        downloaded,
        uploaded: row.get::<_, i64>(7).unwrap_or(0).max(0) as u64,
        download_speed: row.get::<_, i64>(8).unwrap_or(0).max(0) as u64,
        upload_speed: row.get::<_, i64>(9).unwrap_or(0).max(0) as u64,
        progress: if total_size > 0 { (downloaded as f32 / total_size as f32).min(1.0) } else { 0.0 },
        peers: row.get::<_, i32>(10).unwrap_or(0).max(0) as u32,
        seeds: 0,
        eta: None,
        files: Vec::new(),
        state: TaskState::from_str(&row.get::<_, String>(11).unwrap_or_default()),
        error: row.get(12)?,
        priority: row.get(13).unwrap_or(1),
        added_at: row.get(14).unwrap_or_default(),
        completed_at: row.get(15)?,
        download_limit: row.get::<_, i64>(16).unwrap_or(0).max(0) as u64,
        upload_limit: row.get::<_, i64>(17).unwrap_or(0).max(0) as u64,
    })
}

impl Database {
    /// 创建或打开数据库，执行迁移
    pub fn new(db_path: &Path) -> Result<Self> {
        let conn = Connection::open(db_path)?;

        // 性能优化 PRAGMA
        conn.execute_batch(
            "PRAGMA journal_mode=WAL;          -- WAL 模式提升并发读写性能
             PRAGMA foreign_keys=ON;            -- 启用外键约束
             PRAGMA synchronous=NORMAL;         -- WAL 模式下 NORMAL 已足够安全
             PRAGMA cache_size=-8000;           -- 8MB 页缓存（负值单位 KiB）
             PRAGMA temp_store=MEMORY;          -- 临时表存内存
             PRAGMA mmap_size=268435456;        -- 256MB mmap，加速大数据库读取
             PRAGMA busy_timeout=5000;          -- 锁等待超时 5 秒
            ",
        )?;

        let db = Database { conn };
        db.migrate()?;
        Ok(db)
    }

    /// 数据库迁移：创建表结构
    fn migrate(&self) -> Result<()> {
        self.conn.execute_batch(
            "
            -- 任务表
            CREATE TABLE IF NOT EXISTS tasks (
                id              TEXT PRIMARY KEY,
                name            TEXT NOT NULL,
                protocol        TEXT NOT NULL,
                url             TEXT NOT NULL,
                save_path       TEXT NOT NULL,
                total_size      INTEGER DEFAULT 0,
                downloaded      INTEGER DEFAULT 0,
                uploaded        INTEGER DEFAULT 0,
                download_speed  INTEGER DEFAULT 0,
                upload_speed    INTEGER DEFAULT 0,
                peers           INTEGER DEFAULT 0,
                state           TEXT NOT NULL DEFAULT 'queued',
                error           TEXT,
                priority        INTEGER DEFAULT 1,
                added_at        DATETIME DEFAULT CURRENT_TIMESTAMP,
                completed_at    DATETIME,
                metadata        TEXT
            );

            -- 文件表（多文件任务的子文件）
            CREATE TABLE IF NOT EXISTS task_files (
                task_id     TEXT REFERENCES tasks(id) ON DELETE CASCADE,
                idx         INTEGER,
                path        TEXT NOT NULL,
                size        INTEGER NOT NULL,
                priority    INTEGER DEFAULT 1,
                PRIMARY KEY (task_id, idx)
            );

            -- 分块位图（断点续传核心）
            CREATE TABLE IF NOT EXISTS piece_bitmaps (
                task_id     TEXT PRIMARY KEY REFERENCES tasks(id) ON DELETE CASCADE,
                bitmap      BLOB NOT NULL
            );

            -- 下载历史表
            CREATE TABLE IF NOT EXISTS download_history (
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
                duration    INTEGER,
                completed_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );

            -- 标签表
            CREATE TABLE IF NOT EXISTS tags (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                name        TEXT NOT NULL UNIQUE,
                color       TEXT DEFAULT '#3B82F6',
                created_at  DATETIME DEFAULT CURRENT_TIMESTAMP
            );

            -- 任务标签关联表
            CREATE TABLE IF NOT EXISTS task_tags (
                task_id     TEXT REFERENCES tasks(id) ON DELETE CASCADE,
                tag_id      INTEGER REFERENCES tags(id) ON DELETE CASCADE,
                PRIMARY KEY (task_id, tag_id)
            );

            -- RSS 订阅持久化表
            CREATE TABLE IF NOT EXISTS rss_feeds (
                id              TEXT PRIMARY KEY,
                name            TEXT NOT NULL,
                url             TEXT NOT NULL,
                enabled         INTEGER DEFAULT 1,
                interval_sec    INTEGER DEFAULT 1800,
                rules_json      TEXT DEFAULT '[]',
                save_dir        TEXT DEFAULT '',
                last_update     TEXT,
                processed_json  TEXT DEFAULT '[]'
            );

            -- 调度规则持久化表
            CREATE TABLE IF NOT EXISTS schedule_rules (
                id              TEXT PRIMARY KEY,
                name            TEXT NOT NULL,
                rule_type       TEXT NOT NULL,
                cron_expression TEXT NOT NULL,
                task_id         TEXT,
                params_json     TEXT DEFAULT '{}',
                enabled         INTEGER DEFAULT 1,
                created_at      TEXT NOT NULL,
                last_executed   TEXT
            );

            -- 带宽计划持久化表
            CREATE TABLE IF NOT EXISTS bandwidth_schedules (
                id              INTEGER PRIMARY KEY AUTOINCREMENT,
                start_time      TEXT NOT NULL,
                end_time        TEXT NOT NULL,
                download_speed  INTEGER DEFAULT 0,
                upload_speed    INTEGER DEFAULT 0,
                weekdays_json   TEXT DEFAULT 'null'
            );

            -- 解压密码持久化表
            CREATE TABLE IF NOT EXISTS archive_passwords (
                id              TEXT PRIMARY KEY,
                name            TEXT NOT NULL,
                password        TEXT NOT NULL,
                pattern         TEXT DEFAULT '',
                enabled         INTEGER DEFAULT 1,
                use_count       INTEGER DEFAULT 0,
                created_at      TEXT DEFAULT CURRENT_TIMESTAMP
            );

            -- 索引
            CREATE INDEX IF NOT EXISTS idx_tasks_state ON tasks(state);
            CREATE INDEX IF NOT EXISTS idx_tasks_added_at ON tasks(added_at);
            CREATE INDEX IF NOT EXISTS idx_tasks_state_priority ON tasks(state, priority DESC);
            CREATE INDEX IF NOT EXISTS idx_tasks_protocol ON tasks(protocol);
            CREATE INDEX IF NOT EXISTS idx_history_completed_at ON download_history(completed_at);
            CREATE INDEX IF NOT EXISTS idx_history_protocol ON download_history(protocol);
            CREATE INDEX IF NOT EXISTS idx_task_files_task_id ON task_files(task_id);
            CREATE INDEX IF NOT EXISTS idx_task_tags_task_id ON task_tags(task_id);
            CREATE INDEX IF NOT EXISTS idx_task_tags_tag_id ON task_tags(tag_id);
            ",
        )?;

        // 增量迁移：为旧数据库添加新列（忽略已存在的列）
        let migrations = [
            "ALTER TABLE tasks ADD COLUMN download_speed INTEGER DEFAULT 0",
            "ALTER TABLE tasks ADD COLUMN upload_speed INTEGER DEFAULT 0",
            "ALTER TABLE tasks ADD COLUMN peers INTEGER DEFAULT 0",
            "ALTER TABLE tasks ADD COLUMN download_limit INTEGER DEFAULT 0",
            "ALTER TABLE tasks ADD COLUMN upload_limit INTEGER DEFAULT 0",
        ];
        for sql in &migrations {
            // 忽略 "duplicate column name" 错误（列已存在时的正常情况）
            if let Err(e) = self.conn.execute(sql, []) {
                tracing::debug!("迁移 SQL 跳过（可能已存在）: {} - {}", sql, e);
            }
        }
        Ok(())
    }

    /// 插入新任务
    pub fn insert_task(&self, task: &TaskStatus) -> Result<()> {
        self.conn.execute(
            "INSERT INTO tasks (id, name, protocol, url, save_path, total_size, state, priority, added_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                task.id,
                task.name,
                task.protocol.as_str(),
                task.url,
                task.save_path,
                task.total_size,
                task.state.as_str(),
                task.priority,
                task.added_at,
            ],
        )?;
        Ok(())
    }

    /// 保存任务下载参数到 metadata 列（用于重启后恢复）
    pub fn save_task_params(&self, task_id: &str, params: &TaskParams) -> Result<()> {
        let metadata = serde_json::to_string(params).unwrap_or_default();
        self.conn.execute(
            "UPDATE tasks SET metadata = ?1 WHERE id = ?2",
            params![metadata, task_id],
        )?;
        Ok(())
    }

    /// 读取任务的下载参数（用于重启后恢复）
    pub fn get_task_params(&self, task_id: &str) -> Result<Option<TaskParams>> {
        let mut stmt = self.conn.prepare_cached("SELECT metadata FROM tasks WHERE id = ?1")?;
        let result: Option<String> = stmt.query_row(params![task_id], |row| row.get(0)).ok();
        match result {
            Some(metadata) if !metadata.is_empty() => {
                Ok(serde_json::from_str(&metadata).ok())
            }
            _ => Ok(None),
        }
    }

    /// 保存任务文件列表（BT/Magnet 多文件任务）
    pub fn insert_task_files(&self, task_id: &str, files: &[FileInfo]) -> Result<()> {
        // 先删除旧文件记录
        self.conn.execute("DELETE FROM task_files WHERE task_id = ?1", params![task_id])?;
        {
            let mut stmt = self.conn.prepare_cached(
                "INSERT INTO task_files (task_id, file_index, file_path, file_size, priority) VALUES (?1, ?2, ?3, ?4, ?5)"
            )?;
            for f in files {
                stmt.execute(params![task_id, f.index, f.path, f.size as i64, f.priority])?;
            }
        }
        Ok(())
    }

    /// 读取任务的文件列表
    pub fn get_task_files(&self, task_id: &str) -> Result<Vec<FileInfo>> {
        let mut stmt = self.conn.prepare_cached(
            "SELECT file_index, file_path, file_size, priority FROM task_files WHERE task_id = ?1 ORDER BY file_index"
        )?;
        let files = stmt.query_map(params![task_id], |row| {
            Ok(FileInfo {
                index: row.get(0)?,
                path: row.get(1)?,
                size: row.get::<_, i64>(2).unwrap_or(0) as u64,
                priority: row.get(3).unwrap_or(1),
            })
        })?.collect::<Result<Vec<_>, _>>()?;
        Ok(files)
    }

    /// 更新任务状态
    pub fn update_task_state(&self, id: &str, state: &TaskState) -> Result<()> {
        self.conn.execute(
            "UPDATE tasks SET state = ?1 WHERE id = ?2",
            params![state.as_str(), id],
        )?;
        Ok(())
    }

    /// 更新任务进度（下载量、速度、peer 数）
    pub fn update_task_progress(
        &self,
        id: &str,
        downloaded: u64,
        uploaded: u64,
        download_speed: u64,
        upload_speed: u64,
        peers: u32,
    ) -> Result<()> {
        self.conn.execute(
            "UPDATE tasks SET downloaded = ?1, uploaded = ?2, download_speed = ?3, upload_speed = ?4, peers = ?5 WHERE id = ?6",
            params![downloaded as i64, uploaded as i64, download_speed as i64, upload_speed as i64, peers as i32, id],
        )?;
        Ok(())
    }

    /// 更新任务完成时间
    pub fn complete_task(&self, id: &str) -> Result<()> {
        self.conn.execute(
            "UPDATE tasks SET state = 'done', completed_at = CURRENT_TIMESTAMP WHERE id = ?1",
            params![id],
        )?;
        Ok(())
    }

    /// 更新任务错误信息
    pub fn set_task_error(&self, id: &str, error: &str) -> Result<()> {
        self.conn.execute(
            "UPDATE tasks SET state = 'error', error = ?1 WHERE id = ?2",
            params![error, id],
        )?;
        Ok(())
    }

    /// 删除任务
    pub fn delete_task(&self, id: &str) -> Result<()> {
        self.conn.execute("DELETE FROM tasks WHERE id = ?1", params![id])?;
        Ok(())
    }

    /// 批量清除已完成和出错的任务（单条 SQL，不逐行删除）
    pub fn purge_completed_tasks(&self) -> Result<usize> {
        let count = self.conn.execute(
            "DELETE FROM tasks WHERE state IN ('done', 'error')",
            [],
        )?;
        Ok(count)
    }

    /// 获取单个任务
    pub fn get_task(&self, id: &str) -> Result<Option<TaskStatus>> {
        let mut stmt = self.conn.prepare_cached(
            "SELECT id, name, protocol, url, save_path, total_size, downloaded, uploaded,
                    download_speed, upload_speed, peers, state, error, priority, added_at, completed_at,
                    download_limit, upload_limit
             FROM tasks WHERE id = ?1",
        )?;

        let result = stmt.query_row(params![id], row_to_task);

        match result {
            Ok(mut task) => {
                // 加载关联的文件列表
                task.files = self.get_task_files(id).unwrap_or_default();
                Ok(Some(task))
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// 获取所有任务
    pub fn get_all_tasks(&self) -> Result<Vec<TaskStatus>> {
        let mut stmt = self.conn.prepare_cached(
            "SELECT id, name, protocol, url, save_path, total_size, downloaded, uploaded,
                    download_speed, upload_speed, peers, state, error, priority, added_at, completed_at,
                    download_limit, upload_limit
             FROM tasks ORDER BY added_at DESC",
        )?;

        let tasks = stmt
            .query_map([], row_to_task)?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(tasks)
    }

    /// 获取未完成任务（用于程序重启恢复）
    pub fn get_unfinished_tasks(&self) -> Result<Vec<TaskStatus>> {
        let mut stmt = self.conn.prepare_cached(
            "SELECT id, name, protocol, url, save_path, total_size, downloaded, uploaded,
                    download_speed, upload_speed, peers, state, error, priority, added_at, completed_at,
                    download_limit, upload_limit
             FROM tasks WHERE state IN ('downloading', 'queued', 'paused') ORDER BY priority DESC, added_at ASC",
        )?;

        let tasks = stmt
            .query_map([], row_to_task)?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(tasks)
    }

    /// 更新任务优先级
    pub fn update_task_priority(&self, id: &str, priority: i32) -> Result<()> {
        self.conn.execute(
            "UPDATE tasks SET priority = ?1 WHERE id = ?2",
            params![priority, id],
        )?;
        Ok(())
    }

    /// 更新任务速度限制
    /// download_limit 和 upload_limit 单位为 bytes/sec，0 表示不限速
    pub fn update_task_speed_limit(&self, id: &str, download_limit: u64, upload_limit: u64) -> Result<()> {
        self.conn.execute(
            "UPDATE tasks SET download_limit = ?1, upload_limit = ?2 WHERE id = ?3",
            params![download_limit as i64, upload_limit as i64, id],
        )?;
        Ok(())
    }

    /// 记录下载历史
    pub fn record_download_history(
        &self,
        task_id: &str,
        name: &str,
        url: &str,
        protocol: &str,
        save_path: &str,
        total_size: u64,
        downloaded: u64,
        average_speed: u64,
        duration: u64,
    ) -> Result<()> {
        let id = uuid::Uuid::new_v4().to_string();
        let file_path = std::path::Path::new(save_path).join(name);
        let file_path_str = file_path.to_string_lossy().to_string();

        self.conn.execute(
            "INSERT INTO download_history (id, task_id, name, url, protocol, save_path, file_path, total_size, downloaded, average_speed, duration)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                id,
                task_id,
                name,
                url,
                protocol,
                save_path,
                file_path_str,
                total_size as i64,
                downloaded as i64,
                average_speed as i64,
                duration as i64,
            ],
        )?;
        Ok(())
    }

    /// 获取下载历史
    pub fn get_download_history(&self, limit: u32) -> Result<Vec<DownloadHistory>> {
        let mut stmt = self.conn.prepare_cached(
            "SELECT id, task_id, name, url, protocol, save_path, file_path, total_size, downloaded, average_speed, duration, completed_at
             FROM download_history ORDER BY completed_at DESC LIMIT ?1",
        )?;

        let history = stmt
            .query_map(params![limit], |row| {
                Ok(DownloadHistory {
                    id: row.get(0)?,
                    task_id: row.get(1)?,
                    name: row.get(2)?,
                    url: row.get(3)?,
                    protocol: row.get(4)?,
                    save_path: row.get(5)?,
                    file_path: row.get(6)?,
                    total_size: row.get::<_, i64>(7)? as u64,
                    downloaded: row.get::<_, i64>(8)? as u64,
                    average_speed: row.get::<_, i64>(9)? as u64,
                    duration: row.get::<_, i64>(10)? as u64,
                    completed_at: row.get(11)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(history)
    }

    /// 清空下载历史
    pub fn clear_download_history(&self) -> Result<usize> {
        let count = self.conn.execute("DELETE FROM download_history", [])?;
        Ok(count)
    }

    /// 更新任务进度（带下载速度和 ETA 计算用的时间戳）
    pub fn update_task_progress_full(
        &self,
        id: &str,
        downloaded: u64,
        total_size: u64,
    ) -> Result<()> {
        self.conn.execute(
            "UPDATE tasks SET downloaded = ?1, total_size = ?2 WHERE id = ?3",
            params![downloaded as i64, total_size as i64, id],
        )?;
        Ok(())
    }

    /// 按状态获取任务（利用复合索引）
    pub fn get_tasks_by_state(&self, state: &TaskState) -> Result<Vec<TaskStatus>> {
        let mut stmt = self.conn.prepare_cached(
            "SELECT id, name, protocol, url, save_path, total_size, downloaded, uploaded,
                    download_speed, upload_speed, peers, state, error, priority, added_at, completed_at,
                    download_limit, upload_limit
             FROM tasks WHERE state = ?1 ORDER BY priority DESC, added_at ASC",
        )?;

        let tasks = stmt
            .query_map(params![state.as_str()], row_to_task)?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(tasks)
    }

    /// 获取任务统计（利用索引快速统计）
    pub fn get_task_stats(&self) -> Result<(u32, u32, u32, u32, u32, u32)> {
        let mut stmt = self.conn.prepare_cached(
            "SELECT state, COUNT(*) FROM tasks GROUP BY state",
        )?;

        let mut queued = 0u32;
        let mut downloading = 0u32;
        let mut paused = 0u32;
        let mut seeding = 0u32;
        let mut done = 0u32;
        let mut error = 0u32;

        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, u32>(1)?))
        })?;

        for row in rows {
            let (state, count) = row?;
            match state.as_str() {
                "queued" => queued = count,
                "downloading" => downloading = count,
                "paused" => paused = count,
                "seeding" => seeding = count,
                "done" => done = count,
                "error" => error = count,
                _ => {}
            }
        }

        Ok((queued, downloading, paused, seeding, done, error))
    }

    /// 批量更新任务状态（使用事务提高性能）
    pub fn batch_update_state(&self, ids: &[String], state: &TaskState) -> Result<usize> {
        let mut total = 0;
        let state_str = state.as_str();
        // 使用事务包裹批量操作，减少磁盘 I/O
        let tx = self.conn.unchecked_transaction()?;
        {
            let mut stmt = self.conn.prepare_cached("UPDATE tasks SET state = ?1 WHERE id = ?2")?;
            for id in ids {
                total += stmt.execute(params![state_str, id])?;
            }
        }
        tx.commit()?;
        Ok(total)
    }

    /// 将已完成任务归档到历史记录并从活动表删除
    pub fn archive_completed_tasks(&self, older_than_days: u32) -> Result<usize> {
        let count = self.conn.execute(
            "INSERT INTO download_history (id, task_id, name, url, protocol, save_path, total_size, downloaded, average_speed, duration, completed_at)
             SELECT hex(randomblob(16)), id, name, url, protocol, save_path, total_size, downloaded,
                    CASE WHEN (julianday(completed_at) - julianday(added_at)) * 86400 > 0
                         THEN CAST(downloaded / ((julianday(completed_at) - julianday(added_at)) * 86400) AS INTEGER)
                         ELSE 0 END,
                    CAST((julianday(completed_at) - julianday(added_at)) * 86400 AS INTEGER),
                    completed_at
             FROM tasks WHERE state = 'done' AND completed_at < datetime('now', ?1 || ' days')",
            params![format!("-{}", older_than_days)],
        )?;
        self.conn.execute(
            "DELETE FROM tasks WHERE state = 'done' AND completed_at < datetime('now', ?1 || ' days')",
            params![format!("-{}", older_than_days)],
        )?;
        Ok(count)
    }

    /// 数据库维护：清理已删除任务的孤立文件记录，VACUUM 紧缩空间
    pub fn maintenance(&self) -> Result<()> {
        // 清理孤立文件记录
        self.conn.execute(
            "DELETE FROM task_files WHERE task_id NOT IN (SELECT id FROM tasks)",
            [],
        )?;
        // 清理孤立分块位图
        self.conn.execute(
            "DELETE FROM piece_bitmaps WHERE task_id NOT IN (SELECT id FROM tasks)",
            [],
        )?;
        // VACUUM 紧缩数据库文件（需要额外磁盘空间）
        self.conn.execute_batch("VACUUM;")?;
        Ok(())
    }

    /// 获取数据库大小信息
    pub fn get_db_size(&self) -> Result<(i64, i64)> {
        let page_count: i64 = self.conn
            .query_row("PRAGMA page_count", [], |row| row.get(0))?;
        let page_size: i64 = self.conn
            .query_row("PRAGMA page_size", [], |row| row.get(0))?;
        Ok((page_count * page_size, page_count))
    }

    /// 添加镜像URL到任务
    pub fn add_mirror_url(&self, task_id: &str, mirror_url: &str) -> Result<()> {
        // 使用 metadata 字段存储镜像URL列表（JSON格式）
        let metadata: String = self.conn
            .query_row(
                "SELECT COALESCE(metadata, '{}') FROM tasks WHERE id = ?1",
                params![task_id],
                |row| row.get(0),
            )?;

        let mut meta: serde_json::Value = serde_json::from_str(&metadata).unwrap_or_default();
        // 确保 mirrorUrls 字段存在
        if meta.get("mirrorUrls").and_then(|v| v.as_array()).is_none() {
            meta["mirrorUrls"] = serde_json::Value::Array(Vec::new());
        }
        // 安全访问：get_mut + as_array_mut 已通过上方 is_none 检查确保字段存在
        let mirrors = match meta.get_mut("mirrorUrls").and_then(|v| v.as_array_mut()) {
            Some(arr) => arr,
            None => return Ok(()),
        };

        // 检查是否已存在
        let url_exists = mirrors.iter().any(|v| v.as_str() == Some(mirror_url));
        if !url_exists {
            mirrors.push(serde_json::Value::String(mirror_url.to_string()));
        }

        let new_metadata = serde_json::to_string(&meta).unwrap_or_default();
        self.conn.execute(
            "UPDATE tasks SET metadata = ?1 WHERE id = ?2",
            params![new_metadata, task_id],
        )?;
        Ok(())
    }

    /// 移除任务的镜像URL
    pub fn remove_mirror_url(&self, task_id: &str, mirror_url: &str) -> Result<()> {
        let metadata: String = self.conn
            .query_row(
                "SELECT COALESCE(metadata, '{}') FROM tasks WHERE id = ?1",
                params![task_id],
                |row| row.get(0),
            )?;

        let mut meta: serde_json::Value = serde_json::from_str(&metadata).unwrap_or_default();
        if let Some(mirrors) = meta.get_mut("mirrorUrls").and_then(|v| v.as_array_mut()) {
            mirrors.retain(|v| v.as_str() != Some(mirror_url));
        }

        let new_metadata = serde_json::to_string(&meta).unwrap_or_default();
        self.conn.execute(
            "UPDATE tasks SET metadata = ?1 WHERE id = ?2",
            params![new_metadata, task_id],
        )?;
        Ok(())
    }

    /// 获取任务的镜像URL列表
    pub fn get_mirror_urls(&self, task_id: &str) -> Result<Vec<String>> {
        let metadata: String = self.conn
            .query_row(
                "SELECT COALESCE(metadata, '{}') FROM tasks WHERE id = ?1",
                params![task_id],
                |row| row.get(0),
            )?;

        let meta: serde_json::Value = serde_json::from_str(&metadata).unwrap_or_default();
        let mirrors = meta.get("mirrorUrls")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        Ok(mirrors)
    }

    // ==================== RSS 订阅持久化 ====================

    /// 保存所有 RSS 订阅（覆盖写入）
    pub fn save_rss_feeds(&self, feeds: &[crate::rss::RssFeed]) -> Result<()> {
        self.conn.execute("DELETE FROM rss_feeds", [])?;
        {
            let mut stmt = self.conn.prepare_cached(
                "INSERT INTO rss_feeds (id, name, url, enabled, interval_sec, rules_json, save_dir, last_update, processed_json)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)"
            )?;
            for feed in feeds {
                let rules_json = serde_json::to_string(&feed.rules).unwrap_or_default();
                let processed_vec: Vec<&String> = feed.processed.iter().take(1000).collect();
                let processed_json = serde_json::to_string(&processed_vec).unwrap_or_default();
                stmt.execute(params![
                    feed.id, feed.name, feed.url, feed.enabled as i32,
                    feed.interval as i64, rules_json, feed.save_dir,
                    feed.last_update, processed_json
                ])?;
            }
        }
        Ok(())
    }

    /// 加载所有 RSS 订阅
    pub fn load_rss_feeds(&self) -> Result<Vec<crate::rss::RssFeed>> {
        let mut stmt = self.conn.prepare_cached(
            "SELECT id, name, url, enabled, interval_sec, rules_json, save_dir, last_update, processed_json FROM rss_feeds"
        )?;
        let feeds = stmt.query_map([], |row| {
            let rules_json: String = row.get(5).unwrap_or_default();
            let processed_json: String = row.get(8).unwrap_or_default();
            let rules: Vec<crate::rss::FilterRule> = serde_json::from_str(&rules_json).unwrap_or_default();
            let processed_vec: Vec<String> = serde_json::from_str(&processed_json).unwrap_or_default();
            let processed: std::collections::HashSet<String> = processed_vec.into_iter().collect();
            Ok(crate::rss::RssFeed {
                id: row.get(0)?,
                name: row.get(1)?,
                url: row.get(2)?,
                enabled: row.get::<_, i32>(3).unwrap_or(1) != 0,
                interval: row.get::<_, i64>(4).unwrap_or(1800) as u64,
                rules,
                save_dir: row.get(6).unwrap_or_default(),
                last_update: row.get(7)?,
                processed,
                last_poll: None,
            })
        })?.collect::<Result<Vec<_>, _>>()?;
        Ok(feeds)
    }

    // ==================== 调度规则持久化 ====================

    /// 保存所有调度规则（覆盖写入）
    pub fn save_schedule_rules(&self, rules: &[crate::schedule::ScheduleRule]) -> Result<()> {
        self.conn.execute("DELETE FROM schedule_rules", [])?;
        {
            let mut stmt = self.conn.prepare_cached(
                "INSERT INTO schedule_rules (id, name, rule_type, cron_expression, task_id, params_json, enabled, created_at, last_executed)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)"
            )?;
            for rule in rules {
                let rule_type = format!("{:?}", rule.rule_type);
                let params_json = serde_json::to_string(&rule.params).unwrap_or_default();
                stmt.execute(params![
                    rule.id, rule.name, rule_type, rule.cron_expression,
                    rule.task_id, params_json, rule.enabled as i32,
                    rule.created_at, rule.last_executed_at
                ])?;
            }
        }
        Ok(())
    }

    /// 加载所有调度规则
    pub fn load_schedule_rules(&self) -> Result<Vec<crate::schedule::ScheduleRule>> {
        let mut stmt = self.conn.prepare_cached(
            "SELECT id, name, rule_type, cron_expression, task_id, params_json, enabled, created_at, last_executed FROM schedule_rules"
        )?;
        let rules = stmt.query_map([], |row| {
            let rule_type_str: String = row.get(2)?;
            let params_json: String = row.get(5).unwrap_or_default();
            let rule_type = match rule_type_str.as_str() {
                "StartTask" => crate::schedule::ScheduleRuleType::StartTask,
                "PauseTask" => crate::schedule::ScheduleRuleType::PauseTask,
                "BandwidthPlan" => crate::schedule::ScheduleRuleType::BandwidthPlan,
                _ => crate::schedule::ScheduleRuleType::StartTask,
            };
            let params: crate::schedule::ScheduleParams = serde_json::from_str(&params_json).unwrap_or_default();
            Ok(crate::schedule::ScheduleRule {
                id: row.get(0)?,
                name: row.get(1)?,
                rule_type,
                cron_expression: row.get(3)?,
                task_id: row.get(4)?,
                params,
                enabled: row.get::<_, i32>(6).unwrap_or(1) != 0,
                created_at: row.get(7)?,
                last_executed_at: row.get(8)?,
            })
        })?.collect::<Result<Vec<_>, _>>()?;
        Ok(rules)
    }

    // ==================== 带宽计划持久化 ====================

    /// 保存所有带宽计划（覆盖写入）
    pub fn save_bandwidth_schedules(&self, schedules: &[crate::schedule::BandwidthSchedule]) -> Result<()> {
        self.conn.execute("DELETE FROM bandwidth_schedules", [])?;
        {
            let mut stmt = self.conn.prepare_cached(
                "INSERT INTO bandwidth_schedules (start_time, end_time, download_speed, upload_speed, weekdays_json)
                 VALUES (?1, ?2, ?3, ?4, ?5)"
            )?;
            for s in schedules {
                let weekdays_json = serde_json::to_string(&s.weekdays).unwrap_or_default();
                stmt.execute(params![
                    s.start_time, s.end_time,
                    s.download_speed as i64, s.upload_speed as i64,
                    weekdays_json
                ])?;
            }
        }
        Ok(())
    }

    /// 加载所有带宽计划
    pub fn load_bandwidth_schedules(&self) -> Result<Vec<crate::schedule::BandwidthSchedule>> {
        let mut stmt = self.conn.prepare_cached(
            "SELECT start_time, end_time, download_speed, upload_speed, weekdays_json FROM bandwidth_schedules"
        )?;
        let schedules = stmt.query_map([], |row| {
            let weekdays_json: String = row.get(4).unwrap_or_default();
            let weekdays: Option<Vec<u8>> = serde_json::from_str(&weekdays_json).unwrap_or(None);
            Ok(crate::schedule::BandwidthSchedule {
                start_time: row.get(0)?,
                end_time: row.get(1)?,
                download_speed: row.get::<_, i64>(2).unwrap_or(0) as u64,
                upload_speed: row.get::<_, i64>(3).unwrap_or(0) as u64,
                weekdays,
            })
        })?.collect::<Result<Vec<_>, _>>()?;
        Ok(schedules)
    }

    // ==================== 解压密码持久化 ====================

    /// 保存所有解压密码（覆盖写入）
    pub fn save_archive_passwords(&self, passwords: &[crate::archive::SavedPassword]) -> Result<()> {
        self.conn.execute("DELETE FROM archive_passwords", [])?;
        {
            let mut stmt = self.conn.prepare_cached(
                "INSERT INTO archive_passwords (id, name, password, pattern, enabled, use_count, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)"
            )?;
            for pw in passwords {
                stmt.execute(params![
                    pw.id, pw.name, pw.password, pw.pattern,
                    pw.enabled as i32, pw.use_count as i64, pw.created_at
                ])?;
            }
        }
        Ok(())
    }

    /// 加载所有解压密码
    pub fn load_archive_passwords(&self) -> Result<Vec<crate::archive::SavedPassword>> {
        let mut stmt = self.conn.prepare_cached(
            "SELECT id, name, password, pattern, enabled, use_count, created_at FROM archive_passwords"
        )?;
        let passwords = stmt.query_map([], |row| {
            Ok(crate::archive::SavedPassword {
                id: row.get(0)?,
                name: row.get(1)?,
                password: row.get(2)?,
                pattern: row.get(3).unwrap_or_default(),
                enabled: row.get::<_, i32>(4).unwrap_or(1) != 0,
                use_count: row.get::<_, i64>(5).unwrap_or(0) as u32,
                created_at: row.get(6).unwrap_or_default(),
            })
        })?.collect::<Result<Vec<_>, _>>()?;
        Ok(passwords)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn task_state_roundtrip() {
        let states = vec![
            TaskState::Queued,
            TaskState::Downloading,
            TaskState::Paused,
            TaskState::Seeding,
            TaskState::Done,
            TaskState::Error,
        ];
        for state in states {
            let s = state.as_str();
            let back = TaskState::from_str(s);
            assert_eq!(state, back, "roundtrip failed for {:?}", state);
        }
    }

    #[test]
    fn task_state_from_str_unknown_defaults_queued() {
        assert_eq!(TaskState::from_str("unknown"), TaskState::Queued);
        assert_eq!(TaskState::from_str(""), TaskState::Queued);
    }

    #[test]
    fn protocol_roundtrip() {
        let protos = vec![
            Protocol::Http,
            Protocol::Ftp,
            Protocol::Bt,
            Protocol::Magnet,
            Protocol::Ed2k,
            Protocol::Hls,
            Protocol::Dash,
        ];
        for proto in protos {
            let s = proto.as_str();
            let back = Protocol::from_str(s);
            assert_eq!(proto, back, "roundtrip failed for {:?}", proto);
        }
    }

    #[test]
    fn protocol_from_str_case_insensitive() {
        assert_eq!(Protocol::from_str("http"), Protocol::Http);
        assert_eq!(Protocol::from_str("Https"), Protocol::Http);
        assert_eq!(Protocol::from_str("ftp"), Protocol::Ftp);
        assert_eq!(Protocol::from_str("Ftps"), Protocol::Ftp);
    }

    #[test]
    fn protocol_from_str_unknown_defaults_http() {
        assert_eq!(Protocol::from_str("unknown"), Protocol::Http);
    }

    #[test]
    fn database_new_creates_tables() {
        let dir = std::env::temp_dir().join("db_test_new");
        let _ = std::fs::remove_file(&dir);
        let db = Database::new(&dir);
        assert!(db.is_ok(), "Database::new should succeed: {:?}", db.err());
        let _ = std::fs::remove_file(&dir);
    }

    #[test]
    fn database_get_db_size_returns_nonzero() {
        let dir = std::env::temp_dir().join("db_test_size");
        let _ = std::fs::remove_file(&dir);
        let db = Database::new(&dir).unwrap();
        let (size, pages) = db.get_db_size().unwrap();
        assert!(pages > 0, "page count should be > 0");
        assert!(size > 0, "db size should be > 0");
        let _ = std::fs::remove_file(&dir);
    }

    #[test]
    fn insert_and_get_task() {
        let dir = std::env::temp_dir().join("db_test_insert_get");
        let _ = std::fs::remove_file(&dir);
        let db = Database::new(&dir).unwrap();
        let task = TaskStatus {
            id: "test-1".into(), name: "test.zip".into(), protocol: Protocol::Http,
            state: TaskState::Downloading, url: "http://example.com".into(),
            save_path: "/tmp".into(), total_size: 1024, downloaded: 512,
            uploaded: 0, download_speed: 100, upload_speed: 0, progress: 0.5,
            peers: 0, seeds: 0, eta: None, files: vec![], error: None,
            added_at: "2026-01-01 00:00:00".into(), completed_at: None,
            priority: 1, download_limit: 0, upload_limit: 0,
        };
        db.insert_task(&task).unwrap();
        let loaded = db.get_task("test-1").unwrap().unwrap();
        assert_eq!(loaded.id, "test-1");
        assert_eq!(loaded.name, "test.zip");
        assert_eq!(loaded.total_size, 1024);
        let _ = std::fs::remove_file(&dir);
    }

    #[test]
    fn update_task_state_and_progress() {
        let dir = std::env::temp_dir().join("db_test_update");
        let _ = std::fs::remove_file(&dir);
        let db = Database::new(&dir).unwrap();
        let task = TaskStatus {
            id: "test-2".into(), name: "test.zip".into(), protocol: Protocol::Http,
            state: TaskState::Downloading, url: "http://example.com".into(),
            save_path: "/tmp".into(), total_size: 1024, downloaded: 0,
            uploaded: 0, download_speed: 0, upload_speed: 0, progress: 0.0,
            peers: 0, seeds: 0, eta: None, files: vec![], error: None,
            added_at: "2026-01-01 00:00:00".into(), completed_at: None,
            priority: 1, download_limit: 0, upload_limit: 0,
        };
        db.insert_task(&task).unwrap();
        db.update_task_state("test-2", &TaskState::Paused).unwrap();
        db.update_task_progress_full("test-2", 512, 1024).unwrap();
        let loaded = db.get_task("test-2").unwrap().unwrap();
        assert_eq!(loaded.state, TaskState::Paused);
        assert_eq!(loaded.downloaded, 512);
        let _ = std::fs::remove_file(&dir);
    }

    #[test]
    fn delete_task() {
        let dir = std::env::temp_dir().join("db_test_delete");
        let _ = std::fs::remove_file(&dir);
        let db = Database::new(&dir).unwrap();
        let task = TaskStatus {
            id: "test-3".into(), name: "test.zip".into(), protocol: Protocol::Http,
            state: TaskState::Done, url: "http://example.com".into(),
            save_path: "/tmp".into(), total_size: 1024, downloaded: 1024,
            uploaded: 0, download_speed: 0, upload_speed: 0, progress: 1.0,
            peers: 0, seeds: 0, eta: None, files: vec![], error: None,
            added_at: "2026-01-01 00:00:00".into(), completed_at: Some("2026-01-01 01:00:00".into()),
            priority: 1, download_limit: 0, upload_limit: 0,
        };
        db.insert_task(&task).unwrap();
        assert!(db.get_task("test-3").unwrap().is_some());
        db.delete_task("test-3").unwrap();
        assert!(db.get_task("test-3").unwrap().is_none());
        let _ = std::fs::remove_file(&dir);
    }

    #[test]
    fn get_all_tasks_returns_all() {
        let dir = std::env::temp_dir().join("db_test_all");
        let _ = std::fs::remove_file(&dir);
        let db = Database::new(&dir).unwrap();
        for i in 0..3 {
            let task = TaskStatus {
                id: format!("test-{}", i), name: format!("file{}.zip", i), protocol: Protocol::Http,
                state: TaskState::Downloading, url: "http://example.com".into(),
                save_path: "/tmp".into(), total_size: 100, downloaded: 0,
                uploaded: 0, download_speed: 0, upload_speed: 0, progress: 0.0,
                peers: 0, seeds: 0, eta: None, files: vec![], error: None,
                added_at: "2026-01-01 00:00:00".into(), completed_at: None,
                priority: 1, download_limit: 0, upload_limit: 0,
            };
            db.insert_task(&task).unwrap();
        }
        let all = db.get_all_tasks().unwrap();
        assert_eq!(all.len(), 3);
        let _ = std::fs::remove_file(&dir);
    }

    #[test]
    fn batch_update_state() {
        let dir = std::env::temp_dir().join("db_test_batch");
        let _ = std::fs::remove_file(&dir);
        let db = Database::new(&dir).unwrap();
        for i in 0..3 {
            let task = TaskStatus {
                id: format!("batch-{}", i), name: format!("f{}.zip", i), protocol: Protocol::Http,
                state: TaskState::Downloading, url: "http://example.com".into(),
                save_path: "/tmp".into(), total_size: 100, downloaded: 0,
                uploaded: 0, download_speed: 0, upload_speed: 0, progress: 0.0,
                peers: 0, seeds: 0, eta: None, files: vec![], error: None,
                added_at: "2026-01-01 00:00:00".into(), completed_at: None,
                priority: 1, download_limit: 0, upload_limit: 0,
            };
            db.insert_task(&task).unwrap();
        }
        let ids: Vec<String> = (0..3).map(|i| format!("batch-{}", i)).collect();
        let updated = db.batch_update_state(&ids, &TaskState::Paused).unwrap();
        assert_eq!(updated, 3);
        let all = db.get_all_tasks().unwrap();
        assert!(all.iter().all(|t| t.state == TaskState::Paused));
        let _ = std::fs::remove_file(&dir);
    }

    #[test]
    fn record_and_get_download_history() {
        let dir = std::env::temp_dir().join("db_test_history");
        let _ = std::fs::remove_file(&dir);
        let db = Database::new(&dir).unwrap();
        db.record_download_history("hist-1", "test.zip", "http://example.com", "HTTP", "/tmp", 1024, 1024, 500, 60).unwrap();
        let history = db.get_download_history(10).unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].name, "test.zip");
        let _ = std::fs::remove_file(&dir);
    }

    #[test]
    fn mirror_url_operations() {
        let dir = std::env::temp_dir().join("db_test_mirror");
        let _ = std::fs::remove_file(&dir);
        let db = Database::new(&dir).unwrap();
        let task = TaskStatus {
            id: "mirror-1".into(), name: "test.zip".into(), protocol: Protocol::Http,
            state: TaskState::Downloading, url: "http://example.com".into(),
            save_path: "/tmp".into(), total_size: 1024, downloaded: 0,
            uploaded: 0, download_speed: 0, upload_speed: 0, progress: 0.0,
            peers: 0, seeds: 0, eta: None, files: vec![], error: None,
            added_at: "2026-01-01 00:00:00".into(), completed_at: None,
            priority: 1, download_limit: 0, upload_limit: 0,
        };
        db.insert_task(&task).unwrap();
        db.add_mirror_url("mirror-1", "http://mirror1.com/file.zip").unwrap();
        db.add_mirror_url("mirror-1", "http://mirror2.com/file.zip").unwrap();
        let urls = db.get_mirror_urls("mirror-1").unwrap();
        assert_eq!(urls.len(), 2);
        db.remove_mirror_url("mirror-1", "http://mirror1.com/file.zip").unwrap();
        let urls = db.get_mirror_urls("mirror-1").unwrap();
        assert_eq!(urls.len(), 1);
        let _ = std::fs::remove_file(&dir);
    }
}
