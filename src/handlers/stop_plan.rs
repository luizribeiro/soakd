use crate::config;
use crate::driver;
use crate::handlers::mqtt_handler;
use futures::stream::AbortHandle;

#[mqtt_handler(topic = "+/stop")]
pub async fn handle_message(
    current_task_handle: &mut Option<AbortHandle>,
    _config: &config::Configuration,
    _topic: &str,
    _payload: &str,
) {
    if let Some(handle) = current_task_handle {
        println!("Stopping sprinklers");
        handle.abort();
        driver::get_driver().lock().await.shutoff_all_valves();
        *current_task_handle = None;
    } else {
        println!("No ongoing sprinklers task to stop.");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::future::abortable;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_stop_handler_aborts_task() {
        // Initialize driver with noop driver
        let config = config::Configuration {
            mqtt: config::MQTTConfig {
                broker: "localhost".to_string(),
                port: 1883,
                client_id: "test".to_string(),
                topic_prefix: "test".to_string(),
            },
            driver: config::Driver::Noop,
            pump: config::PumpConfig { pin: 0, delay: 1 },
            zones: vec![],
            plans: vec![],
        };

        driver::init_driver(&config).await;

        // Create a long-running abortable task
        let (task, handle) = abortable(async {
            sleep(Duration::from_secs(10)).await;
        });

        let task_handle = tokio::spawn(task);
        let mut current_task_handle = Some(handle);

        // Call stop handler
        handle_message(&mut current_task_handle, &config, "test/stop", "").await;

        // Verify handle was cleared
        assert!(current_task_handle.is_none());

        // Verify task was aborted
        let result = task_handle.await;
        assert!(result.is_ok()); // Task completed
        assert!(result.unwrap().is_err()); // But was aborted
    }

    #[tokio::test]
    async fn test_stop_handler_with_no_running_task() {
        let config = config::Configuration {
            mqtt: config::MQTTConfig {
                broker: "localhost".to_string(),
                port: 1883,
                client_id: "test".to_string(),
                topic_prefix: "test".to_string(),
            },
            driver: config::Driver::Noop,
            pump: config::PumpConfig { pin: 0, delay: 1 },
            zones: vec![],
            plans: vec![],
        };

        driver::init_driver(&config).await;

        let mut current_task_handle = None;

        // Call stop handler with no running task
        handle_message(&mut current_task_handle, &config, "test/stop", "").await;

        // Should not panic, handle remains None
        assert!(current_task_handle.is_none());
    }
}
