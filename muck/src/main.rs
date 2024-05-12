mod muck;
mod status_gossiper;

use std::{
    thread,
    time::{SystemTime, UNIX_EPOCH},
};

use muck::muck::{Muck, MuckConfig};

use tracing_subscriber;

fn main() {
    tracing_subscriber::fmt::init();

    // let muck1 = Muck {
    //     config: MuckConfig {
    //         name: "My Mucky Muck".to_string(),
    //     },
    //     watcher: status_gossiper::Watcher::new(
    //         status_gossiper::NodeInfo {
    //             id: "hello".to_string(),
    //             address: "0.0.0.0:8002".to_string(),
    //             sent_at: SystemTime::now()
    //                 .duration_since(UNIX_EPOCH)
    //                 .unwrap()
    //                 .as_secs(),
    //             generation: 0,
    //             version: 1,
    //             status: "ok".to_string(),
    //         },
    //         seed_nodes,
    //         10,
    //         1,
    //     ),
    // };

    for i in 0..10 {
        let _ = thread::spawn(move || {
            let seed_nodes = vec!["0.0.0.0:8001".to_string()];

            let id = i.to_string();
            let port = 8000 + i;
            let address = "0.0.0.0:".to_string() + &port.to_string();

            let muck = Muck {
                config: MuckConfig { name: id.clone() },
                watcher: status_gossiper::Watcher::new(
                    status_gossiper::NodeInfo {
                        id: id.clone(),
                        address: address.clone(),
                        sent_at: SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_secs(),
                        generation: 0,
                        version: 1,
                        status: "ok".to_string(),
                    },
                    seed_nodes,
                    10,
                    3,
                ),
            };

            muck.start()
        });
    }

    loop {}
}
