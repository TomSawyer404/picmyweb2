/// 路径工具函数
pub struct PathUtils;

impl PathUtils {
    /// 获取桌面路径
    pub fn get_desktop_path() -> String {
        if let Ok(home) = std::env::var("USERPROFILE") {
            format!("{}\\Desktop", home)
        } else {
            ".".to_string()
        }
    }

    /// 获取默认截图目录
    pub fn get_default_screenshots_dir() -> String {
        format!("{}\\screen_shots", Self::get_desktop_path())
    }

    /// 确保路径存在
    pub fn ensure_path_exists(path: &str) -> Result<(), Box<dyn std::error::Error>> {
        if !std::path::Path::new(path).exists() {
            std::fs::create_dir_all(path)?;
        }
        Ok(())
    }
}
