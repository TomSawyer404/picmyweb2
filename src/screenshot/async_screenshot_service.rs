use headless_chrome::{Browser, LaunchOptions, protocol::cdp::Page::CaptureScreenshotFormatOption};
use std::fs;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::task;

use crate::config::app_config::AppConfig;
use crate::models::target::{ScreenshotResult, Target};

/// 异步截图服务
pub struct AsyncScreenshotService {
    config: Arc<AppConfig>,
}

impl AsyncScreenshotService {
    pub fn new(config: AppConfig) -> Self {
        Self {
            config: Arc::new(config),
        }
    }

    /// 异步执行截图操作
    pub async fn take_screenshot_async(
        &self,
        target: Arc<Target>,
    ) -> Result<ScreenshotResult, Box<dyn std::error::Error + Send + Sync>> {
        let config = Arc::clone(&self.config);

        // 使用tokio的blocking任务执行同步的浏览器操作
        let result =
            task::spawn_blocking(move || Self::take_screenshot_sync(&config, &target)).await?;

        result
    }

    /// 同步截图操作（在阻塞任务中执行）
    fn take_screenshot_sync(
        config: &AppConfig,
        target: &Target,
    ) -> Result<ScreenshotResult, Box<dyn std::error::Error + Send + Sync>> {
        // 创建浏览器实例
        let browser = Browser::new(
            LaunchOptions::default_builder()
                .headless(config.headless)
                .window_size(Some(config.window_size))
                .build()?,
        )?;

        // 创建新标签页
        let tab = browser.new_tab()?;
        tab.set_user_agent(&config.user_agent, None, None)?;

        // 导航到目标URL
        tab.navigate_to(&target.url)?;
        tab.wait_until_navigated()?;

        // 等待页面加载完成
        std::thread::sleep(std::time::Duration::from_secs(3));

        // 添加地址栏
        Self::add_address_bar(&tab, target)?;

        // 生成文件名并保存截图
        let filename = Self::generate_filename(target);
        let full_path = format!("{}\\{}", config.screenshots_dir, filename);

        let screenshot_data =
            tab.capture_screenshot(CaptureScreenshotFormatOption::Png, None, None, true)?;

        fs::write(&full_path, screenshot_data)?;

        Ok(ScreenshotResult::success(target.clone(), full_path))
    }

    /// 添加自定义地址栏到页面
    fn add_address_bar(
        tab: &headless_chrome::Tab,
        target: &Target,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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
                document.body.style.paddingTop = '44px';
                document.documentElement.style.paddingTop = '44px';
            </script>
            "#,
            target.url
        );

        tab.evaluate(
            &format!(
                "document.documentElement.insertAdjacentHTML('afterbegin', `{}`);",
                address_bar_html.replace('`', "\\`")
            ),
            true,
        )?;

        std::thread::sleep(std::time::Duration::from_millis(500));
        Ok(())
    }

    /// 生成截图文件名
    fn generate_filename(target: &Target) -> String {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let clean_name = target.get_clean_filename();
        format!("{}_{}.png", clean_name, timestamp)
    }
}
