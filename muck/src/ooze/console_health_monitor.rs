use std::thread;
use std::time::Duration;

use tracing::info;

use super::{Ooze, OozeError};

pub struct ConsoleHealthMonitoring;

impl Ooze for ConsoleHealthMonitoring {
    fn run(&self) -> Result<thread::JoinHandle<()>, OozeError> {
        let handle = thread::spawn(move || loop {
            info!("Muck is healthy!");
            thread::sleep(Duration::from_secs(1));
        });
        Ok(handle)
    }
}
