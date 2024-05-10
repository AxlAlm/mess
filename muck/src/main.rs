mod muck;
mod status_gossiper;

use muck::muck::{Muck, MuckConfig};

use tracing_subscriber;

fn main() {
    tracing_subscriber::fmt::init();

    let muck1 = Muck {
        config: MuckConfig {
            name: "My Mucky Muck".to_string(),
        },
        status_gossiper: Box::new(status_gossiper::)
    };

    muck1.start();
}
