use crate::status_gossiper;
use std::thread;
use std::time::Duration;

use tracing::info;

pub struct Muck {
    pub config: MuckConfig,
    pub watcher: status_gossiper::Watcher,
}

pub struct MuckConfig {
    pub name: String,
}

impl Muck {
    pub fn start(&self) {
        // let span = span!(Level::INFO, "muck", id = "123");
        // let _enter = span.enter();
        info!(name:"ok", what="ok", "Muck {} is staring ...", self.config.name);
        let _ = self.watcher.run();

        info!("Muck {} started successfully", self.config.name);
        loop {
            info!("Main thread is still running...");
            thread::sleep(Duration::from_secs(5));
        }
    }
}
