// WASM 插件系统模块
// 提供标准化插件 API，支持插件加载/卸载/管理
// 插件运行在 WASM 沙箱中，无法直接访问系统

pub mod api;
pub mod loader;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tracing::{info, warn};

/// 插件状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PluginState {
    /// 已安装
    Installed,
    /// 已启用
    Enabled,
    /// 已禁用
    Disabled,
    /// 加载错误
    Error(String),
}

/// 插件信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    /// 唯一标识
    pub id: String,
    /// 插件名称
    pub name: String,
    /// 版本号
    pub version: String,
    /// 插件描述
    pub description: String,
    /// 作者
    pub author: String,
    /// 支持的协议
    pub supported_protocols: Vec<String>,
    /// 插件类型
    pub plugin_type: PluginType,
    /// 所需权限
    pub permissions: Vec<PluginPermission>,
    /// 状态
    pub state: PluginState,
    /// WASM 文件路径
    pub wasm_path: PathBuf,
}

/// 插件类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PluginType {
    /// 下载引擎插件
    Engine,
    /// 解析器插件（解析新的链接格式）
    Parser,
    /// 通知插件
    Notification,
    /// 后处理插件（下载完成后执行）
    PostProcess,
}

/// 插件权限
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PluginPermission {
    /// 网络请求
    Network,
    /// 文件读取（限制范围）
    FileSystemRead,
    /// 文件写入（限制范围）
    FileSystemWrite,
    /// 执行外部命令
    Process,
}

/// 插件配置项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginSetting {
    /// 配置键
    pub key: String,
    /// 配置名称
    pub label: String,
    /// 配置类型（string/number/boolean）
    pub setting_type: String,
    /// 默认值
    pub default_value: serde_json::Value,
    /// 当前值
    pub value: Option<serde_json::Value>,
    /// 描述
    pub description: String,
}

/// 插件管理器
pub struct PluginManager {
    /// 已安装的插件
    plugins: HashMap<String, PluginInfo>,
    /// 插件目录
    plugin_dir: PathBuf,
    /// WASM 加载器
    loader: loader::WasmLoader,
}

impl PluginManager {
    /// 创建新的插件管理器
    pub fn new(plugin_dir: PathBuf) -> Self {
        info!("插件管理器初始化: 目录={}", plugin_dir.display());

        PluginManager {
            plugins: HashMap::new(),
            plugin_dir,
            loader: loader::WasmLoader::new(),
        }
    }

    /// 扫描插件目录，加载已安装的插件
    pub fn scan_plugins(&mut self) -> Result<()> {
        if !self.plugin_dir.exists() {
            std::fs::create_dir_all(&self.plugin_dir)?;
        }

        for entry in std::fs::read_dir(&self.plugin_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().map_or(false, |ext| ext == "wasm") {
                match self.load_plugin_manifest(&path) {
                    Ok(info) => {
                        info!("发现插件: {} v{}", info.name, info.version);
                        self.plugins.insert(info.id.clone(), info);
                    }
                    Err(e) => {
                        warn!("加载插件失败 {}: {}", path.display(), e);
                    }
                }
            }
        }

        info!("共发现 {} 个插件", self.plugins.len());
        Ok(())
    }

    /// 安装插件
    pub fn install_plugin(&mut self, wasm_path: &Path) -> Result<String> {
        let info = self.load_plugin_manifest(wasm_path)?;

        // 复制到插件目录
        let dest = self.plugin_dir.join(format!("{}.wasm", info.id));
        std::fs::copy(wasm_path, &dest)?;

        let id = info.id.clone();
        self.plugins.insert(id.clone(), info);
        info!("插件已安装: {}", id);

        Ok(id)
    }

    /// 卸载插件
    pub fn uninstall_plugin(&mut self, id: &str) -> Result<()> {
        if let Some(info) = self.plugins.remove(id) {
            let wasm_path = self.plugin_dir.join(format!("{}.wasm", id));
            if wasm_path.exists() {
                std::fs::remove_file(wasm_path)?;
            }
            info!("插件已卸载: {}", info.name);
        }
        Ok(())
    }

    /// 启用插件
    pub fn enable_plugin(&mut self, id: &str) -> Result<()> {
        if let Some(info) = self.plugins.get_mut(id) {
            info.state = PluginState::Enabled;
            info!("插件已启用: {}", info.name);
        }
        Ok(())
    }

    /// 禁用插件
    pub fn disable_plugin(&mut self, id: &str) -> Result<()> {
        if let Some(info) = self.plugins.get_mut(id) {
            info.state = PluginState::Disabled;
            info!("插件已禁用: {}", info.name);
        }
        Ok(())
    }

    /// 获取所有插件列表
    pub fn list_plugins(&self) -> Vec<&PluginInfo> {
        self.plugins.values().collect()
    }

    /// 获取指定插件
    pub fn get_plugin(&self, id: &str) -> Option<&PluginInfo> {
        self.plugins.get(id)
    }

    /// 加载插件清单
    /// 尝试从 WASM 自定义 section "plugin_manifest" 中解析 JSON 清单
    /// 如果没有自定义 section，则从文件名推断基本信息
    fn load_plugin_manifest(&self, wasm_path: &Path) -> Result<PluginInfo> {
        let wasm_bytes = std::fs::read(wasm_path)?;
        let filename = wasm_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");

        // 尝试解析 WASM 自定义 section
        if let Some(manifest_json) = extract_wasm_custom_section(&wasm_bytes, "plugin_manifest") {
            if let Ok(info) = serde_json::from_str::<PluginManifest>(&manifest_json) {
                return Ok(PluginInfo {
                    id: info.id.unwrap_or_else(|| filename.to_string()),
                    name: info.name.unwrap_or_else(|| filename.to_string()),
                    version: info.version.unwrap_or_else(|| "1.0.0".to_string()),
                    description: info.description.unwrap_or_default(),
                    author: info.author.unwrap_or_default(),
                    supported_protocols: info.supported_protocols.unwrap_or_default(),
                    plugin_type: match info.plugin_type.as_deref() {
                        Some("notification") => PluginType::Notification,
                        Some("post_process") => PluginType::PostProcess,
                        _ => PluginType::Engine,
                    },
                    permissions: info.permissions.unwrap_or_else(|| vec![PluginPermission::Network]),
                    state: PluginState::Installed,
                    wasm_path: wasm_path.to_path_buf(),
                });
            }
        }

        // 回退：从文件名推断
        Ok(PluginInfo {
            id: filename.to_string(),
            name: filename.to_string(),
            version: "1.0.0".to_string(),
            description: String::new(),
            author: String::new(),
            supported_protocols: Vec::new(),
            plugin_type: PluginType::Engine,
            permissions: vec![PluginPermission::Network],
            state: PluginState::Installed,
            wasm_path: wasm_path.to_path_buf(),
        })
    }

    /// 验证 WASM 插件文件是否有效
    pub fn validate_wasm_plugin(&self, wasm_path: &Path) -> Result<()> {
        self.loader.validate_wasm(wasm_path)
    }

    /// 执行插件的解析 URL 方法
    pub fn call_parse_url(&self, plugin_id: &str, url: &str) -> Result<Option<api::TaskParams>> {
        let info = self.plugins.get(plugin_id)
            .ok_or_else(|| anyhow::anyhow!("插件不存在: {}", plugin_id))?;

        if info.state != PluginState::Enabled {
            return Ok(None);
        }

        self.loader.call_parse_url(&info.wasm_path, url)
    }

    /// 执行插件的 on_complete 方法
    pub fn call_on_complete(&self, plugin_id: &str, task_info: &api::TaskInfo) -> Result<()> {
        let info = self.plugins.get(plugin_id)
            .ok_or_else(|| anyhow::anyhow!("插件不存在: {}", plugin_id))?;

        if info.state != PluginState::Enabled {
            return Ok(());
        }

        self.loader.call_on_complete(&info.wasm_path, task_info)
    }
}

/// 插件清单（从 WASM 自定义 section 解析）
#[derive(Debug, Deserialize)]
struct PluginManifest {
    id: Option<String>,
    name: Option<String>,
    version: Option<String>,
    description: Option<String>,
    author: Option<String>,
    supported_protocols: Option<Vec<String>>,
    plugin_type: Option<String>,
    permissions: Option<Vec<PluginPermission>>,
}

/// 从 WASM 二进制中提取指定名称的自定义 section
///
/// WASM 文件格式：
/// - Magic: 4 bytes (\0asm)
/// - Version: 4 bytes
/// - Sections: [section_id, section_size, ...]
///   - section_id=0 为自定义 section
///   - 自定义 section: name_len(LEB128) + name(UTF-8) + data
fn extract_wasm_custom_section(wasm_bytes: &[u8], section_name: &str) -> Option<String> {
    if wasm_bytes.len() < 8 {
        return None;
    }

    // 跳过 header (magic + version)
    let mut offset = 8;

    while offset < wasm_bytes.len() {
        if offset + 1 >= wasm_bytes.len() {
            break;
        }

        let section_id = wasm_bytes[offset];
        offset += 1;

        // 读取 section size (LEB128)
        let (section_size, bytes_read) = read_leb128(wasm_bytes, offset)?;
        offset += bytes_read;

        if offset + section_size as usize > wasm_bytes.len() {
            break;
        }

        let section_end = offset + section_size as usize;

        // section_id=0 是自定义 section
        if section_id == 0 {
            if let Some(name) = read_wasm_name(wasm_bytes, offset) {
                let name_bytes_len = leb128_size(wasm_bytes, offset) + name.len();
                let data_start = offset + name_bytes_len;
                if data_start <= section_end {
                    let data = &wasm_bytes[data_start..section_end];
                    if name == section_name {
                        return std::str::from_utf8(data).ok().map(|s| s.to_string());
                    }
                }
            }
        }

        offset = section_end;
    }

    None
}

/// 读取 LEB128 编码的无符号整数
fn read_leb128(bytes: &[u8], offset: usize) -> Option<(u32, usize)> {
    let mut result: u32 = 0;
    let mut shift = 0;
    let mut pos = offset;

    loop {
        if pos >= bytes.len() {
            return None;
        }
        let byte = bytes[pos];
        result |= ((byte & 0x7F) as u32) << shift;
        pos += 1;
        if byte & 0x80 == 0 {
            return Some((result, pos - offset));
        }
        shift += 7;
        if shift >= 32 {
            return None;
        }
    }
}

/// 读取 WASM 名称（LEB128 长度 + UTF-8 字符串）
fn read_wasm_name(bytes: &[u8], offset: usize) -> Option<String> {
    let (len, bytes_read) = read_leb128(bytes, offset)?;
    let start = offset + bytes_read;
    let end = start + len as usize;
    if end > bytes.len() {
        return None;
    }
    std::str::from_utf8(&bytes[start..end]).ok().map(|s| s.to_string())
}

/// 获取 LEB128 编码的字节长度
fn leb128_size(bytes: &[u8], offset: usize) -> usize {
    let mut pos = offset;
    while pos < bytes.len() {
        if bytes[pos] & 0x80 == 0 {
            return pos - offset + 1;
        }
        pos += 1;
    }
    0
}
