// TS 分片合并模块
// 将多个 .ts 分片合并为完整的 mp4/mkv 文件
// 支持内置合并（二进制拼接）和可选 ffmpeg 合并

use anyhow::{Context, Result};
use std::path::Path;
use tokio::io::AsyncWriteExt;
use tracing::{debug, info};

/// 合并 TS 分片文件为单个文件
///
/// # 参数
/// - `segments`: 分片文件路径列表（按顺序）
/// - `output`: 输出文件路径
///
/// # 说明
/// 简单的二进制拼接方式，适用于大多数 HLS 流
/// 如果需要更精确的容器格式转换（如转为 mp4），需要使用 ffmpeg
pub async fn merge_ts_files(segments: &[std::path::PathBuf], output: &Path) -> Result<()> {
    if segments.is_empty() {
        anyhow::bail!("没有分片文件需要合并");
    }

    info!("开始合并 {} 个分片到: {}", segments.len(), output.display());

    // 确保输出目录存在
    if let Some(parent) = output.parent() {
        tokio::fs::create_dir_all(parent).await.ok();
    }

    let mut output_file = tokio::fs::File::create(output)
        .await
        .context("创建输出文件失败")?;

    let mut total_bytes: u64 = 0;

    for (i, segment_path) in segments.iter().enumerate() {
        let data = tokio::fs::read(segment_path)
            .await
            .context(format!("读取分片 {} 失败", segment_path.display()))?;

        output_file
            .write_all(&data)
            .await
            .context("写入输出文件失败")?;

        total_bytes += data.len() as u64;

        if (i + 1) % 100 == 0 {
            debug!("已合并 {}/{} 个分片", i + 1, segments.len());
        }
    }

    output_file.flush().await.ok();

    info!("分片合并完成: {} bytes", total_bytes);
    Ok(())
}

/// 使用 ffmpeg 合并 TS 分片为 MP4
///
/// # 参数
/// - `segments`: 分片文件路径列表
/// - `output`: 输出文件路径（.mp4）
///
/// # 说明
/// 需要系统安装 ffmpeg
/// 适用于需要精确容器格式转换的场景
#[cfg(feature = "ffmpeg")]
pub async fn merge_with_ffmpeg(segments: &[std::path::PathBuf], output: &Path) -> Result<()> {
    use tokio::process::Command;

    // 创建分片列表文件
    let list_file = output.with_extension("txt");
    let mut list_content = String::new();
    for segment in segments {
        list_content.push_str(&format!("file '{}'\n", segment.display()));
    }
    tokio::fs::write(&list_file, list_content).await?;

    // 调用 ffmpeg 合并
    let status = Command::new("ffmpeg")
        .args([
            "-f", "concat",
            "-safe", "0",
            "-i", &list_file.to_string_lossy(),
            "-c", "copy",
            "-y",
            &output.to_string_lossy(),
        ])
        .status()
        .await
        .context("执行 ffmpeg 失败")?;

    // 清理列表文件
    tokio::fs::remove_file(&list_file).await.ok();

    if !status.success() {
        anyhow::bail!("ffmpeg 合并失败");
    }

    info!("ffmpeg 合并完成: {}", output.display());
    Ok(())
}

/// 检测 ffmpeg 是否可用
pub async fn is_ffmpeg_available() -> bool {
    tokio::process::Command::new("ffmpeg")
        .args(["-version"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .await
        .map(|s| s.success())
        .unwrap_or(false)
}

/// 生成 ffmpeg 文件列表内容
pub fn generate_concat_list(segments: &[std::path::PathBuf]) -> String {
    segments
        .iter()
        .map(|p| format!("file '{}'", p.display()))
        .collect::<Vec<_>>()
        .join("\n")
}
