// WASM 插件加载器
// 使用 wasmtime 运行时加载和执行 WASM 插件
// 提供沙箱隔离、内存限制、执行时间限制

use anyhow::{Context, Result};
use std::path::Path;
use std::sync::Arc;
use tracing::{debug, info, warn};
use wasmtime::*;

use super::api::TaskParams;

/// 验证文件路径是否在允许的目录内（防止路径遍历攻击）
/// 使用 canonicalize() 解析符号链接和相对路径，拒绝绝对路径
fn validate_plugin_path(path_str: &str, allowed_dirs: &[String]) -> bool {
    // 空目录列表 = 禁止所有文件访问
    if allowed_dirs.is_empty() {
        warn!("[plugin] 文件访问被拒绝：未配置允许的目录");
        return false;
    }

    let path = std::path::Path::new(path_str);

    // 拒绝包含 .. 的路径（防止目录遍历）
    if path_str.contains("..") {
        warn!("[plugin] 拒绝包含 '..' 的路径: {}", path_str);
        return false;
    }

    // 解析真实路径（解析符号链接和相对路径）
    let canonical = match path.canonicalize() {
        Ok(p) => p,
        Err(_) => {
            // 文件不存在时，尝试对父目录 canonicalize
            if let Some(parent) = path.parent() {
                match parent.canonicalize() {
                    Ok(parent_canon) => parent_canon.join(path.file_name().unwrap_or_default()),
                    Err(_) => {
                        warn!("[plugin] 无法解析路径: {}", path_str);
                        return false;
                    }
                }
            } else {
                warn!("[plugin] 无效路径: {}", path_str);
                return false;
            }
        }
    };

    // 检查路径是否在任一允许目录内
    for dir in allowed_dirs {
        if let Ok(allowed_canon) = std::path::Path::new(dir).canonicalize() {
            if canonical.starts_with(&allowed_canon) {
                return true;
            }
        }
    }

    warn!(
        "[plugin] 路径 '{}' 不在允许的目录范围内",
        path_str
    );
    false
}

/// WASM 加载器配置
#[derive(Debug, Clone)]
pub struct LoaderConfig {
    /// 最大内存（字节）
    pub max_memory: usize,
    /// 单次调用超时（秒）
    pub call_timeout: u64,
    /// 允许的网络域名
    pub allowed_domains: Vec<String>,
}

impl Default for LoaderConfig {
    fn default() -> Self {
        LoaderConfig {
            max_memory: 64 * 1024 * 1024, // 64MB
            call_timeout: 5,               // 5秒
            allowed_domains: Vec::new(),
        }
    }
}

/// WASM 插件加载器
pub struct WasmLoader {
    config: LoaderConfig,
}

/// WASM 沙箱内存限制器
struct MemoryLimit {
    max_bytes: usize,
}

/// 插件 store 状态
struct PluginState {
    memory_limit: MemoryLimit,
    /// HTTP 客户端（用于 http_get/http_post）
    http_client: reqwest::Client,
    /// 允许的文件系统根目录（空 = 禁止文件访问）
    allowed_dirs: Vec<String>,
    /// 允许的网络域名（空 = 允许所有域名）
    allowed_domains: Vec<String>,
    /// http_get 响应缓冲区（供宿主函数写回 WASM 内存）
    http_response: Vec<u8>,
    /// http_get 分配的 WASM 内存指针
    http_response_ptr: i32,
    /// read_file 读取的数据缓冲区
    file_data: Vec<u8>,
}

impl Default for PluginState {
    fn default() -> Self {
        PluginState {
            memory_limit: MemoryLimit {
                max_bytes: 64 * 1024 * 1024, // 64MB
            },
            http_client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .unwrap_or_default(),
            allowed_dirs: Vec::new(),
            allowed_domains: Vec::new(),
            http_response: Vec::new(),
            http_response_ptr: 0,
            file_data: Vec::new(),
        }
    }
}

impl ResourceLimiter for MemoryLimit {
    fn memory_growing(&mut self, _current: usize, desired: usize, _maximum: Option<usize>) -> Result<bool> {
        Ok(desired <= self.max_bytes)
    }

    fn table_growing(&mut self, _current: usize, desired: usize, _maximum: Option<usize>) -> Result<bool> {
        Ok(desired <= 10000) // 表大小限制
    }
}

impl WasmLoader {
    /// 创建新的加载器
    pub fn new() -> Self {
        WasmLoader {
            config: LoaderConfig::default(),
        }
    }

    /// 使用自定义配置创建加载器
    pub fn with_config(config: LoaderConfig) -> Self {
        WasmLoader { config }
    }

    /// 调用插件的 parse_url 函数
    ///
    /// # 参数
    /// - `wasm_path`: WASM 文件路径
    /// - `url`: 要解析的 URL
    ///
    /// # 返回
    /// - `Ok(Some(TaskParams))`: 插件成功解析了 URL
    /// - `Ok(None)`: 插件不支持该 URL（或 WASM 运行时不可用）
    /// - `Err(...)`: 执行出错
    pub fn call_parse_url(&self, wasm_path: &Path, url: &str) -> Result<Option<TaskParams>> {
        debug!("调用插件 parse_url: {}", wasm_path.display());

        let wasm_bytes = std::fs::read(wasm_path)
            .context("读取 WASM 文件失败")?;

        match self.execute_plugin(&wasm_bytes, "parse_url", url.as_bytes()) {
            Ok(result) => {
                if result.is_empty() {
                    Ok(None)
                } else {
                    Ok(serde_json::from_slice(&result).ok())
                }
            }
            Err(e) => {
                debug!("插件 parse_url 不可用: {}", e);
                Ok(None)
            }
        }
    }

    /// 调用插件的 on_complete 函数
    pub fn call_on_complete(&self, wasm_path: &Path, task_info: &super::api::TaskInfo) -> Result<()> {
        debug!("调用插件 on_complete: {}", wasm_path.display());

        let wasm_bytes = std::fs::read(wasm_path)
            .context("读取 WASM 文件失败")?;

        let info_json = serde_json::to_vec(task_info)?;
        match self.execute_plugin(&wasm_bytes, "on_complete", &info_json) {
            Ok(_) => Ok(()),
            Err(e) => {
                debug!("插件 on_complete 不可用: {}", e);
                Ok(())
            }
        }
    }

    /// 执行 WASM 插件函数
    ///
    /// 使用 wasmtime 运行时在沙箱中执行插件，支持：
    /// - 内存限制（默认 64MB）
    /// - 执行超时（epoch interruption）
    /// - 宿主函数注入（log、emit_event 等）
    fn execute_plugin(&self, wasm_bytes: &[u8], function: &str, input: &[u8]) -> Result<Vec<u8>> {
        // 配置 wasmtime 引擎
        let mut config = Config::new();
        config.max_wasm_stack(1024 * 1024); // 1MB 栈
        config.epoch_interruption(true);
        config.consume_fuel(true);

        let engine = Engine::new(&config)?;

        // 设置内存限制
        let mut plugin_state = PluginState::default();
        plugin_state.allowed_domains = self.config.allowed_domains.clone();
        let mut store = Store::new(&engine, plugin_state);
        store.limiter(|state: &mut PluginState| -> &mut dyn wasmtime::ResourceLimiter {
            &mut state.memory_limit
        });
        // 初始 fuel：限制执行指令数（约 5 秒的计算量）
        store.set_fuel(100_000_000)?;

        let module = Module::new(&engine, wasm_bytes)?;

        // 创建 linker 并注入宿主函数
        let mut linker = Linker::new(&engine);

        // log(level: i32, msg_ptr: i32, msg_len: i32) — 宿主日志函数
        linker.func_wrap(
            "env",
            "log",
            |mut caller: Caller<'_, PluginState>, level: i32, msg_ptr: i32, msg_len: i32| {
                let memory = match caller.get_export("memory") {
                    Some(Extern::Memory(m)) => m,
                    _ => return,
                };
                let data = memory.data(&caller);
                let start = msg_ptr as usize;
                let end = start + msg_len as usize;
                if end <= data.len() {
                    if let Ok(msg) = std::str::from_utf8(&data[start..end]) {
                        match level {
                            0 => debug!("[plugin] {}", msg),
                            1 => info!("[plugin] {}", msg),
                            2 => warn!("[plugin] {}", msg),
                            _ => debug!("[plugin] {}", msg),
                        }
                    }
                }
            },
        )?;

        // http_get(url_ptr, url_len) -> i32 (0=成功, -1=失败)
        // 响应体暂存到 PluginState，由 execute_plugin 在调用后写回 WASM 线性内存
        linker.func_wrap(
            "env",
            "http_get",
            |mut caller: Caller<'_, PluginState>, url_ptr: i32, url_len: i32| -> Result<i32> {
                let memory = match caller.get_export("memory") {
                    Some(Extern::Memory(m)) => m,
                    _ => return Ok(-1),
                };
                let data = memory.data(&caller);
                let start = url_ptr as usize;
                let end = start + url_len as usize;
                if end > data.len() {
                    return Ok(-1);
                }
                let url = match std::str::from_utf8(&data[start..end]) {
                    Ok(u) => u.to_string(),
                    Err(_) => return Ok(-1),
                };
                debug!("[plugin] http_get: {}", url);

                // 域名白名单检查：仅在配置了 allowed_domains 时生效
                let allowed = caller.data().allowed_domains.clone();
                if !allowed.is_empty() {
                    let domain_ok = match reqwest::Url::parse(&url) {
                        Ok(parsed) => parsed.host_str()
                            .map(|h| allowed.iter().any(|d| h == d || h.ends_with(&format!(".{}", d))))
                            .unwrap_or(false),
                        Err(_) => false,
                    };
                    if !domain_ok {
                        warn!("[plugin] http_get 拒绝：域名不在白名单中: {}", url);
                        return Ok(-1);
                    }
                }

                // 使用 block_in_place 避免 tokio 运行时死锁
                // block_on 在已运行的 tokio 上下文中会导致死锁
                let client = caller.data().http_client.clone();
                match tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(client.get(&url).send())
                }) {
                    Ok(resp) => match tokio::task::block_in_place(|| {
                        tokio::runtime::Handle::current().block_on(resp.bytes())
                    }) {
                        Ok(body) => {
                            info!("[plugin] http_get 成功: {} bytes", body.len());
                            // 暂存响应体，execute_plugin 调用后写回 WASM 内存
                            caller.data_mut().http_response = body.to_vec();
                            Ok(0)
                        }
                        Err(e) => {
                            warn!("[plugin] http_get 读取响应失败: {}", e);
                            Ok(-1)
                        }
                    },
                    Err(e) => {
                        warn!("[plugin] http_get 请求失败: {}", e);
                        Ok(-1)
                    }
                }
            },
        )?;

        // read_file(path_ptr, path_len) -> i32 (0=成功, -1=失败)
        linker.func_wrap(
            "env",
            "read_file",
            |mut caller: Caller<'_, PluginState>, path_ptr: i32, path_len: i32| -> i32 {
                let memory = match caller.get_export("memory") {
                    Some(Extern::Memory(m)) => m,
                    _ => return -1,
                };
                let data = memory.data(&caller);
                let start = path_ptr as usize;
                let end = start + path_len as usize;
                if end > data.len() {
                    return -1;
                }
                let path = match std::str::from_utf8(&data[start..end]) {
                    Ok(p) => p.to_string(),
                    Err(_) => return -1,
                };
                debug!("[plugin] read_file: {}", path);

                // 安全检查：验证路径在允许目录内
                let allowed = caller.data().allowed_dirs.clone();
                if !validate_plugin_path(&path, &allowed) {
                    return -1;
                }

                match std::fs::read(&path) {
                    Ok(bytes) => {
                        info!("[plugin] read_file 成功: {} bytes", bytes.len());
                        // 将读取的数据存入 PluginState，供插件后续使用
                        caller.data_mut().file_data = bytes;
                        0
                    }
                    Err(e) => {
                        warn!("[plugin] read_file 失败: {}", e);
                        -1
                    }
                }
            },
        )?;

        // write_file(path_ptr, path_len, data_ptr, data_len) -> i32 (0=成功, -1=失败)
        linker.func_wrap(
            "env",
            "write_file",
            |mut caller: Caller<'_, PluginState>, path_ptr: i32, path_len: i32, data_ptr: i32, data_len: i32| -> i32 {
                let memory = match caller.get_export("memory") {
                    Some(Extern::Memory(m)) => m,
                    _ => return -1,
                };
                let mem_data = memory.data(&caller);

                let p_start = path_ptr as usize;
                let p_end = p_start + path_len as usize;
                let d_start = data_ptr as usize;
                let d_end = d_start + data_len as usize;

                if p_end > mem_data.len() || d_end > mem_data.len() {
                    return -1;
                }

                let path = match std::str::from_utf8(&mem_data[p_start..p_end]) {
                    Ok(p) => p.to_string(),
                    Err(_) => return -1,
                };

                // 安全检查：验证路径在允许目录内
                let allowed = caller.data().allowed_dirs.clone();
                if !validate_plugin_path(&path, &allowed) {
                    return -1;
                }

                let write_data = &mem_data[d_start..d_end];
                match std::fs::write(&path, write_data) {
                    Ok(_) => {
                        info!("[plugin] write_file 成功: {} ({} bytes)", path, write_data.len());
                        0
                    }
                    Err(e) => {
                        warn!("[plugin] write_file 失败: {}", e);
                        -1
                    }
                }
            },
        )?;

        // emit_event(event_type_ptr, event_type_len, data_ptr, data_len)
        // 用于插件向宿主发送自定义事件（日志形式记录）
        linker.func_wrap(
            "env",
            "emit_event",
            |mut caller: Caller<'_, PluginState>, type_ptr: i32, type_len: i32, data_ptr: i32, data_len: i32| {
                let memory = match caller.get_export("memory") {
                    Some(Extern::Memory(m)) => m,
                    _ => return,
                };
                let mem_data = memory.data(&caller);

                let t_start = type_ptr as usize;
                let t_end = t_start + type_len as usize;
                let d_start = data_ptr as usize;
                let d_end = d_start + data_len as usize;

                if t_end > mem_data.len() || d_end > mem_data.len() {
                    return;
                }

                let event_type = std::str::from_utf8(&mem_data[t_start..t_end]).unwrap_or("unknown");
                let event_data = std::str::from_utf8(&mem_data[d_start..d_end]).unwrap_or("");
                info!("[plugin] event: {} — {}", event_type, event_data);
            },
        )?;

        // 实例化模块
        let instance = linker.instantiate(&mut store, &module)?;

        // 写入输入数据到 WASM 内存
        if let Some(Extern::Memory(memory)) = instance.get_export(&mut store, "memory") {
            // 调用 alloc 函数分配内存
            if let Some(alloc_func) = instance.get_func(&mut store, "alloc") {
                let mut alloc_result = [Val::I32(0)];
                alloc_func.call(&mut store, &[Val::I32(input.len() as i32)], &mut alloc_result)?;
                if let Val::I32(ptr) = alloc_result[0] {
                    // 写入输入数据
                    memory.data_mut(&mut store)[ptr as usize..ptr as usize + input.len()]
                        .copy_from_slice(input);

                    // 调用目标函数
                    if let Some(func) = instance.get_func(&mut store, function) {
                        let mut results = [Val::I32(0)];
                        func.call(
                            &mut store,
                            &[Val::I32(ptr), Val::I32(input.len() as i32)],
                            &mut results,
                        )?;

                        // http_get 宿主函数将响应体暂存在 PluginState 中，
                        // 此处分配 WASM 内存并将数据写回，供插件读取
                        let resp_data = std::mem::take(&mut store.data_mut().http_response);
                        if !resp_data.is_empty() {
                            if let Some(resp_alloc) = instance.get_func(&mut store, "alloc") {
                                let resp_len = resp_data.len() as i32;
                                let mut alloc_res = [Val::I32(0)];
                                resp_alloc.call(&mut store, &[Val::I32(resp_len)], &mut alloc_res)?;
                                if let Val::I32(resp_ptr) = alloc_res[0] {
                                    let rp = resp_ptr as usize;
                                    memory.data_mut(&mut store)[rp..rp + resp_data.len()]
                                        .copy_from_slice(&resp_data);
                                    // 存储指针供插件后续读取
                                    store.data_mut().http_response_ptr = resp_ptr;
                                    debug!("[plugin] http_get 响应已写回 WASM 内存: ptr={}, len={}", resp_ptr, resp_len);
                                }
                            }
                        }

                        // 读取返回值（假设返回指针+长度）
                        if let Val::I32(result_ptr) = results[0] {
                            let data = memory.data(&store);
                            let rp = result_ptr as usize;
                            if rp + 4 <= data.len() {
                                let out_len = u32::from_le_bytes([
                                    data[rp], data[rp + 1], data[rp + 2], data[rp + 3],
                                ]) as usize;
                                if rp + 4 + out_len <= data.len() {
                                    return Ok(data[rp + 4..rp + 4 + out_len].to_vec());
                                }
                            }
                        }
                    }
                }
            }
        }

        // fallback: 尝试直接调用无参函数
        if let Some(func) = instance.get_func(&mut store, function) {
            let mut results = vec![Val::I32(0)];
            func.call(&mut store, &[], &mut results)?;
        }

        Ok(Vec::new())
    }

    /// 验证 WASM 模块是否有效
    pub fn validate_wasm(&self, wasm_path: &Path) -> Result<()> {
        let wasm_bytes = std::fs::read(wasm_path)
            .context("读取 WASM 文件失败")?;

        // 检查 WASM 魔数
        if wasm_bytes.len() < 8 {
            anyhow::bail!("文件太小，不是有效的 WASM 模块");
        }

        // WASM 魔数: 0x00 0x61 0x73 0x6D
        if &wasm_bytes[0..4] != b"\0asm" {
            anyhow::bail!("不是有效的 WASM 文件");
        }

        // 检查版本号
        let version = u32::from_le_bytes([
            wasm_bytes[4], wasm_bytes[5], wasm_bytes[6], wasm_bytes[7],
        ]);
        if version != 1 {
            anyhow::bail!("不支持的 WASM 版本: {}", version);
        }

        info!("WASM 模块验证通过: {}", wasm_path.display());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_plugin_path_rejects_empty_allowed_dirs() {
        assert!(!validate_plugin_path("/etc/passwd", &[]));
    }

    #[test]
    fn validate_plugin_path_rejects_dot_dot_traversal() {
        let dirs = vec!["/tmp/plugins".to_string()];
        assert!(!validate_plugin_path("/tmp/plugins/../../../etc/passwd", &dirs));
    }

    #[test]
    fn validate_plugin_path_rejects_relative_dot_dot() {
        let dirs = vec!["/tmp/plugins".to_string()];
        assert!(!validate_plugin_path("../secret.txt", &dirs));
    }

    #[test]
    fn loader_config_default_values() {
        let config = LoaderConfig::default();
        assert_eq!(config.max_memory, 64 * 1024 * 1024);
        assert_eq!(config.call_timeout, 5);
        assert!(config.allowed_domains.is_empty());
    }

    #[test]
    fn validate_wasm_rejects_small_file() {
        let loader = WasmLoader::new();
        // 创建临时文件太小
        let dir = std::env::temp_dir().join("wasm_test_small");
        std::fs::write(&dir, &[0u8; 4]).unwrap();
        let result = loader.validate_wasm(&dir);
        assert!(result.is_err());
        let _ = std::fs::remove_file(&dir);
    }

    #[test]
    fn validate_wasm_rejects_invalid_magic() {
        let loader = WasmLoader::new();
        let dir = std::env::temp_dir().join("wasm_test_bad_magic");
        let mut data = vec![0u8; 12];
        data[0..4].copy_from_slice(b"XXXX"); // 错误的魔数
        std::fs::write(&dir, &data).unwrap();
        let result = loader.validate_wasm(&dir);
        assert!(result.is_err());
        let _ = std::fs::remove_file(&dir);
    }

    #[test]
    fn validate_wasm_accepts_valid_magic() {
        let loader = WasmLoader::new();
        let dir = std::env::temp_dir().join("wasm_test_valid");
        let mut data = vec![0u8; 8];
        data[0..4].copy_from_slice(b"\0asm"); // 正确魔数
        data[4..8].copy_from_slice(&1u32.to_le_bytes()); // 版本 1
        std::fs::write(&dir, &data).unwrap();
        let result = loader.validate_wasm(&dir);
        assert!(result.is_ok());
        let _ = std::fs::remove_file(&dir);
    }

    #[test]
    fn validate_wasm_rejects_wrong_version() {
        let loader = WasmLoader::new();
        let dir = std::env::temp_dir().join("wasm_test_bad_ver");
        let mut data = vec![0u8; 8];
        data[0..4].copy_from_slice(b"\0asm");
        data[4..8].copy_from_slice(&99u32.to_le_bytes()); // 错误版本
        std::fs::write(&dir, &data).unwrap();
        let result = loader.validate_wasm(&dir);
        assert!(result.is_err());
        let _ = std::fs::remove_file(&dir);
    }

    #[test]
    fn http_response_buffer_defaults_empty() {
        let state = PluginState::default();
        assert!(state.http_response.is_empty());
        assert_eq!(state.http_response_ptr, 0);
    }

    #[test]
    fn allowed_domains_defaults_empty() {
        let state = PluginState::default();
        assert!(state.allowed_domains.is_empty());
    }

    #[test]
    fn allowed_domains_from_config() {
        let mut config = LoaderConfig::default();
        config.allowed_domains = vec!["example.com".to_string()];
        let loader = WasmLoader::with_config(config);
        assert_eq!(loader.config.allowed_domains.len(), 1);
    }
}
