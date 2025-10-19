use regex::Regex;
use std::fmt;

/// 目标类型枚举
#[derive(Debug, Clone, PartialEq)]
pub enum TargetType {
    Url,
    Domain,
    Ip,
    IpPort,
}

impl fmt::Display for TargetType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TargetType::Url => write!(f, "URL"),
            TargetType::Domain => write!(f, "域名"),
            TargetType::Ip => write!(f, "IP地址"),
            TargetType::IpPort => write!(f, "IP:端口"),
        }
    }
}

/// 截图目标结构体
#[derive(Debug, Clone)]
pub struct Target {
    pub url: String,
    pub original_text: String,
}

impl Target {
    /// 创建新的目标实例
    pub fn new(text: String) -> Option<Self> {
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

    /// 获取目标类型
    pub fn get_type(&self) -> TargetType {
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

    /// 获取清理后的文件名
    pub fn get_clean_filename(&self) -> String {
        self.original_text
            .replace("://", "_")
            .replace('/', "_")
            .replace(':', "_")
            .replace('?', "_")
            .replace('=', "_")
            .replace('&', "_")
            .replace('%', "_")
    }
}

/// 截图结果结构体
#[derive(Debug)]
pub struct ScreenshotResult {
    pub target: Target,
    pub success: bool,
    pub file_path: Option<String>,
    pub error_message: Option<String>,
}

impl ScreenshotResult {
    pub fn success(target: Target, file_path: String) -> Self {
        Self {
            target,
            success: true,
            file_path: Some(file_path),
            error_message: None,
        }
    }

    pub fn failure(target: Target, error_message: String) -> Self {
        Self {
            target,
            success: false,
            file_path: None,
            error_message: Some(error_message),
        }
    }
}
