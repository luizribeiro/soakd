use crate::config;
use futures::StreamExt;
use std::time::Duration;

pub struct MQTTClient {
    pub client: paho_mqtt::AsyncClient,
    pub stream: paho_mqtt::AsyncReceiver<Option<paho_mqtt::Message>>,
}

impl MQTTClient {
    pub async fn new(config: &config::Configuration) -> Result<Self, std::io::Error> {
        let create_opts = paho_mqtt::CreateOptionsBuilder::new()
            .server_uri(format!("tcp://{}:{}", config.mqtt.broker, config.mqtt.port))
            .client_id("sprinkler_controller")
            .finalize();

        let mut client = paho_mqtt::AsyncClient::new(create_opts)?;
        let stream = client.get_stream(25);

        let conn_opts = paho_mqtt::ConnectOptionsBuilder::new()
            .keep_alive_interval(Duration::from_secs(20))
            .clean_session(true)
            .finalize();

        client.connect(conn_opts).await?;

        client.subscribe("sprinklers/#", 1).await?;

        Ok(Self { client, stream })
    }

    pub async fn next(&mut self) -> Result<paho_mqtt::Message, std::io::Error> {
        // TODO: clean up this mess
        let msg_opt = self.stream.next().await;
        if let Some(Some(message)) = msg_opt {
            Ok(message)
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "MQTT stream closed",
            ))
        }
    }
}
