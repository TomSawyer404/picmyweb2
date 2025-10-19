use futures::stream::{self, StreamExt};
use indicatif::{ProgressBar, ProgressStyle};
use std::sync::Arc;
use tokio::sync::Semaphore;

use crate::config::app_config::AppConfig;
use crate::models::target::Target;
use crate::screenshot::async_screenshot_service::AsyncScreenshotService;

/// 并发执行器
pub struct ConcurrentExecutor {
    semaphore: Arc<Semaphore>,
    screenshot_service: Arc<AsyncScreenshotService>,
}

impl ConcurrentExecutor {
    pub fn new(config: AppConfig) -> Self {
        let concurrency = config.concurrency;
        Self {
            semaphore: Arc::new(Semaphore::new(concurrency)),
            screenshot_service: Arc::new(AsyncScreenshotService::new(config)),
        }
    }

    /// 并发执行截图任务，实时返回每个任务的结果
    pub async fn execute_concurrent_screenshots<F>(
        &self,
        targets: Vec<Target>,
        mut on_result: F,
    ) -> (usize, usize)
    where
        F: FnMut(
                &Target,
                Result<
                    crate::models::target::ScreenshotResult,
                    Box<dyn std::error::Error + Send + Sync>,
                >,
            ) + Send
            + Sync
            + 'static,
    {
        let total_tasks = targets.len();

        // 创建单个进度条
        let progress_bar = ProgressBar::new(total_tasks as u64);
        let style = ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta}) 成功: {msg}")
            .unwrap_or_else(|_| ProgressStyle::default_bar())
            .progress_chars("█▓▒░");
        progress_bar.set_style(style);
        progress_bar.set_message("0");

        let success_count = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let fail_count = Arc::new(std::sync::atomic::AtomicUsize::new(0));

        // 保存原始目标的引用，避免移动后无法访问
        let targets_arc: Vec<Arc<Target>> = targets
            .iter()
            .map(|target| Arc::new(target.clone()))
            .collect();

        // 使用流处理任务，实现实时更新
        let tasks = stream::iter(targets_arc.clone().into_iter().enumerate())
            .map(|(index, target_arc)| {
                let target_arc_clone = Arc::clone(&target_arc);
                let semaphore = Arc::clone(&self.semaphore);
                let screenshot_service = Arc::clone(&self.screenshot_service);
                let success_count_clone = Arc::clone(&success_count);
                let fail_count_clone = Arc::clone(&fail_count);

                async move {
                    // 获取信号量许可
                    let permit_result = semaphore.acquire().await;
                    let _permit = match permit_result {
                        Ok(permit) => permit,
                        Err(e) => {
                            eprintln!("获取信号量许可失败: {}", e);
                            return (
                                index,
                                target_arc,
                                Err(Box::new(std::io::Error::new(
                                    std::io::ErrorKind::Other,
                                    "获取并发许可失败",
                                ))
                                    as Box<dyn std::error::Error + Send + Sync>),
                            );
                        }
                    };

                    let result = screenshot_service
                        .take_screenshot_async(target_arc_clone)
                        .await;

                    // 更新计数器
                    match &result {
                        Ok(screenshot_result) => {
                            if screenshot_result.success {
                                success_count_clone
                                    .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                            } else {
                                fail_count_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                            }
                        }
                        Err(_) => {
                            fail_count_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                        }
                    }

                    (index, target_arc, result)
                }
            })
            .buffer_unordered(self.semaphore.available_permits());

        // 使用for_each处理任务流，实现实时更新
        use futures::pin_mut;
        pin_mut!(tasks);

        let mut completed_count = 0;
        while let Some((_index, target, result)) = tasks.next().await {
            completed_count += 1;

            // 更新进度条
            progress_bar.inc(1);
            let current_success = success_count.load(std::sync::atomic::Ordering::SeqCst);
            progress_bar.set_message(current_success.to_string());

            // 调用回调函数
            on_result(&target, result);
        }

        // 完成进度条
        let final_success = success_count.load(std::sync::atomic::Ordering::SeqCst);
        let final_fail = fail_count.load(std::sync::atomic::Ordering::SeqCst);

        progress_bar.finish_with_message(format!(
            "完成! 成功: {} 失败: {}",
            final_success, final_fail
        ));

        (final_success, final_fail)
    }

    /// 简化的并发执行方法，只显示基本进度信息
    pub async fn execute_concurrent_screenshots_simple(
        &self,
        targets: Vec<Target>,
    ) -> (usize, usize) {
        let total_tasks = targets.len();

        // 创建简单的进度条
        let progress_bar = ProgressBar::new(total_tasks as u64);
        let style = ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta}) 成功: {msg}")
            .unwrap_or_else(|_| ProgressStyle::default_bar())
            .progress_chars("█▓▒░");
        progress_bar.set_style(style);
        progress_bar.set_message("0");

        let success_count = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let fail_count = Arc::new(std::sync::atomic::AtomicUsize::new(0));

        let targets_arc: Vec<Arc<Target>> = targets
            .iter()
            .map(|target| Arc::new(target.clone()))
            .collect();

        // 使用流处理任务，实现实时更新
        let tasks = stream::iter(targets_arc.into_iter())
            .map(|target_arc| {
                let target_arc_clone = Arc::clone(&target_arc);
                let semaphore = Arc::clone(&self.semaphore);
                let screenshot_service = Arc::clone(&self.screenshot_service);
                let success_count_clone = Arc::clone(&success_count);
                let fail_count_clone = Arc::clone(&fail_count);

                async move {
                    // 获取信号量许可
                    let permit_result = semaphore.acquire().await;
                    let _permit = match permit_result {
                        Ok(permit) => permit,
                        Err(e) => {
                            eprintln!("获取信号量许可失败: {}", e);
                            return Err(Box::new(std::io::Error::new(
                                std::io::ErrorKind::Other,
                                "获取并发许可失败",
                            ))
                                as Box<dyn std::error::Error + Send + Sync>);
                        }
                    };

                    let result = screenshot_service
                        .take_screenshot_async(target_arc_clone)
                        .await;

                    // 更新计数器
                    match &result {
                        Ok(screenshot_result) => {
                            if screenshot_result.success {
                                success_count_clone
                                    .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                            } else {
                                fail_count_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                            }
                        }
                        Err(_) => {
                            fail_count_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                        }
                    }

                    result
                }
            })
            .buffer_unordered(self.semaphore.available_permits());

        // 使用for_each处理任务流，实现实时更新
        use futures::pin_mut;
        pin_mut!(tasks);

        while let Some(_) = tasks.next().await {
            let current_success = success_count.load(std::sync::atomic::Ordering::SeqCst);

            progress_bar.inc(1);
            progress_bar.set_message(current_success.to_string());
        }

        let final_success = success_count.load(std::sync::atomic::Ordering::SeqCst);
        let final_fail = fail_count.load(std::sync::atomic::Ordering::SeqCst);

        progress_bar.finish_with_message(format!(
            "完成! 成功: {} 失败: {}",
            final_success, final_fail
        ));

        (final_success, final_fail)
    }
}
