use crate::config;
use crate::driver;
use futures::{future::abortable, stream::AbortHandle};

pub async fn handle_message(
    current_task_handle: &mut Option<AbortHandle>,
    config: &config::Configuration,
    topic: &str,
    _payload: &str,
) {
    if current_task_handle.is_some() {
        println!("Already have an ongoing sprinklers task. Ignoring.");
    }

    let plan_name = &topic["sprinklers/start_plan/".len()..];
    let plan = config.plans.iter().find(|p| p.name == plan_name);

    if let Some(plan) = plan {
        let config = config.clone();
        let plan = plan.clone();
        let (task, handle) = abortable(async move {
            start_plan(&config, &plan).await;
        });
        tokio::spawn(task);
        *current_task_handle = Some(handle);
    } else {
        println!("Unknown plan: {}", plan_name);
    }
}

async fn start_plan(config: &config::Configuration, plan: &config::SprinklerPlan) {
    for zone_duration in &plan.zone_durations {
        let zone_config = config
            .zones
            .iter()
            .find(|z| z.zone == zone_duration.zone)
            .unwrap();
        println!(
            "Activating zone {} for {} minutes",
            zone_config.zone, zone_duration.duration
        );
        driver::activate_zone(&config.pump, &zone_config, zone_duration.duration.into()).await;
        println!("Done watering zone {}", zone_config.zone);
    }
}
