use clap::{Arg, Command};

use crate::config::app_config::AppConfig;

/// 命令行参数解析器
pub struct CliParser;

impl CliParser {
    /// 解析命令行参数
    pub fn parse() -> AppConfig {
        let matches = Command::new("picmyweb2")
            .version("0.1.1")
            .about("网页截图工具 - 从文本文件读取URL并截图")
            .author("MrBanana @ 佛子岭日夜加班有限公司")
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
            .arg(
                Arg::new("output")
                    .short('o')
                    .long("output")
                    .value_name("DIRECTORY")
                    .help("截图保存目录"),
            )
            .get_matches();

        let timeout: u64 = matches
            .get_one::<String>("timeout")
            .unwrap()
            .parse()
            .unwrap_or(5);

        let headless = !matches.contains_id("headless"); // 默认启用无头模式

        let mut config = AppConfig::default();
        config.timeout_seconds = timeout;
        config.headless = headless;

        if let Some(output_dir) = matches.get_one::<String>("output") {
            config.screenshots_dir = output_dir.clone();
        }

        config
    }

    /// 获取文件路径参数
    pub fn get_file_path() -> String {
        let matches = Command::new("picmyweb2")
            .arg(
                Arg::new("file")
                    .short('f')
                    .long("file")
                    .value_name("FILE")
                    .required(true),
            )
            .get_matches();

        matches.get_one::<String>("file").unwrap().clone()
    }
}
