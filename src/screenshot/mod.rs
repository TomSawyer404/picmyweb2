//! 截图功能模块
//!
//! 包含网页截图的核心功能

pub mod async_screenshot_service;
pub mod concurrent_executor;
pub mod screenshot_service;

pub use async_screenshot_service::AsyncScreenshotService;
pub use concurrent_executor::ConcurrentExecutor;
pub use screenshot_service::ScreenshotService;
