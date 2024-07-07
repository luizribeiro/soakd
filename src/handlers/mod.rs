use crate::config;
use futures::stream::AbortHandle;
use macros::mqtt_handler;
use regex::Regex;
use std::future::Future;
use std::pin::Pin;
use std::sync::Mutex;

pub mod start_plan;
pub mod stop_plan;
pub mod water_zone;

type MQTTHandlerFn = dyn for<'a> Fn(
        &'a mut Option<AbortHandle>,
        &'a config::Configuration,
        &'a str,
        &'a str,
    ) -> Pin<Box<dyn Future<Output = ()> + 'a>>
    + Sync
    + Send;

lazy_static! {
    static ref HANDLERS: Mutex<Vec<(Regex, Box<MQTTHandlerFn>)>> = Mutex::new(Vec::new());
}

#[mqtt_handler(topic = "foo/bar")]
async fn foo_bar_handler(
    _current_task_handle: &mut Option<AbortHandle>,
    _config: &config::Configuration,
    topic: &str,
    payload: &str,
) {
    println!("foo_bar_handler: topic: {}, payload: {}", topic, payload);
}

fn register_handler(topic: &str, handler: Box<MQTTHandlerFn>) {
    let mut handlers = HANDLERS.lock().unwrap();
    handlers.push((Regex::new(topic).unwrap(), Box::new(handler)));
}

async fn handle_message(
    current_task_handle: &mut Option<AbortHandle>,
    config: &config::Configuration,
    topic: &str,
    payload: &str,
) {
    let handlers = HANDLERS.lock().unwrap();
    for (regex, handler) in handlers.iter() {
        if regex.is_match(topic) {
            handler(current_task_handle, config, topic, payload).await;
        }
    }
}
