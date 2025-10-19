//! picmyweb2 - 网页截图工具库
//!
//! 按照关注点分离原则设计的模块化网页截图工具

pub mod cli;
pub mod config;
pub mod file_io;
pub mod models;
pub mod screenshot;
pub mod utils;

// 重新导出常用类型，方便外部使用
pub use cli::cli_parser::CliParser;
pub use config::app_config::AppConfig;
pub use file_io::file_operations::FileOperations;
pub use models::target::{Target, TargetType};
pub use screenshot::async_screenshot_service::AsyncScreenshotService;
pub use screenshot::concurrent_executor::ConcurrentExecutor;
pub use screenshot::screenshot_service::ScreenshotService;
