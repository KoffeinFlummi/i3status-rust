use std::fs::File;
use std::io::{BufRead, BufReader};
use std::time::Duration;

use crossbeam_channel::Sender;
use serde_derive::Deserialize;

use crate::blocks::{Block, ConfigBlock, Update};
use crate::config::Config;
use crate::de::deserialize_duration;
use crate::errors::*;
use crate::input::I3BarEvent;
use crate::scheduler::Task;
use crate::util::pseudo_uuid;
use crate::widget::{I3BarWidget, State};
use crate::widgets::text::TextWidget;

pub struct Killswitch {
    text: TextWidget,
    id: String,
    update_interval: Duration,

    //useful, but optional
    #[allow(dead_code)]
    config: Config,
    #[allow(dead_code)]
    tx_update_request: Sender<Task>,
}

#[derive(Deserialize, Debug, Default, Clone)]
#[serde(deny_unknown_fields)]
pub struct KillswitchConfig {
    /// Update interval in seconds
    #[serde(
        default = "KillswitchConfig::default_interval",
        deserialize_with = "deserialize_duration"
    )]
    pub interval: Duration,
}

impl KillswitchConfig {
    fn default_interval() -> Duration {
        Duration::from_secs(5)
    }
}

impl ConfigBlock for Killswitch {
    type Config = KillswitchConfig;

    fn new(
        block_config: Self::Config,
        config: Config,
        tx_update_request: Sender<Task>,
    ) -> Result<Self> {
        Ok(Killswitch {
            id: pseudo_uuid().to_string(),
            update_interval: block_config.interval,
            text: TextWidget::new(config.clone())
                .with_text("")
                .with_icon("killswitch"),
            tx_update_request,
            config,
        })
    }
}

fn is_killswitch_active() -> bool {
    let file = File::open("/etc/default/ufw").unwrap();
    for line in BufReader::new(file).lines() {
        if line.unwrap() == "DEFAULT_OUTPUT_POLICY=\"DROP\"" {
            return true;
        }
    }

    false
}

impl Block for Killswitch {
    fn update(&mut self) -> Result<Option<Update>> {
        let active = is_killswitch_active();

        self.text
            .set_text(if active { "" } else { "down" }.to_string());
        self.text
            .set_state(if active { State::Good } else { State::Critical });

        Ok(Some(self.update_interval.into()))
    }

    fn view(&self) -> Vec<&dyn I3BarWidget> {
        vec![&self.text]
    }

    fn click(&mut self, _: &I3BarEvent) -> Result<()> {
        Ok(())
    }

    fn id(&self) -> &str {
        &self.id
    }
}
