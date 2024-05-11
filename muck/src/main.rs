mod muck;
mod status_gossiper;

use std::time::{SystemTime, UNIX_EPOCH};

use muck::muck::{Muck, MuckConfig};

use tracing_subscriber;

fn main() {
    tracing_subscriber::fmt::init();

    let seed_nodes = vec![status_gossiper::Node {
        id: "hello".to_string(),
        address: "0.0.0.0:8001".to_string(),
        sent_at: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        status: "ok".to_string(),
    }];

    let muck1 = Muck {
        config: MuckConfig {
            name: "My Mucky Muck".to_string(),
        },
        watcher: status_gossiper::Watcher::new(
            status_gossiper::Node {
                id: "hello".to_string(),
                address: "0.0.0.0:8002".to_string(),
                sent_at: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                status: "ok".to_string(),
            },
            seed_nodes,
            10,
            1,
        ),
    };

    muck1.start();
}
