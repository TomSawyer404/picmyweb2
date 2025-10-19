use picmyweb2::cli::cli_parser::CliParser;
use picmyweb2::config::app_config::AppConfig;
use picmyweb2::file_io::file_operations::{FileOperations, ScreenshotRecord};
use picmyweb2::models::target::{Target, TargetType};
use picmyweb2::screenshot::concurrent_executor::ConcurrentExecutor;

use csv::Writer;
use log::{error, info, warn};
use std::fs;
use std::sync::{Arc, Mutex};
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志系统
    env_logger::init();

    info!("picmyweb2 应用程序启动");

    // 解析命令行参数
    let config = CliParser::parse();
    let file_path = CliParser::get_file_path();

    info!("正在读取文件: {}", file_path);

    // 检查文件是否存在
    if !FileOperations::file_exists(&file_path) {
        error!("文件不存在: {}", file_path);
        return Ok(());
    }

    // 解析目标
    let targets = FileOperations::parse_targets_from_file(&file_path)?;

    if targets.is_empty() {
        warn!("未找到有效的URL目标");
        return Ok(());
    }

    // 显示统计信息
    display_target_statistics(&targets);

    // 确保截图目录存在
    FileOperations::ensure_screenshots_dir(&config.screenshots_dir)?;

    // 创建CSV日志文件
    let csv_path = format!("{}\\screenshot_log.csv", config.screenshots_dir);
    let csv_writer = FileOperations::create_csv_log_file(&csv_path)?;
    let csv_writer_arc = Arc::new(Mutex::new(csv_writer));

    // 开始异步截图会话
    let (success_count, fail_count) =
        start_async_screenshot_session(&targets, config.clone(), csv_writer_arc).await?;

    // 输出最终结果信息
    let completion_message = format!(
        "异步截图完成! 成功: {}, 失败: {}，成功率：{:.2}%",
        success_count,
        fail_count,
        (success_count as f64 / (success_count + fail_count) as f64) * 100.0
    );
    println!("{}", completion_message);
    println!("截图保存在: {}", config.screenshots_dir);
    println!("CSV日志文件: {}", csv_path);
    println!("应用程序正常退出");
    Ok(())
}

/// 显示目标统计信息
fn display_target_statistics(targets: &[Target]) {
    info!("找到 {} 个目标", targets.len());

    let mut url_count = 0;
    let mut domain_count = 0;
    let mut ip_count = 0;
    let mut ip_port_count = 0;

    for target in targets {
        match target.get_type() {
            TargetType::Url => url_count += 1,
            TargetType::Domain => domain_count += 1,
            TargetType::Ip => ip_count += 1,
            TargetType::IpPort => ip_port_count += 1,
        }
    }

    info!("目标类型统计:");
    info!("  - URL: {}", url_count);
    info!("  - 域名: {}", domain_count);
    info!("  - IP地址: {}", ip_count);
    info!("  - IP:端口: {}", ip_port_count);
}

/// 开始异步截图会话
async fn start_async_screenshot_session(
    targets: &[Target],
    config: AppConfig,
    csv_writer_arc: Arc<Mutex<Writer<fs::File>>>,
) -> Result<(usize, usize), Box<dyn std::error::Error>> {
    let start_message = format!(
        "开始异步截图，目标数量: {}, 并发数: {}",
        targets.len(),
        config.concurrency
    );
    info!("{}", start_message);

    info!("使用多线程异步模式进行截图...");

    // 创建并发执行器
    let executor = ConcurrentExecutor::new(config.clone());

    // 执行并发截图，实时处理每个结果
    let csv_writer_arc_clone = Arc::clone(&csv_writer_arc);

    let (success_count, fail_count) = executor
        .execute_concurrent_screenshots(targets.to_vec(), move |target, result| {
            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();

            // 创建截图记录
            let mut record = ScreenshotRecord {
                timestamp,
                target: target.original_text.clone(),
                target_type: target.get_type().to_string(),
                success: false,
                error_message: None,
                screenshot_path: None,
            };

            // 实时处理每个截图结果
            match result {
                Ok(screenshot_result) => {
                    if screenshot_result.success {
                        record.success = true;
                        record.screenshot_path = screenshot_result.file_path.clone();

                        let success_log = format!("✓ 成功截图: {}", target.original_text);
                        info!("{}", success_log);

                        // 写入CSV记录
                        if let Ok(mut csv_writer) = csv_writer_arc_clone.lock() {
                            let _ = FileOperations::log_csv_record(&mut *csv_writer, &record);
                        }
                    } else {
                        record.success = false;
                        record.error_message = screenshot_result.error_message.clone();

                        let error_log = format!(
                            "✗ 截图失败 {}: {}",
                            target.original_text,
                            screenshot_result
                                .error_message
                                .as_ref()
                                .unwrap_or(&"未知错误".to_string())
                        );
                        error!("{}", error_log);

                        if let Ok(mut csv_writer) = csv_writer_arc_clone.lock() {
                            let _ = FileOperations::log_csv_record(&mut *csv_writer, &record);
                        }
                    }
                }
                Err(e) => {
                    record.success = false;
                    record.error_message = Some(e.to_string());

                    let error_log = format!("✗ 截图失败 {}: {}", target.original_text, e);
                    error!("{}", error_log);

                    if let Ok(mut csv_writer) = csv_writer_arc_clone.lock() {
                        let _ = FileOperations::log_csv_record(&mut *csv_writer, &record);
                    }
                }
            }
        })
        .await;

    Ok((success_count, fail_count))
}
