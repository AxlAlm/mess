use rand::{seq::SliceRandom, thread_rng};
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::io::ErrorKind;
use std::net::UdpSocket;
use std::thread;
use std::time::{Duration, Instant};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Node {
    pub id: String,
    pub address: String,
    pub status: String,
    pub sent_at: u64,
}

#[derive(Clone)]
struct NodeTable {
    nodes: HashMap<String, Node>,
}

pub struct Watcher {
    node: Node,
    node_info_table: NodeTable, // Shared NodeTable
    gossip_interval: u64,
    gossip_to_n_nodes: usize,
}

impl Watcher {
    pub fn new(
        node: Node,
        seed_nodes: Vec<Node>,
        gossip_interval: u64,
        gossip_to_n_nodes: usize,
    ) -> Self {
        let mut table: NodeTable = NodeTable {
            nodes: HashMap::new(),
        };

        for node in &seed_nodes {
            table.nodes.insert(node.id.clone(), node.clone());
        }

        Watcher {
            node,
            node_info_table: table,
            gossip_interval,
            gossip_to_n_nodes,
        }
    }

    pub fn run(&self) -> Result<(), String> {
        let heartbeat_interval_secs = self.gossip_interval;
        let mut table = self.node_info_table.clone();
        let gossip_to_n_nodes = self.gossip_to_n_nodes;
        let node = self.node.clone();
        let poll_interval_milisecs = 1000;

        let socket = UdpSocket::bind(&self.node.address).map_err(|e| e.to_string())?;
        socket.set_nonblocking(true).map_err(|e| e.to_string())?;

        let _ = thread::spawn(move || {
            let mut last_update_sent_at = Instant::now();

            loop {
                // poll interval
                thread::sleep(Duration::from_millis(poll_interval_milisecs));

                if last_update_sent_at.elapsed() >= Duration::from_secs(heartbeat_interval_secs) {
                    let node = table.nodes.get(&node.id);
                    if node.is_none() {
                        continue;
                    }
                    let node = node.unwrap();

                    let selected_addresses =
                        select_random_node_addresses(&table, gossip_to_n_nodes);

                    let msg = serde_json::to_string(node);
                    if msg.is_err() {
                        println!("Faild serialize node");
                        continue;
                    }

                    if send_msg(msg.unwrap(), selected_addresses, &socket).is_err() {
                        println!("failed to send node");
                        continue;
                    }

                    println!("Succefully sent node info");
                    last_update_sent_at = Instant::now()
                }

                let mut buf = [0; 256];
                let (size, _src) = match socket.recv_from(&mut buf) {
                    Ok(data) => data,
                    Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                        println!("No packets available yet.");
                        continue;
                    }
                    Err(_) => {
                        println!("Could not read the UDP packet");
                        continue;
                    }
                };

                let received_node_info = match serde_json::from_slice::<Node>(&buf[..size]) {
                    Ok(node) => node,
                    Err(_) => {
                        println!("Failed to parse JSON");
                        continue;
                    }
                };

                let should_insert = table
                    .nodes
                    .get(&received_node_info.id)
                    .map_or(true, |node| received_node_info.sent_at > node.sent_at);

                if !should_insert {
                    println!("Nothing to update");
                    continue;
                }
                table.nodes.insert(
                    received_node_info.id.to_string(),
                    received_node_info.clone(),
                );

                let msg = match serde_json::to_string(&received_node_info) {
                    Ok(s) => s,
                    Err(_) => {
                        println!("Failed to serialize node");
                        continue;
                    }
                };
                let selected_addresses = select_random_node_addresses(&table, gossip_to_n_nodes);
                if send_msg(msg, selected_addresses, &socket).is_err() {
                    continue;
                }
            }
        });

        Ok(())
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

fn select_random_node_addresses(table: &NodeTable, n: usize) -> Vec<String> {
    let mut target_addresses: Vec<String> =
        table.nodes.iter().map(|(_, v)| v.address.clone()).collect();
    let mut rng = thread_rng();
    target_addresses.shuffle(&mut rng);
    target_addresses[..n].to_vec()
}
