use rand::Rng;
use rand::{seq::SliceRandom, thread_rng};
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::io::ErrorKind;
use std::net::UdpSocket;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::{thread, time};

type NodeID = String;

fn now_unix() -> u64 {
    return time::SystemTime::now()
        .duration_since(time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HeartBeat {
    pub id: NodeID,
    pub address: String,
    pub generation: u64,
    pub version: u64,
    pub timestamp: u64,
}

impl Default for HeartBeat {
    fn default() -> Self {
        HeartBeat {
            id: "".to_string(),
            address: "".to_string(),
            generation: 0,
            version: 0,
            timestamp: now_unix(),
        }
    }
}

// store a counter for how many
type HeartBeatReceivedCount = Arc<Mutex<HashMap<NodeID, u64>>>;

// contains the counts a particular hartbeat
type HeartBeatSentCounts = Arc<Mutex<HashMap<NodeID, HashMap<NodeID, u64>>>>;
type NodeHeartBeats = Arc<Mutex<HashMap<String, HeartBeat>>>;

trait HeartBeatChannel {
    fn send_heartbeat(
        &self,
        heartbeat: HeartBeat,
        target_addresses: Vec<String>,
    ) -> Result<(), String>;
}

struct HeartBeatUdapChannel {
    socket: Arc<Mutex<UdpSocket>>,
}

impl HeartBeatChannel for HeartBeatUdapChannel {
    fn send_heartbeat(
        &self,
        heartbeat: HeartBeat,
        target_addresses: Vec<String>,
    ) -> Result<(), String> {
        let msg = serde_json::to_string(&heartbeat).map_err(|e| e.to_string())?;
        let locked_socket = self.socket.lock().map_err(|e| e.to_string())?;
        for address in target_addresses {
            locked_socket
                .send_to(msg.as_bytes(), address)
                .map_err(|e| e.to_string())?;
        }
        Ok(())
    }
}

trait Addresses {
    fn get(&self) -> Vec<String>;
}

impl Addresses for NodeHeartBeats {
    fn get(&self) -> Vec<String> {
        return vec!["123".to_string()];
    }
}

type VersionStore = Arc<Mutex<u64>>;

pub struct Watcher {
    node: HeartBeat,
    node_info_table: NodeHeartBeats,
    gossip_interval: u64,
    gossip_to_n_nodes: usize,
}

impl Watcher {
    pub fn new(
        node: HeartBeat,
        seed_node_addresses: Vec<String>,
        gossip_interval: u64,
        gossip_to_n_nodes: usize,
    ) -> Self {
        let node_heartbeats: NodeHeartBeats = Arc::new(Mutex::new(HashMap::new()));

        for address in &seed_node_addresses {
            table.insert(
                address.to_string(),
                NodeInfoRow {
                    n_times_recevied: 0,
                    node_info: HeartBeat {
                        address: address.to_string(),
                        ..Default::default()
                    },
                    send_node_counts: HashMap::new(),
                },
            );
        }

        // table.insert(
        //     node.id.to_string(),
        //     NodeInfoRow {
        //         n_times_recevied: 0,
        //         node_info: node.clone(),
        //         send_node_counts: HashMap::new(),
        //     },
        // );

        Watcher {
            node,
            node_info_table: shared_table,
            gossip_interval,
            gossip_to_n_nodes,
        }
    }

    pub fn run(&self) -> Result<(), String> {
        let heartbeat_interval_secs = self.gossip_interval;
        let node_info_table = self.node_info_table.clone();
        let gossip_to_n_nodes = self.gossip_to_n_nodes;
        let node = self.node.clone();

        let address = "123";

        let poll_interval_milisecs = 10;

        let socket = UdpSocket::bind(&self.node.address).map_err(|e| e.to_string())?;
        socket.set_nonblocking(true).map_err(|e| e.to_string())?;
        let heartbeat_channel = HeartBeatUdapChannel {
            socket: Arc::new(Mutex::new(socket)),
        };

        let node_heartbeats: NodeHeartBeats = Arc::new(Mutex::new(HashMap::new()));

        let _ = thread::spawn(move || {
            periodic_heartbeat(
                node.id,
                address.to_string(),
                1,
                heartbeat_interval_secs,
                gossip_to_n_nodes,
                &node_heartbeats,
                &heartbeat_channel,
            )
        });

        // let _ = thread::spawn(move || {
        //     periodic_heartbeat(
        //         node.id,
        //         address.to_string(),
        //         1,
        //         heartbeat_interval_secs,
        //         gossip_to_n_nodes,
        //         &node_heartbeats,
        //         &heartbeat_channel,
        //     )
        // });

        let s_table = node_info_table.clone();
        let s_socket = shared_socket.clone();
        let _ = thread::spawn(move || {
            gossip_process(poll_interval_milisecs, gossip_to_n_nodes, s_table, s_socket)
        });

        let s_table = node_info_table.clone();
        let _ = thread::spawn(move || loop {
            thread::sleep(Duration::from_millis(1000));
            println!("Nodes stored: {:?}/10", s_table.lock().unwrap().len())
        });

        Ok(())
    }

    pub fn update_status() -> Result<(), String> {
        Ok(())
    }
}

fn periodic_heartbeat(
    node_id: NodeID,
    address: String,
    generation: u64,
    heartbeat_interval_secs: u64,
    heartbeat_spread: usize,
    addresses: &dyn Addresses,
    heartbeat_channel: &dyn HeartBeatChannel,
) {
    let mut version = 0;
    loop {
        version += 1;
        let heartbeat = HeartBeat {
            id: node_id.clone(),
            address: address.clone(),
            generation,
            version,
            timestamp: now_unix(),
        };

        let target_addresses = select_random_n_strings(addresses.get(), heartbeat_spread);
        match heartbeat_channel.send_heartbeat(heartbeat, target_addresses) {
            Ok(_) => thread::sleep(Duration::from_secs(heartbeat_interval_secs)),
            Err(_) => println!("failed to send shit"),
        };
    }
}

fn gossip_process(
    poll_interval_milisecs: u64,
    gossip_to_n_nodes: usize,
    node_info_table: NodeHeartBeats,
    socket: Arc<Mutex<UdpSocket>>,
) {
    loop {
        thread::sleep(Duration::from_millis(poll_interval_milisecs));

        let l_socket = socket.lock().unwrap();
        let mut buf = [0; 256];
        let (size, _src) = match l_socket.recv_from(&mut buf) {
            Ok(data) => data,
            Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                continue;
            }
            Err(_) => {
                println!("Could not read the UDP packet");
                continue;
            }
        };

        let received_node_info = match serde_json::from_slice::<HeartBeat>(&buf[..size]) {
            Ok(node) => node,
            Err(_) => {
                println!("Failed to parse JSON");
                continue;
            }
        };

        let table = &mut node_info_table.lock().unwrap();
        let n_times_received = match table.get(&received_node_info.id) {
            Some(current) => {
                let is_new = received_node_info.generation > current.node_info.generation
                    || received_node_info.version > current.node_info.version;

                if is_new {
                    0
                } else {
                    current.n_times_recevied + 1
                }
            }
            None => 0,
        };

        if !should_forward(n_times_received) {
            continue;
        }

        // if n_times_received > 0 {
        //     continue;
        // }

        table.insert(
            received_node_info.id.to_string(),
            NodeInfoRow {
                n_times_recevied: n_times_received,
                node_info: received_node_info.clone(),
            },
        );

        let addresses: Vec<String> = table
            .iter()
            .map(|(_, v)| v.node_info.address.clone())
            .collect();
        let selected_addresses = select_random_n_strings(addresses, gossip_to_n_nodes);
        let msg = match serde_json::to_string(&received_node_info) {
            Ok(s) => s,
            Err(_) => {
                println!("Failed to serialize node");
                continue;
            }
        };

        match send_msg(msg, selected_addresses, &l_socket) {
            Ok(_) => {
                println!("sending update");
            }
            Err(e) => {
                println!("{}", e.to_string());
            }
        }
    }
}

fn select_random_n_strings(a: Vec<String>, n: usize) -> Vec<String> {
    let mut a = a;
    let mut rng = thread_rng();
    a.shuffle(&mut rng);

    if a.len() < n {
        return a;
    }
    a[..n].to_vec()
}

fn should_forward(n_times_receieved: u64) -> bool {
    let base_probability = 1.0; // Starting probability of forwarding
    let decay_factor = 0.3; // Adjust this factor based on your needs
    let probability = base_probability * f64::exp(-decay_factor * n_times_receieved as f64);
    let mut rng = rand::thread_rng();
    rng.gen::<f64>() < probability
}
