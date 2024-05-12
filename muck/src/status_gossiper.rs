use rand::Rng;
use rand::{seq::SliceRandom, thread_rng};
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::io::ErrorKind;
use std::net::UdpSocket;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::{thread, time};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NodeInfo {
    pub id: String,
    pub address: String,
    pub status: String,
    pub generation: u64,
    pub version: u64,
    pub sent_at: u64,
}

impl Default for NodeInfo {
    fn default() -> Self {
        NodeInfo {
            id: "".to_string(),
            address: "".to_string(),
            status: "unknown".to_string(),
            generation: 0,
            version: 0,
            sent_at: time::SystemTime::now()
                .duration_since(time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }
}

struct NodeInfoRow {
    n_times_recevied: u64,
    node_info: NodeInfo,
    // node_addresses_sent_to: Vec<String>,
}

type NodeInfoTable = Arc<Mutex<HashMap<String, NodeInfoRow>>>;

pub struct Watcher {
    node: NodeInfo,
    node_info_table: NodeInfoTable,
    gossip_interval: u64,
    gossip_to_n_nodes: usize,
}

impl Watcher {
    pub fn new(
        node: NodeInfo,
        seed_node_addresses: Vec<String>,
        gossip_interval: u64,
        gossip_to_n_nodes: usize,
    ) -> Self {
        let mut table = HashMap::new();
        for address in &seed_node_addresses {
            table.insert(
                address.to_string(),
                NodeInfoRow {
                    n_times_recevied: 0,
                    node_info: NodeInfo {
                        address: address.to_string(),
                        ..Default::default()
                    },
                },
            );
        }

        table.insert(
            node.id.to_string(),
            NodeInfoRow {
                n_times_recevied: 0,
                node_info: node.clone(),
            },
        );
        let shared_table = Arc::new(Mutex::new(table));

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
        let poll_interval_milisecs = 10;

        let socket = UdpSocket::bind(&self.node.address).map_err(|e| e.to_string())?;
        socket.set_nonblocking(true).map_err(|e| e.to_string())?;
        let shared_socket = Arc::new(Mutex::new(socket));

        let s_table = node_info_table.clone();
        let s_socket = shared_socket.clone();
        let _ = thread::spawn(move || {
            send_periodic_node_info_process(
                &node.id,
                heartbeat_interval_secs,
                gossip_to_n_nodes,
                s_table,
                s_socket,
            )
        });

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

fn send_periodic_node_info_process(
    node_id: &str,
    heartbeat_interval_secs: u64,
    gossip_to_n_nodes: usize,
    node_info_table: NodeInfoTable,
    socket: Arc<Mutex<UdpSocket>>,
) {
    loop {
        let table = node_info_table.lock().unwrap();
        let row = match table.get(node_id) {
            Some(row) => row,
            None => continue,
        };

        let node = NodeInfo {
            id: row.node_info.id.clone(),
            address: row.node_info.address.clone(),
            version: row.node_info.version.clone() + 1,
            generation: row.node_info.generation.clone(),
            status: "OK".to_string(),
            sent_at: row.node_info.sent_at.clone(),
        };

        let msg = match serde_json::to_string(&node) {
            Ok(s) => s,
            Err(_) => {
                println!("Failed to serialize node");
                continue;
            }
        };

        let addresses: Vec<String> = table
            .iter()
            .map(|(_, v)| v.node_info.address.clone())
            .collect();
        let selected_addresses = select_random_n_strings(addresses, gossip_to_n_nodes);
        let l_socket = socket.lock().unwrap();
        match send_msg(msg, selected_addresses, &l_socket) {
            Ok(_) => {
                drop(l_socket);
                drop(table);
                thread::sleep(Duration::from_secs(heartbeat_interval_secs));
            }
            Err(e) => {
                println!("{}", e.to_string());
            }
        }
    }
}

fn gossip_process(
    poll_interval_milisecs: u64,
    gossip_to_n_nodes: usize,
    node_info_table: NodeInfoTable,
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

        let received_node_info = match serde_json::from_slice::<NodeInfo>(&buf[..size]) {
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

fn send_msg(msg: String, target_addresses: Vec<String>, socket: &UdpSocket) -> Result<(), String> {
    for address in target_addresses {
        socket
            .send_to(msg.as_bytes(), address)
            .map_err(|e| e.to_string())?;
    }
    Ok(())
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
