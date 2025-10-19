use csv::Writer;
use log::info;
use std::fs;
use std::path::Path;

use crate::models::target::Target;

/// 截图结果记录
#[derive(Debug, Clone)]
pub struct ScreenshotRecord {
    pub timestamp: u64,
    pub target: String,
    pub target_type: String,
    pub success: bool,
    pub error_message: Option<String>,
    pub screenshot_path: Option<String>,
}

/// 文件操作服务
pub struct FileOperations;

impl FileOperations {
    /// 从文件解析目标列表
    pub fn parse_targets_from_file(
        file_path: &str,
    ) -> Result<Vec<Target>, Box<dyn std::error::Error>> {
        info!("正在读取文件: {}", file_path);
        let content = fs::read_to_string(file_path)?;
        let mut targets = Vec::new();

        for line in content.lines() {
            if let Some(target) = Target::new(line.to_string()) {
                targets.push(target);
            }
        }

        info!("成功解析 {} 个目标", targets.len());
        Ok(targets)
    }

    /// 确保截图目录存在
    pub fn ensure_screenshots_dir(screenshots_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
        if !Path::new(screenshots_dir).exists() {
            fs::create_dir_all(screenshots_dir)?;
            info!("创建截图文件夹: {}", screenshots_dir);
        }
        Ok(())
    }

    /// 创建CSV日志文件
    pub fn create_csv_log_file(
        csv_path: &str,
    ) -> Result<Writer<fs::File>, Box<dyn std::error::Error>> {
        let file = fs::File::create(csv_path)?;
        let mut writer = Writer::from_writer(file);

        // 写入CSV表头
        writer.write_record(&[
            "timestamp",
            "target",
            "target_type",
            "success",
            "error_message",
            "screenshot_path",
        ])?;
        writer.flush()?;

        info!("创建CSV日志文件: {}", csv_path);
        Ok(writer)
    }

    /// 检查文件是否存在
    pub fn file_exists(file_path: &str) -> bool {
        Path::new(file_path).exists()
    }

    /// 写入CSV记录
    pub fn log_csv_record(
        csv_writer: &mut Writer<fs::File>,
        record: &ScreenshotRecord,
    ) -> Result<(), Box<dyn std::error::Error>> {
        csv_writer.write_record(&[
            &record.timestamp.to_string(),
            &record.target,
            &record.target_type,
            &record.success.to_string(),
            &record.error_message.as_ref().unwrap_or(&"".to_string()),
            &record.screenshot_path.as_ref().unwrap_or(&"".to_string()),
        ])?;
        csv_writer.flush()?;
        Ok(())
    }
}
