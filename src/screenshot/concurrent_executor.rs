use futures::future::join_all;
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
        let mut tasks = Vec::new();
        let mut success_count = 0;
        let mut fail_count = 0;

        // 保存原始目标的引用，避免移动后无法访问
        let targets_arc: Vec<Arc<Target>> = targets
            .iter()
            .map(|target| Arc::new(target.clone()))
            .collect();

        // 为每个目标创建任务
        for target_arc in targets_arc.iter() {
            let target_arc_clone = Arc::clone(target_arc);
            let semaphore = Arc::clone(&self.semaphore);
            let screenshot_service = Arc::clone(&self.screenshot_service);

            let task = async move {
                let _permit = semaphore.acquire_owned().await.unwrap();
                screenshot_service
                    .take_screenshot_async(target_arc_clone)
                    .await
            };

            tasks.push(task);
        }

        // 使用join_all等待所有任务完成
        let results = join_all(tasks).await;

        // 处理每个结果并调用回调
        for (i, result) in results.into_iter().enumerate() {
            let target = &targets_arc[i];

            // 先进行统计，避免使用clone
            match &result {
                Ok(screenshot_result) => {
                    if screenshot_result.success {
                        success_count += 1;
                    } else {
                        fail_count += 1;
                    }
                }
                Err(_) => {
                    fail_count += 1;
                }
            }

            // 调用回调函数
            on_result(target, result);
        }

        (success_count, fail_count)
    }
}
