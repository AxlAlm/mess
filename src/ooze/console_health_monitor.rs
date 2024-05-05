use std::thread;
use std::time::Duration;

use super::{Ooze, OozeError};

pub struct ConsoleHealthMonitoring;

impl Ooze for ConsoleHealthMonitoring {
    fn run(&self) -> Result<thread::JoinHandle<()>, OozeError> {
        let handle = thread::spawn(move || loop {
            println!("Health is Ok!");
            thread::sleep(Duration::from_secs(1));
        });
        Ok(handle)
    }
}
