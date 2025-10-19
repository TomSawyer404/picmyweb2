use picmyweb2::cli::cli_parser::CliParser;
use picmyweb2::config::app_config::AppConfig;
use picmyweb2::file_io::file_operations::FileOperations;
use picmyweb2::models::target::{Target, TargetType};
use picmyweb2::screenshot::concurrent_executor::ConcurrentExecutor;

use std::fs;
use std::sync::{Arc, Mutex};
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 解析命令行参数
    let config = CliParser::parse();
    let file_path = CliParser::get_file_path();

    println!("正在读取文件: {}", file_path);

    // 检查文件是否存在
    if !FileOperations::file_exists(&file_path) {
        eprintln!("错误: 文件不存在: {}", file_path);
        return Ok(());
    }

    // 解析目标
    let targets = FileOperations::parse_targets_from_file(&file_path)?;

    if targets.is_empty() {
        println!("未找到有效的URL目标");
        return Ok(());
    }

    // 显示统计信息
    display_target_statistics(&targets);

    // 确保截图目录存在
    FileOperations::ensure_screenshots_dir(&config.screenshots_dir)?;

    // 创建日志文件
    let log_path = format!("{}\\screenshot_log.txt", config.screenshots_dir);
    let log_file = FileOperations::create_log_file(&log_path)?;
    let log_file_arc = Arc::new(Mutex::new(log_file));

    // 开始异步截图会话
    start_async_screenshot_session(&targets, config, log_file_arc, &log_path).await?;

    Ok(())
}

/// 显示目标统计信息
fn display_target_statistics(targets: &[Target]) {
    println!("找到 {} 个目标", targets.len());

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

    println!("目标类型统计:");
    println!("  - URL: {}", url_count);
    println!("  - 域名: {}", domain_count);
    println!("  - IP地址: {}", ip_count);
    println!("  - IP:端口: {}", ip_port_count);
}

/// 开始异步截图会话
async fn start_async_screenshot_session(
    targets: &[Target],
    config: AppConfig,
    log_file_arc: Arc<Mutex<fs::File>>,
    log_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // 添加分隔线标识新的运行会话
    let separator = "=".repeat(60);
    let session_start = format!("新的截图会话开始 {}", separator);

    {
        let mut log_file = log_file_arc.lock().unwrap();
        FileOperations::log_message(&mut *log_file, &session_start)?;
    }

    let start_message = format!(
        "开始异步截图，目标数量: {}, 并发数: {}",
        targets.len(),
        config.concurrency
    );
    println!("\n{}", start_message);

    {
        let mut log_file = log_file_arc.lock().unwrap();
        FileOperations::log_message(&mut *log_file, &start_message)?;
    }

    println!("使用多线程异步模式进行截图...");

    {
        let mut log_file = log_file_arc.lock().unwrap();
        FileOperations::log_message(&mut *log_file, "使用多线程异步模式进行截图...")?;
    }

    // 创建并发执行器
    let executor = ConcurrentExecutor::new(config.clone());

    // 执行并发截图，实时处理每个结果
    let log_file_arc_clone = Arc::clone(&log_file_arc); // 预先克隆Arc
    let (success_count, fail_count) = executor
        .execute_concurrent_screenshots(targets.to_vec(), move |target, result| {
            // 使用move关键字
            // 实时处理每个截图结果
            match result {
                Ok(screenshot_result) => {
                    if screenshot_result.success {
                        let success_log = format!("✓ 成功截图: {}", target.original_text);
                        println!("{}", success_log);

                        // 异步写入日志文件
                        if let Ok(mut log_file) = log_file_arc_clone.lock() {
                            let _ = FileOperations::log_message(&mut *log_file, &success_log);
                        }
                    } else {
                        let error_log = format!(
                            "✗ 截图失败 {}: {}",
                            target.original_text,
                            screenshot_result
                                .error_message
                                .as_ref()
                                .unwrap_or(&"未知错误".to_string())
                        );
                        eprintln!("{}", error_log);

                        if let Ok(mut log_file) = log_file_arc_clone.lock() {
                            let _ = FileOperations::log_message(&mut *log_file, &error_log);
                        }
                    }
                }
                Err(e) => {
                    let error_log = format!("✗ 截图失败 {}: {}", target.original_text, e);
                    eprintln!("{}", error_log);

                    if let Ok(mut log_file) = log_file_arc_clone.lock() {
                        let _ = FileOperations::log_message(&mut *log_file, &error_log);
                    }
                }
            }
        })
        .await;

    let completion_message = format!(
        "异步截图完成! 成功: {}, 失败: {}",
        success_count, fail_count
    );
    println!("\n{}", completion_message);

    {
        let mut log_file = log_file_arc.lock().unwrap();
        FileOperations::log_message(&mut *log_file, &completion_message)?;
    }

    let session_end = format!("截图会话结束 {}", separator);

    {
        let mut log_file = log_file_arc.lock().unwrap();
        FileOperations::log_message(&mut *log_file, &session_end)?;
        FileOperations::log_message(&mut *log_file, "")?; // 添加空行分隔不同会话
    }

    println!("截图保存在: {}", config.screenshots_dir);
    println!("日志文件: {}", log_path);

    Ok(())
}
