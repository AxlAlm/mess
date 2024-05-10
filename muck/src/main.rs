mod muck;
mod ooze;
mod registration;

use muck::muck::{Muck, MuckConfig};

use tracing_subscriber;

fn main() {
    tracing_subscriber::fmt::init();

    let muck1 = Muck {
        config: MuckConfig {
            name: "My Mucky Muck".to_string(),
        },
        registration: Box::new(registration::DummyRegistration),
        oozes: vec![Box::new(
            ooze::console_health_monitor::ConsoleHealthMonitoring,
        )],
    };

    muck1.start();
}
