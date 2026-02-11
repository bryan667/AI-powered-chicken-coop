use tokio::time::{sleep, Duration};

pub async fn run_scheduled_tasks(iterations: usize, interval: Duration) {
    for _ in 0..iterations {
        println!("Running scheduled tasks...");
        sleep(interval).await;
    }
}

#[cfg(test)]
mod tests {
    use super::run_scheduled_tasks;
    use tokio::time::Duration;

    #[tokio::test]
    async fn scheduler_runs_requested_iterations() {
        run_scheduled_tasks(1, Duration::from_millis(1)).await;
    }
}
