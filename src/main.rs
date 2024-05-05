mod muck;
mod ooze;
mod registration;

use muck::muck::{Muck, MuckConfig};

fn main() {
    let muck1 = Muck {
        config: MuckConfig {
            name: "Service One".to_string(),
        },
        registration: Box::new(registration::DummyRegistration),
        oozes: vec![Box::new(
            ooze::console_health_monitor::ConsoleHealthMonitoring,
        )],
    };

    muck1.start();
}
