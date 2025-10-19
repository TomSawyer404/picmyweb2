//! picmyweb2 - 网页截图工具库
//!
//! 按照关注点分离原则设计的模块化网页截图工具

pub mod cli;
pub mod config;
pub mod file_io;
pub mod models;
pub mod screenshot;
pub mod utils;

pub use cli::cli_parser::CliParser;
pub use config::app_config::AppConfig;
pub use file_io::file_operations::FileOperations;
pub use models::target::{ScreenshotResult, Target, TargetType};
pub use screenshot::{
    async_screenshot_service::AsyncScreenshotService, concurrent_executor::ConcurrentExecutor,
    screenshot_service::ScreenshotService,
};
