// 自动解压模块
// 下载完成后自动解压 .zip/.rar/.7z/.tar.gz 等格式
// 支持密码管理、递归解压、解压后删除

pub mod password;

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tracing::{info, warn};

/// 支持的压缩格式
#[derive(Debug, Clone, PartialEq)]
pub enum ArchiveFormat {
    Zip,
    Tar,
    TarGz,
    TarBz2,
    TarXz,
    Rar,
    SevenZ,
    Unknown,
}

/// 检测文件的压缩格式
pub fn detect_format(file_path: &str) -> ArchiveFormat {
    let lower = file_path.to_lowercase();

    if lower.ends_with(".zip") {
        ArchiveFormat::Zip
    } else if lower.ends_with(".tar.gz") || lower.ends_with(".tgz") {
        ArchiveFormat::TarGz
    } else if lower.ends_with(".tar.bz2") || lower.ends_with(".tbz2") {
        ArchiveFormat::TarBz2
    } else if lower.ends_with(".tar.xz") || lower.ends_with(".txz") {
        ArchiveFormat::TarXz
    } else if lower.ends_with(".tar") {
        ArchiveFormat::Tar
    } else if lower.ends_with(".rar") {
        ArchiveFormat::Rar
    } else if lower.ends_with(".7z") {
        ArchiveFormat::SevenZ
    } else {
        ArchiveFormat::Unknown
    }
}

/// 解压配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveConfig {
    /// 是否启用自动解压
    pub auto_extract: bool,
    /// 解压后是否删除原文件
    pub delete_after_extract: bool,
    /// 解压目标目录（空 = 同目录）
    pub extract_dir: String,
    /// 已保存的密码列表
    pub passwords: Vec<SavedPassword>,
}

/// 已保存的密码
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedPassword {
    /// 唯一 ID
    pub id: String,
    /// 密码名称
    pub name: String,
    /// 密码值
    pub password: String,
    /// 关联的域名/路径模式
    pub pattern: String,
    /// 是否启用
    pub enabled: bool,
    /// 使用次数
    #[serde(default)]
    pub use_count: u32,
    /// 创建时间
    #[serde(default)]
    pub created_at: String,
}

impl Default for ArchiveConfig {
    fn default() -> Self {
        ArchiveConfig {
            auto_extract: false,
            delete_after_extract: false,
            extract_dir: String::new(),
            passwords: Vec::new(),
        }
    }
}

/// 解压结果
#[derive(Debug)]
pub struct ExtractResult {
    /// 解压出的文件列表
    pub files: Vec<PathBuf>,
    /// 总解压大小
    pub total_size: u64,
    /// 是否成功
    pub success: bool,
    /// 错误信息
    pub error: Option<String>,
}

/// 校验解压路径是否在目标目录内（防止 Zip Slip 路径遍历攻击）
fn validate_safe_path(extract_dir: &Path, out_path: &Path) -> Result<PathBuf> {
    // 提取相对于 extract_dir 的路径部分
    let relative = match out_path.strip_prefix(extract_dir) {
        Ok(r) => r,
        Err(_) => {
            bail!(
                "安全校验失败: 解压路径 {:?} 不在目标目录 {:?} 下",
                out_path,
                extract_dir
            );
        }
    };

    // 拒绝任何包含 .. 的路径组件（防止目录遍历）
    for component in relative.components() {
        if let std::path::Component::ParentDir = component {
            bail!(
                "安全校验失败: 解压路径 {:?} 包含 '..' 遍历序列",
                out_path
            );
        }
    }

    // 规范化路径：解析 . 和 .. 组件
    let normalized = normalize_path(extract_dir.join(relative));

    // 双重校验：规范化后的路径必须仍在目标目录下
    let canonical_extract = normalize_path(extract_dir.to_path_buf());
    if !normalized.starts_with(&canonical_extract) {
        bail!(
            "安全校验失败: 解压路径 {:?} 超出目标目录 {:?}",
            out_path,
            extract_dir
        );
    }

    Ok(normalized)
}

/// 规范化路径：解析 . 和 .. 组件，不依赖文件系统存在性
fn normalize_path(path: PathBuf) -> PathBuf {
    let mut components = Vec::new();
    for comp in path.components() {
        match comp {
            std::path::Component::ParentDir => {
                // 只在有非根组件时弹出
                if let Some(last) = components.last() {
                    if !matches!(last, std::path::Component::RootDir) {
                        components.pop();
                    }
                }
            }
            std::path::Component::CurDir => {}
            other => components.push(other),
        }
    }
    components.iter().collect()
}

/// 解压管理器
pub struct ArchiveManager {
    config: ArchiveConfig,
}

impl ArchiveManager {
    /// 创建新的解压管理器
    pub fn new(config: ArchiveConfig) -> Self {
        ArchiveManager { config }
    }

    /// 自动解压文件
    pub async fn auto_extract(&self, file_path: &str) -> Result<ExtractResult> {
        let format = detect_format(file_path);

        if format == ArchiveFormat::Unknown {
            return Ok(ExtractResult {
                files: Vec::new(),
                total_size: 0,
                success: false,
                error: Some("不支持的压缩格式".to_string()),
            });
        }

        let path = Path::new(file_path);
        let extract_dir = if self.config.extract_dir.is_empty() {
            path.parent()
                .unwrap_or(Path::new("."))
                .to_path_buf()
        } else {
            PathBuf::from(&self.config.extract_dir)
        };

        // 尝试密码列表
        let passwords = self.get_passwords_for_file(file_path);

        info!("开始解压: {} (格式: {:?})", file_path, format);

        let result = match format {
            ArchiveFormat::Zip => {
                self.extract_zip(path, &extract_dir, &passwords).await
            }
            ArchiveFormat::Tar => {
                self.extract_tar(path, &extract_dir).await
            }
            ArchiveFormat::TarGz => {
                self.extract_tar_gz(path, &extract_dir).await
            }
            ArchiveFormat::TarBz2 => {
                self.extract_tar_bz2(path, &extract_dir).await
            }
            ArchiveFormat::TarXz => {
                self.extract_tar_xz(path, &extract_dir).await
            }
            ArchiveFormat::Rar => {
                self.extract_rar(path, &extract_dir, &passwords).await
            }
            ArchiveFormat::SevenZ => {
                self.extract_7z(path, &extract_dir).await
            }
            ArchiveFormat::Unknown => {
                Err(anyhow::anyhow!("不支持的压缩格式: {}", file_path))
            }
        };

        // 解压成功后删除原文件（如果配置了 delete_after_extract）
        if let Ok(ref r) = result {
            if r.success && self.config.delete_after_extract {
                if let Err(e) = tokio::fs::remove_file(path).await {
                    warn!("删除原压缩文件失败: {} - {}", file_path, e);
                } else {
                    info!("已删除原压缩文件: {}", file_path);
                }
            }
        }

        result
    }

    /// 解压 ZIP 文件
    async fn extract_zip(
        &self,
        path: &Path,
        extract_dir: &Path,
        passwords: &[String],
    ) -> Result<ExtractResult> {
        let path = path.to_path_buf();
        let extract_dir = extract_dir.to_path_buf();
        let passwords = passwords.to_vec();

        tokio::task::spawn_blocking(move || {
            let file = std::fs::File::open(&path).context("打开 ZIP 文件失败")?;
            let mut archive = zip::ZipArchive::new(file).context("解析 ZIP 文件失败")?;

            let mut files = Vec::new();
            let mut total_size = 0u64;

            let num_files = archive.len();
            for i in 0..num_files {
                // 尝试用密码解密（如果有密码的话）
                // 注意：by_index_decrypt 返回的 ZipFile 借用 archive，
                // 必须在下一次循环前释放
                let (raw_path, is_dir, file_size) = if !passwords.is_empty() {
                    let mut result = None;
                    for password in &passwords {
                        if let Ok(Ok(mut f)) = archive.by_index_decrypt(i, password.as_bytes()) {
                            let path = f.mangled_name();
                            let dir = f.is_dir();
                            let size = f.size();
                            // 如果是文件，立即读取内容
                            if !dir {
                                let out_path = match validate_safe_path(&extract_dir, &extract_dir.join(&path)) {
                                    Ok(p) => p,
                                    Err(e) => {
                                        warn!("跳过不安全的 ZIP 条目 '{}': {}", path.display(), e);
                                        break;
                                    }
                                };
                                if let Some(parent) = out_path.parent() {
                                    std::fs::create_dir_all(parent).ok();
                                }
                                let mut out_file = std::fs::File::create(&out_path)?;
                                std::io::copy(&mut f, &mut out_file)?;
                                files.push(out_path);
                                total_size += size;
                            } else {
                                let out_path = validate_safe_path(&extract_dir, &extract_dir.join(&path))
                                    .unwrap_or_else(|_| extract_dir.join(&path));
                                std::fs::create_dir_all(&out_path).ok();
                            }
                            result = Some((path, dir, size));
                            break;
                        }
                    }
                    match result {
                        Some(r) => r,
                        None => continue,
                    }
                } else {
                    let mut f = archive.by_index(i).context("读取 ZIP 条目失败")?;
                    let path = f.mangled_name();
                    let dir = f.is_dir();
                    let size = f.size();
                    if !dir {
                        let out_path = match validate_safe_path(&extract_dir, &extract_dir.join(&path)) {
                            Ok(p) => p,
                            Err(e) => {
                                warn!("跳过不安全的 ZIP 条目 '{}': {}", path.display(), e);
                                continue;
                            }
                        };
                        if let Some(parent) = out_path.parent() {
                            std::fs::create_dir_all(parent).ok();
                        }
                        let mut out_file = std::fs::File::create(&out_path)?;
                        std::io::copy(&mut f, &mut out_file)?;
                        files.push(out_path);
                        total_size += size;
                    } else {
                        let out_path = validate_safe_path(&extract_dir, &extract_dir.join(&path))
                            .unwrap_or_else(|_| extract_dir.join(&path));
                        std::fs::create_dir_all(&out_path).ok();
                    }
                    (path, dir, size)
                };
            }

            Ok(ExtractResult {
                files,
                total_size,
                success: true,
                error: None,
            })
        })
        .await?
    }

    /// 解压 TAR 文件
    async fn extract_tar(
        &self,
        path: &Path,
        extract_dir: &Path,
    ) -> Result<ExtractResult> {
        let path = path.to_path_buf();
        let extract_dir = extract_dir.to_path_buf();

        tokio::task::spawn_blocking(move || {
            let file = std::fs::File::open(&path).context("打开 TAR 文件失败")?;
            let mut archive = tar::Archive::new(file);

            let mut files = Vec::new();
            let mut total_size = 0u64;

            for entry in archive.entries().context("读取 TAR 条目失败")? {
                let mut entry = entry.context("解析 TAR 条目失败")?;
                let raw_path = entry.path().context("获取 TAR 条目路径失败")?.to_path_buf();
                let out_path = match validate_safe_path(&extract_dir, &extract_dir.join(&raw_path)) {
                    Ok(p) => p,
                    Err(e) => {
                        warn!("跳过不安全的 TAR 条目 '{}': {}", raw_path.display(), e);
                        continue;
                    }
                };

                if let Some(parent) = out_path.parent() {
                    std::fs::create_dir_all(parent).ok();
                }

                entry.unpack(&out_path).context("解压 TAR 条目失败")?;
                total_size += entry.header().size().unwrap_or(0);
                files.push(out_path);
            }

            Ok(ExtractResult {
                files,
                total_size,
                success: true,
                error: None,
            })
        })
        .await?
    }

    /// 解压 TAR.GZ 文件
    async fn extract_tar_gz(
        &self,
        path: &Path,
        extract_dir: &Path,
    ) -> Result<ExtractResult> {
        let path = path.to_path_buf();
        let extract_dir = extract_dir.to_path_buf();

        tokio::task::spawn_blocking(move || {
            let file = std::fs::File::open(&path).context("打开 TAR.GZ 文件失败")?;
            let decoder = flate2::read::GzDecoder::new(file);
            let mut archive = tar::Archive::new(decoder);

            let mut files = Vec::new();
            let mut total_size = 0u64;

            for entry in archive.entries().context("读取 TAR.GZ 条目失败")? {
                let mut entry = entry.context("解析 TAR.GZ 条目失败")?;
                let raw_path = entry.path().context("获取路径失败")?.to_path_buf();
                let out_path = match validate_safe_path(&extract_dir, &extract_dir.join(&raw_path)) {
                    Ok(p) => p,
                    Err(e) => {
                        warn!("跳过不安全的 TAR.GZ 条目 '{}': {}", raw_path.display(), e);
                        continue;
                    }
                };

                if let Some(parent) = out_path.parent() {
                    std::fs::create_dir_all(parent).ok();
                }

                entry.unpack(&out_path).context("解压失败")?;
                total_size += entry.header().size().unwrap_or(0);
                files.push(out_path);
            }

            Ok(ExtractResult {
                files,
                total_size,
                success: true,
                error: None,
            })
        })
        .await?
    }

    /// 解压 TAR.BZ2 文件
    async fn extract_tar_bz2(
        &self,
        path: &Path,
        extract_dir: &Path,
    ) -> Result<ExtractResult> {
        let path = path.to_path_buf();
        let extract_dir = extract_dir.to_path_buf();

        tokio::task::spawn_blocking(move || {
            let file = std::fs::File::open(&path).context("打开 TAR.BZ2 文件失败")?;
            let decoder = bzip2::read::BzDecoder::new(file);
            let mut archive = tar::Archive::new(decoder);

            let mut files = Vec::new();
            let mut total_size = 0u64;

            for entry in archive.entries().context("读取 TAR.BZ2 条目失败")? {
                let mut entry = entry.context("解析 TAR.BZ2 条目失败")?;
                let raw_path = entry.path().context("获取路径失败")?.to_path_buf();
                let out_path = match validate_safe_path(&extract_dir, &extract_dir.join(&raw_path)) {
                    Ok(p) => p,
                    Err(e) => {
                        warn!("跳过不安全的 TAR.BZ2 条目 '{}': {}", raw_path.display(), e);
                        continue;
                    }
                };

                if let Some(parent) = out_path.parent() {
                    std::fs::create_dir_all(parent).ok();
                }

                entry.unpack(&out_path).context("解压失败")?;
                total_size += entry.header().size().unwrap_or(0);
                files.push(out_path);
            }

            Ok(ExtractResult {
                files,
                total_size,
                success: true,
                error: None,
            })
        })
        .await?
    }

    /// 解压 TAR.XZ 文件
    async fn extract_tar_xz(
        &self,
        path: &Path,
        extract_dir: &Path,
    ) -> Result<ExtractResult> {
        let path = path.to_path_buf();
        let extract_dir = extract_dir.to_path_buf();

        tokio::task::spawn_blocking(move || {
            let file = std::fs::File::open(&path).context("打开 TAR.XZ 文件失败")?;
            let decoder = xz2::read::XzDecoder::new(file);
            let mut archive = tar::Archive::new(decoder);

            let mut files = Vec::new();
            let mut total_size = 0u64;

            for entry in archive.entries().context("读取 TAR.XZ 条目失败")? {
                let mut entry = entry.context("解析 TAR.XZ 条目失败")?;
                let raw_path = entry.path().context("获取路径失败")?.to_path_buf();
                let out_path = match validate_safe_path(&extract_dir, &extract_dir.join(&raw_path)) {
                    Ok(p) => p,
                    Err(e) => {
                        warn!("跳过不安全的 TAR.XZ 条目 '{}': {}", raw_path.display(), e);
                        continue;
                    }
                };

                if let Some(parent) = out_path.parent() {
                    std::fs::create_dir_all(parent).ok();
                }

                entry.unpack(&out_path).context("解压 TAR.XZ 条目失败")?;
                total_size += entry.header().size().unwrap_or(0);
                files.push(out_path);
            }

            Ok(ExtractResult {
                files,
                total_size,
                success: true,
                error: None,
            })
        })
        .await?
    }

    /// 解压 RAR 文件
    async fn extract_rar(
        &self,
        path: &Path,
        extract_dir: &Path,
        passwords: &[String],
    ) -> Result<ExtractResult> {
        let path = path.to_path_buf();
        let extract_dir = extract_dir.to_path_buf();
        let passwords = passwords.to_vec();

        tokio::task::spawn_blocking(move || {
            let mut files = Vec::new();
            let mut total_size = 0u64;

            // 尝试无密码打开，如果条目加密则尝试密码列表
            let mut archive = unrar::Archive::new(path.to_str().unwrap_or(""))
                .open_for_processing()
                .context("打开 RAR 文件失败")?;

            while let Some(header) = archive.read_header().context("读取 RAR 条目失败")? {
                let entry = header.entry();
                let entry_size = entry.unpacked_size as u64;
                let is_encrypted = entry.is_encrypted();

                if entry.is_file() {
                    let out_path = extract_dir.join(entry.filename.to_string_lossy().as_ref());

                    // Zip Slip 校验
                    let out_path = match validate_safe_path(&extract_dir, &out_path) {
                        Ok(p) => p,
                        Err(e) => {
                            warn!("跳过不安全的 RAR 条目: {}", e);
                            archive = header.skip().context("跳过 RAR 条目失败")?;
                            continue;
                        }
                    };

                    if let Some(parent) = out_path.parent() {
                        std::fs::create_dir_all(parent).ok();
                    }

                    if is_encrypted && !passwords.is_empty() {
                        // 尝试用密码列表解密
                        let mut extracted = false;
                        for password in &passwords {
                            let mut pw_archive = match unrar::Archive::with_password(
                                path.to_str().unwrap_or(""),
                                password,
                            )
                            .open_for_processing()
                            {
                                Ok(a) => a,
                                Err(_) => continue,
                            };

                            while let Some(pw_header) = pw_archive.read_header().unwrap_or(None) {
                                let pw_entry = pw_header.entry();
                                if pw_entry.filename == entry.filename {
                                    if pw_header.extract_to(extract_dir.to_str().unwrap_or("").to_string()).is_ok() {
                                        total_size += entry_size;
                                        files.push(out_path.clone());
                                        extracted = true;
                                    }
                                    break;
                                }
                                pw_archive = match pw_header.skip() {
                                    Ok(a) => a,
                                    Err(_) => break,
                                };
                            }
                            if extracted {
                                break;
                            }
                        }
                        if !extracted {
                            warn!("RAR 条目需要密码: {}", entry.filename.display());
                        }
                        // 跳过当前条目，继续处理下一个
                        archive = match header.skip() {
                            Ok(a) => a,
                            Err(_) => break,
                        };
                    } else {
                        // 无密码或不需要密码，直接解压
                        archive = header
                            .extract_to(extract_dir.to_str().unwrap_or("").to_string())
                            .context("解压 RAR 条目失败")?;
                        total_size += entry_size;
                        files.push(out_path);
                    }
                } else {
                    // 目录或非文件条目，跳过
                    archive = header.skip().context("跳过 RAR 条目失败")?;
                }
            }

            Ok(ExtractResult {
                files,
                total_size,
                success: true,
                error: None,
            })
        })
        .await?
    }

    /// 解压 7z 文件
    /// 注意：sevenz_rust::decompress_file 不支持逐条目过滤，
    /// 因此在解压后对新文件执行路径安全校验，移除逃逸文件
    async fn extract_7z(
        &self,
        path: &Path,
        extract_dir: &Path,
    ) -> Result<ExtractResult> {
        let path = path.to_path_buf();
        let extract_dir = extract_dir.to_path_buf();

        tokio::task::spawn_blocking(move || {
            // 安全策略：先解压到临时目录，校验后再移动到目标目录
            // 防止恶意 7z 文件通过路径遍历写入目标目录外的位置
            let temp_dir = extract_dir.join(format!(".7z_temp_{}", uuid::Uuid::new_v4()));
            std::fs::create_dir_all(&temp_dir)?;

            let result = (|| -> Result<ExtractResult> {
                sevenz_rust::decompress_file(
                    path.to_str().unwrap_or(""),
                    temp_dir.to_str().unwrap_or(""),
                )
                .context("解压 7z 文件失败")?;

                // 收集解压出的文件，逐个校验路径安全性
                let extracted_files = walkdir_paths(&temp_dir);
                let mut files = Vec::new();
                let mut total_size = 0u64;

                for f in &extracted_files {
                    match validate_safe_path(&temp_dir, f) {
                        Ok(safe_rel_path) => {
                            // 计算目标路径并移动文件
                            let dest = extract_dir.join(&safe_rel_path);
                            if let Some(parent) = dest.parent() {
                                std::fs::create_dir_all(parent).ok();
                            }
                            std::fs::rename(f, &dest).or_else(|_| {
                                std::fs::copy(f, &dest).map(|_| { std::fs::remove_file(f).ok(); })
                            }).ok();
                            if let Ok(meta) = std::fs::metadata(&dest) {
                                total_size += meta.len();
                            }
                            files.push(dest);
                        }
                        Err(e) => {
                            warn!("拒绝不安全的 7z 解压文件 {:?}: {}", f, e);
                            std::fs::remove_file(f).ok();
                        }
                    }
                }

                Ok(ExtractResult {
                    files,
                    total_size,
                    success: true,
                    error: None,
                })
            })();

            // 清理临时目录
            std::fs::remove_dir_all(&temp_dir).ok();
            result
        })
        .await?
    }

    /// 获取与文件关联的密码
    fn get_passwords_for_file(&self, file_path: &str) -> Vec<String> {
        self.config
            .passwords
            .iter()
            .filter(|p| p.enabled)
            .filter(|p| {
                if p.pattern.is_empty() {
                    return true;
                }
                // 简单的域名/路径匹配
                file_path.contains(&p.pattern)
            })
            .map(|p| p.password.clone())
            .collect()
    }

    /// 获取当前配置
    pub fn config(&self) -> &ArchiveConfig {
        &self.config
    }

    /// 更新配置
    pub fn update_config(&mut self, config: ArchiveConfig) {
        self.config = config;
    }
}

/// 递归遍历目录，返回所有文件路径
fn walkdir_paths(dir: &Path) -> Vec<PathBuf> {
    let mut result = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                result.extend(walkdir_paths(&path));
            } else {
                result.push(path);
            }
        }
    }
    result
}
