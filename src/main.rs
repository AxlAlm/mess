mod muck;
use muck::muck::{Muck, MuckConfig, Ooze, Registration};

use std::thread;
use std::time::Duration;

pub struct DummyRegistration;

impl Registration for DummyRegistration {
    fn register(&self) {
        dbg!("Registration done!");
    }

    fn deregister(&self) {
        dbg!("Deregistration done!");
    }
}

pub struct ConsoleHealthMonitoring;

impl Ooze for ConsoleHealthMonitoring {
    fn run(&self) -> thread::JoinHandle<()> {
        thread::spawn(move || loop {
            thread::sleep(Duration::from_secs(1));
        })
    }
}

fn main() {
    let muck1 = Muck {
        config: MuckConfig {
            name: "Service One".to_string(),
        },
        registration: Box::new(DummyRegistration),
        oozes: vec![Box::new(ConsoleHealthMonitoring)],
    };

    muck1.start();
}
