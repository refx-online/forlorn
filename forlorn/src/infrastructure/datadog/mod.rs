use dogstatsd::{Client, OptionsBuilder};

use crate::config::DatadogConfig;

pub fn create_metric(config: DatadogConfig) -> Client {
    let datadog_url = format!("{}:{}", config.host, config.port);

    let options = OptionsBuilder::new()
        .to_addr(datadog_url)
        .namespace(config.namespace)
        .build();

    Client::new(options).expect("failed to init dogstatsd client")
}
