// 插件 API 定义模块
// 定义插件与宿主之间的接口规范
// 插件通过导出这些函数来实现功能

use serde::{Deserialize, Serialize};

/// 插件任务参数（从插件返回给宿主）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskParams {
    /// 下载 URL
    pub url: String,
    /// 文件名
    pub filename: String,
    /// 保存目录
    pub save_dir: Option<String>,
    /// 协议类型
    pub protocol: Option<String>,
    /// 附加参数
    pub extra: Option<serde_json::Value>,
}

/// 任务信息（宿主传递给插件）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskInfo {
    /// 任务 ID
    pub task_id: String,
    /// 文件名
    pub filename: String,
    /// 文件大小
    pub file_size: u64,
    /// 保存路径
    pub save_path: String,
    /// 下载 URL
    pub url: String,
    /// 协议类型
    pub protocol: String,
    /// 下载耗时（秒）
    pub duration: u64,
    /// 平均速度（bytes/sec）
    pub average_speed: u64,
}

/// 插件导出函数签名
///
/// # parse_url
/// 解析 URL，返回任务参数
/// - 输入: URL 字符串
/// - 输出: Option<TaskParams>
///
/// # on_complete
/// 下载完成后的回调
/// - 输入: TaskInfo
/// - 输出: 无
///
/// # on_error
/// 下载出错时的回调
/// - 输入: task_id, error_message
/// - 输出: 无
///
/// # get_settings
/// 获取插件配置项列表
/// - 输入: 无
/// - 输出: Vec<PluginSetting>
pub trait PluginApi {
    /// 解析 URL
    fn parse_url(&self, url: &str) -> Option<TaskParams>;

    /// 下载完成回调
    fn on_complete(&self, task_info: &TaskInfo);

    /// 下载出错回调
    fn on_error(&self, task_id: &str, error: &str);

    /// 获取配置项
    fn get_settings(&self) -> Vec<PluginSetting>;
}

/// 插件配置项定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginSetting {
    /// 配置键
    pub key: String,
    /// 显示名称
    pub label: String,
    /// 类型 (string/number/boolean/select)
    pub setting_type: String,
    /// 默认值
    pub default_value: serde_json::Value,
    /// 可选值（用于 select 类型）
    pub options: Option<Vec<serde_json::Value>>,
    /// 描述
    pub description: String,
}

/// 插件宿主提供的能力
///
/// 插件可以通过这些函数与宿主交互
pub trait HostApi {
    /// 发送 HTTP GET 请求
    fn http_get(&self, url: &str) -> Result<Vec<u8>, String>;

    /// 发送 HTTP POST 请求
    fn http_post(&self, url: &str, body: &[u8], content_type: &str) -> Result<Vec<u8>, String>;

    /// 读取文件
    fn read_file(&self, path: &str) -> Result<Vec<u8>, String>;

    /// 写入文件
    fn write_file(&self, path: &str, data: &[u8]) -> Result<(), String>;

    /// 记录日志
    fn log(&self, level: &str, message: &str);

    /// 获取配置值
    fn get_config(&self, key: &str) -> Option<String>;

    /// 获取临时目录
    fn temp_dir(&self) -> String;
}
