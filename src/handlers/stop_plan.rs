use crate::config;
use crate::driver;
use futures::stream::AbortHandle;

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
