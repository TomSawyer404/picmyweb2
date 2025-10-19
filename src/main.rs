use clap::{Arg, Command};
use headless_chrome::{Browser, LaunchOptions, protocol::cdp::Page::CaptureScreenshotFormatOption};
use regex::Regex;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug)]
struct Target {
    url: String,
    original_text: String,
}

#[derive(Debug)]
enum TargetType {
    Url,
    Domain,
    Ip,
    IpPort,
}

const USER_AGENT: &str = "User-Agent,Mozilla/5.0 (iPhone; U; CPU iPhone OS 4_3_3 like Mac OS X; en-us) AppleWebKit/533.17.9 (KHTML, like Gecko) Version/5.0.2 Mobile/8J2 Safari/6533.18.5";

impl Target {
    fn new(text: String) -> Option<Self> {
        let trimmed = text.trim().to_string();
        if trimmed.is_empty() {
            return None;
        }

        // 添加协议前缀如果缺失
        let url = if !trimmed.starts_with("http://") && !trimmed.starts_with("https://") {
            format!("https://{}", trimmed)
        } else {
            trimmed.clone()
        };

        Some(Target {
            url,
            original_text: trimmed,
        })
    }

    fn get_type(&self) -> TargetType {
        let url_regex = Regex::new(r"^https?://").unwrap();
        let ip_regex = Regex::new(r"^\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}$").unwrap();
        let ip_port_regex = Regex::new(r"^\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}:\d+$").unwrap();

        let clean_text = self.original_text.trim();

        if url_regex.is_match(clean_text) {
            TargetType::Url
        } else if ip_port_regex.is_match(clean_text) {
            TargetType::IpPort
        } else if ip_regex.is_match(clean_text) {
            TargetType::Ip
        } else {
            TargetType::Domain
        }
    }
}

fn parse_targets_from_file(file_path: &str) -> Result<Vec<Target>, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(file_path)?;
    let mut targets = Vec::new();

    for line in content.lines() {
        if let Some(target) = Target::new(line.to_string()) {
            targets.push(target);
        }
    }

    Ok(targets)
}

fn get_desktop_path() -> String {
    if let Ok(home) = std::env::var("USERPROFILE") {
        format!("{}\\Desktop", home)
    } else {
        ".".to_string()
    }
}

fn ensure_screenshots_dir() -> Result<String, Box<dyn std::error::Error>> {
    let desktop_path = get_desktop_path();
    let screenshots_dir = format!("{}\\screen_shots", desktop_path);

    if !Path::new(&screenshots_dir).exists() {
        fs::create_dir_all(&screenshots_dir)?;
        println!("创建截图文件夹: {}", screenshots_dir);
    }

    Ok(screenshots_dir)
}

fn generate_filename(target: &Target) -> String {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // 清理URL中的特殊字符用于文件名
    let clean_name = target
        .original_text
        .replace("://", "_")
        .replace('/', "_")
        .replace(':', "_")
        .replace('?', "_")
        .replace('=', "_")
        .replace('&', "_")
        .replace('%', "_");

    format!("{}_{}.png", clean_name, timestamp)
}

fn log_to_file(log_file: &mut fs::File, message: &str) -> Result<(), Box<dyn std::error::Error>> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    writeln!(log_file, "[{}] {}", timestamp, message)?;
    Ok(())
}

fn take_screenshot(
    target: &Target,
    log_file: &mut fs::File,
) -> Result<(), Box<dyn std::error::Error>> {
    let log_message = format!("正在访问: {}", target.url);
    println!("{}", log_message);
    log_to_file(log_file, &log_message)?;

    // 创建浏览器实例
    let browser = Browser::new(
        LaunchOptions::default_builder()
            .headless(true)
            .window_size(Some((414, 896))) // iPhone XR 的尺寸，更美观的移动设备比例
            .build()?,
    )?;

    // 创建新标签页
    let tab = browser.new_tab()?;

    tab.set_user_agent(USER_AGENT, None, None)?;

    // 导航到目标URL
    tab.navigate_to(&target.url)?;

    // 等待页面加载完成
    tab.wait_until_navigated()?;

    // 等待额外时间确保页面完全渲染
    std::thread::sleep(std::time::Duration::from_secs(3));

    // 添加地址栏显示
    let address_bar_html = format!(
        r#"
        <div id="custom-address-bar" style="
            position: fixed;
            top: 0;
            left: 0;
            width: 100%;
            height: 44px;
            background: linear-gradient(to bottom, #f8f8f8, #e8e8e8);
            border-bottom: 1px solid #b2b2b2;
            display: flex;
            align-items: center;
            padding: 0 12px;
            box-sizing: border-box;
            z-index: 999999;
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', 'Roboto', sans-serif;
            font-size: 14px;
        ">
            <div style="
                background: white;
                border: 1px solid #b2b2b2;
                border-radius: 18px;
                padding: 8px 12px;
                width: 100%;
                color: #333;
                overflow: hidden;
                text-overflow: ellipsis;
                white-space: nowrap;
                box-shadow: 0 1px 3px rgba(0,0,0,0.1);
            ">
                {}
            </div>
        </div>
        
        <script>
            // 调整页面内容以避免被地址栏遮挡
            document.body.style.paddingTop = '44px';
            document.documentElement.style.paddingTop = '44px';
        </script>
        "#,
        target.url
    );

    // 注入地址栏到页面
    tab.evaluate(
        &format!(
            "document.documentElement.insertAdjacentHTML('afterbegin', `{}`);",
            address_bar_html.replace('`', "\\`")
        ),
        true,
    )?;

    // 等待一下确保地址栏渲染完成
    std::thread::sleep(std::time::Duration::from_millis(500));

    // 获取截图文件夹路径并生成文件名
    let screenshots_dir = ensure_screenshots_dir()?;
    let filename = generate_filename(target);
    let full_path = format!("{}\\{}", screenshots_dir, filename);

    // 截图并保存
    let screenshot_data =
        tab.capture_screenshot(CaptureScreenshotFormatOption::Png, None, None, true)?;

    fs::write(&full_path, screenshot_data)?;

    let success_message = format!("截图已保存: {}", full_path);
    println!("{}", success_message);
    log_to_file(log_file, &success_message)?;

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = Command::new("picmyweb2")
        .version("0.1.0")
        .about("网页截图工具 - 从文本文件读取URL并截图")
        .author("Your Name")
        .arg(
            Arg::new("file")
                .short('f')
                .long("file")
                .value_name("FILE")
                .help("包含URL的文本文件路径")
                .required(true),
        )
        .arg(
            Arg::new("timeout")
                .short('t')
                .long("timeout")
                .value_name("SECONDS")
                .help("页面加载等待时间（秒）")
                .default_value("5"),
        )
        .arg(
            Arg::new("headless")
                .long("headless")
                .help("是否使用无头模式（默认启用）"),
        )
        .get_matches();

    let file_path = matches.get_one::<String>("file").unwrap();
    let _timeout: u64 = matches
        .get_one::<String>("timeout")
        .unwrap()
        .parse()
        .unwrap_or(5);

    println!("正在读取文件: {}", file_path);

    // 检查文件是否存在
    if !Path::new(file_path).exists() {
        eprintln!("错误: 文件不存在: {}", file_path);
        return Ok(());
    }

    // 解析目标
    let targets = parse_targets_from_file(file_path)?;

    if targets.is_empty() {
        println!("未找到有效的URL目标");
        return Ok(());
    }

    // 只显示统计量，不列出具体值
    println!("找到 {} 个目标", targets.len());

    // 统计不同类型的目标数量
    let mut url_count = 0;
    let mut domain_count = 0;
    let mut ip_count = 0;
    let mut ip_port_count = 0;

    for target in &targets {
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

    // 创建日志文件（追加模式）
    let screenshots_dir = ensure_screenshots_dir()?;
    let log_path = format!("{}\\screenshot_log.txt", screenshots_dir);
    let mut log_file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)?;

    // 添加分隔线标识新的运行会话
    let separator = "=".repeat(60);
    let session_start = format!("新的截图会话开始 {}", separator);
    log_to_file(&mut log_file, &session_start)?;

    let start_message = format!("开始截图，目标数量: {}", targets.len());
    println!("\n{}", start_message);
    log_to_file(&mut log_file, &start_message)?;

    println!("使用headless_chrome进行截图...");
    log_to_file(&mut log_file, "使用headless_chrome进行截图...")?;

    let mut success_count = 0;
    let mut fail_count = 0;

    for target in targets {
        match take_screenshot(&target, &mut log_file) {
            Ok(_) => {
                let success_log = format!("✓ 成功截图: {}", target.original_text);
                println!("{}", success_log);
                log_to_file(&mut log_file, &success_log)?;
                success_count += 1;
            }
            Err(e) => {
                let error_log = format!("✗ 截图失败 {}: {}", target.original_text, e);
                eprintln!("{}", error_log);
                log_to_file(&mut log_file, &error_log)?;
                fail_count += 1;
            }
        }

        // 间隔一下避免请求过快
        std::thread::sleep(std::time::Duration::from_secs(1));
    }

    let completion_message = format!("截图完成! 成功: {}, 失败: {}", success_count, fail_count);
    println!("\n{}", completion_message);
    log_to_file(&mut log_file, &completion_message)?;

    let session_end = format!("截图会话结束 {}", separator);
    log_to_file(&mut log_file, &session_end)?;
    log_to_file(&mut log_file, "")?; // 添加空行分隔不同会话

    println!("截图保存在: {}", screenshots_dir);
    println!("日志文件: {}", log_path);

    Ok(())
}
