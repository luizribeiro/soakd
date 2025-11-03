use crate::config;
use futures::stream::AbortHandle;
use macros::mqtt_handler;
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

struct MQTTHandler {
    pattern: String,
    handler: Box<MQTTHandlerFn>,
}

lazy_static! {
    static ref HANDLERS: Mutex<Vec<MQTTHandler>> = Mutex::new(Vec::new());
}

fn register_handler(pattern: &str, handler: Box<MQTTHandlerFn>) {
    let mut handlers = HANDLERS.lock().unwrap();
    handlers.push(MQTTHandler {
        pattern: pattern.to_string(),
        handler,
    });
}

fn topic_matches(topic: &str, pattern: &str) -> bool {
    let mut topic_parts = topic.split('/');
    let mut pattern_parts = pattern.split('/');
    loop {
        let topic_part = topic_parts.next();
        let pattern_part = pattern_parts.next();
        match (topic_part, pattern_part) {
            (Some(_), Some("+")) => continue,
            (Some(_), Some("#")) => return true,
            (Some(t), Some(p)) if t == p => continue,
            (None, None) => return true,
            _ => return false,
        }
    }
}

#[allow(clippy::await_holding_lock)]
pub async fn handle_message(
    current_task_handle: &mut Option<AbortHandle>,
    config: &config::Configuration,
    topic: &str,
    payload: &str,
) {
    let handlers = HANDLERS.lock().unwrap();
    for handler in handlers.iter() {
        if topic_matches(topic, &handler.pattern) {
            return (handler.handler)(current_task_handle, config, topic, payload).await;
        }
    }
    println!("Message on unexpected topic: {}", topic);
}
