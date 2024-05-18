use rand::Rng;
use rand::{seq::SliceRandom, thread_rng};
use serde::{Deserialize, Serialize};
use serde_json;
use serde_json::Error as SerdeError;
use std::collections::HashMap;
use std::fmt;
use std::io;
use std::net::UdpSocket;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::Duration;
use std::{thread, time};

trait ForwardStategy {
    fn should_forward(&self) -> bool;
}

trait HeartBeatDataGetter {
    fn get_addresses(&self) -> HashMap<NodeID, HeartBeatData>;
    fn get_received_count(&self, node_id: NodeID) -> u64;
}

trait HeartBeatDataSetter {
    fn upsert(&self, heartbeat: HeartBeat) -> Result<u64, String>;
}

trait NodeAddressSelecter {
    fn select_addresses(&self, node_heartbeat_data: HashMap<NodeID, HeartBeatData>) -> Vec<String>;
}

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

struct HeartBeatData {
    id: NodeID,
    address: String,
    generation: u64,
    version: u64,
    timestamp: u64,
    received_count: u64,
    sent_counts: HashMap<NodeID, u64>,
}

struct Storage {
    data: Arc<Mutex<HashMap<NodeID, HeartBeatData>>>,
}

trait HeartBeatChannelSender {
    fn send(&self, heartbeat: HeartBeat, target_addresses: Vec<String>) -> Result<(), String>;
}

struct HeartBeatUdapChannel {
    socket: Arc<Mutex<UdpSocket>>,
}

impl HeartBeatChannelSender for HeartBeatUdapChannel {
    fn send(&self, heartbeat: HeartBeat, target_addresses: Vec<String>) -> Result<(), String> {
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

#[derive(Debug)]
pub enum HeartBeatError {
    Io(io::Error),
    Serde(SerdeError),
    WouldBlock,
    Poison(String),
    Other(String),
}

impl fmt::Display for HeartBeatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HeartBeatError::Io(err) => write!(f, "IO error: {}", err),
            HeartBeatError::Serde(err) => write!(f, "Serialization error: {}", err),
            HeartBeatError::WouldBlock => write!(f, "Operation would block"),
            HeartBeatError::Poison(err) => write!(f, "Poison error: {}", err),
            HeartBeatError::Other(err) => write!(f, "Other error: {}", err),
        }
    }
}

impl std::error::Error for HeartBeatError {}

impl From<io::Error> for HeartBeatError {
    fn from(err: io::Error) -> HeartBeatError {
        if err.kind() == io::ErrorKind::WouldBlock {
            HeartBeatError::WouldBlock
        } else {
            HeartBeatError::Io(err)
        }
    }
}

impl From<SerdeError> for HeartBeatError {
    fn from(err: SerdeError) -> HeartBeatError {
        HeartBeatError::Serde(err)
    }
}

impl From<String> for HeartBeatError {
    fn from(err: String) -> HeartBeatError {
        HeartBeatError::Other(err)
    }
}

impl<T> From<PoisonError<MutexGuard<'_, T>>> for HeartBeatError {
    fn from(err: PoisonError<MutexGuard<'_, T>>) -> HeartBeatError {
        HeartBeatError::Poison(err.to_string())
    }
}

pub trait HeartBeatChannelReceiver {
    fn receive(&self) -> Result<HeartBeat, HeartBeatError>;
}

impl HeartBeatChannelReceiver for HeartBeatUdapChannel {
    fn receive(&self) -> Result<HeartBeat, HeartBeatError> {
        let mut buf = [0; 256];
        let locked_socket = self.socket.lock()?;
        let (size, _src) = locked_socket.recv_from(&mut buf)?;
        let heartbeat = serde_json::from_slice::<HeartBeat>(&buf[..size])?;
        Ok(heartbeat)
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

        // for address in &seed_node_addresses {
        //     table.insert(
        //         address.to_string(),
        //         NodeInfoRow {
        //             n_times_recevied: 0,
        //             node_info: HeartBeat {
        //                 address: address.to_string(),
        //                 ..Default::default()
        //             },
        //             send_node_counts: HashMap::new(),
        //         },
        //     );
        // }

        Watcher {
            node,
            node_info_table: node_heartbeats,
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

        // let s_socket = shared_socket.clone();
        // let s_table = node_info_table.clone();
        let _ = thread::spawn(move || {
            gossip(poll_interval_milisecs, gossip_to_n_nodes, s_table, s_socket)
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
    node_address_selector: &impl NodeAddressSelecter,
    sender: &impl HeartBeatChannelSender,
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

        let addresses = node_address_selector.select_addresses();
        match sender.send(heartbeat, addresses) {
            Ok(_) => thread::sleep(Duration::from_secs(heartbeat_interval_secs)),
            Err(_) => println!("failed to send shit"),
        };
    }
}

// let table = &mut heartbeats.lock().unwrap();
// let n_times_received = match table.get(&received_node_info.id) {
//     Some(current) => {
//         let is_new = received_node_info.generation > current.node_info.generation
//             || received_node_info.version > current.node_info.version;

//         if is_new {
//             0
//         } else {
//             current.n_times_recevied + 1
//         }
//     }
//     None => 0,
// };

// if !should_forward(n_times_received) {
//     continue;
// }

// if n_times_received > 0 {
//     continue;
// }
//
//
//
// let addresses: Vec<String> = table
//     .iter()
//     .map(|(_, v)| v.node_info.address.clone())
//     .collect();
// let selected_addresses = select_random_n_strings(addresses, gossip_to_n_nodes);

fn gossip(
    poll_interval_milisecs: u64,
    setter: &impl HeartBeatDataSetter,
    channel: &(impl HeartBeatChannelReceiver + HeartBeatChannelSender),
    forwarder: &impl ForwardStategy,
    node_address_selector: &impl NodeAddressSelecter,
) {
    loop {
        thread::sleep(Duration::from_millis(poll_interval_milisecs));
        let heartbeat = match channel.receive() {
            Ok(heartbeat) => heartbeat,
            Err(HeartBeatError::WouldBlock) => continue,
            Err(_e) => {
                eprint!("Oh no could not receive heartbeat");
                continue;
            }
        };

        setter.upsert(heartbeat.clone());
        if !forwarder.should_forward() {
            continue;
        }
        let addresses = node_address_selector.select_addresses();
        match channel.send(heartbeat.clone(), addresses) {
            Ok(_) => (),
            Err(_) => println!("failed to send shit"),
        };
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
