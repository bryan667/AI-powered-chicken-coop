use tokio::time::{sleep, Duration};

pub async fn run_scheduler() {
    loop {
        println!("Running scheduled tasks...");
        sleep(Duration::from_secs(3600)).await; // hourly task
    }
}
