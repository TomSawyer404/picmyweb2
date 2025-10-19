use std::fs;
use std::io::Write;
use std::path::Path;

use crate::models::target::Target;

/// 文件操作服务
pub struct FileOperations;

impl FileOperations {
    /// 从文件解析目标列表
    pub fn parse_targets_from_file(
        file_path: &str,
    ) -> Result<Vec<Target>, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(file_path)?;
        let mut targets = Vec::new();

        for line in content.lines() {
            if let Some(target) = Target::new(line.to_string()) {
                targets.push(target);
            }
        }

        Ok(targets)
    }

    /// 确保截图目录存在
    pub fn ensure_screenshots_dir(screenshots_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
        if !Path::new(screenshots_dir).exists() {
            fs::create_dir_all(screenshots_dir)?;
            println!("创建截图文件夹: {}", screenshots_dir);
        }
        Ok(())
    }

    /// 创建或打开日志文件
    pub fn create_log_file(log_path: &str) -> Result<fs::File, Box<dyn std::error::Error>> {
        let log_file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_path)?;
        Ok(log_file)
    }

    /// 检查文件是否存在
    pub fn file_exists(file_path: &str) -> bool {
        Path::new(file_path).exists()
    }

    /// 写入日志消息
    pub fn log_message(
        log_file: &mut fs::File,
        message: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        writeln!(log_file, "[{}] {}", timestamp, message)?;
        Ok(())
    }
}
