use crate::config;
use crate::driver;
use futures::stream::AbortHandle;

pub async fn handle_message(
    current_task_handle: &mut Option<AbortHandle>,
    config: &config::Configuration,
    _topic: &str,
    _payload: &str,
) {
    if let Some(handle) = current_task_handle {
        println!("Stopping sprinklers");
        handle.abort();
        driver::shutoff_all_valves(&config);
        *current_task_handle = None;
    } else {
        println!("No ongoing sprinklers task to stop.");
    }
}
