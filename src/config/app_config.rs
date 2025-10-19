use std::time::Duration;

/// 应用程序配置
#[derive(Debug, Clone)]
pub struct AppConfig {
    /// 默认等待超时时间（秒）
    pub timeout_seconds: u64,
    /// 是否使用无头模式
    pub headless: bool,
    /// 浏览器窗口尺寸
    pub window_size: (u32, u32),
    /// 截图保存目录
    pub screenshots_dir: String,
    /// 用户代理字符串
    pub user_agent: String,
    /// 截图之间的延迟（秒）
    pub delay_between_screenshots: u64,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            timeout_seconds: 5,
            headless: true,
            window_size: (414, 896), // iPhone XR 尺寸
            screenshots_dir: Self::get_default_screenshots_dir(),
            user_agent: "User-Agent,Mozilla/5.0 (iPhone; U; CPU iPhone OS 4_3_3 like Mac OS X; en-us) AppleWebKit/533.17.9 (KHTML, like Gecko) Version/5.0.2 Mobile/8J2 Safari/6533.18.5".to_string(),
            delay_between_screenshots: 1,
        }
    }
}

impl AppConfig {
    /// 获取默认的截图保存目录
    pub fn get_default_screenshots_dir() -> String {
        if let Ok(home) = std::env::var("USERPROFILE") {
            format!("{}\\Desktop\\screen_shots", home)
        } else {
            "screen_shots".to_string()
        }
    }

    /// 获取等待超时时间
    pub fn get_timeout_duration(&self) -> Duration {
        Duration::from_secs(self.timeout_seconds)
    }

    /// 获取截图之间的延迟时间
    pub fn get_delay_duration(&self) -> Duration {
        Duration::from_secs(self.delay_between_screenshots)
    }
}
