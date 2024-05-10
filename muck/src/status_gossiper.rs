use std::fmt;
use std::str::Bytes;

use serde::{Deserialize, Serialize};
use serde_json;

use std::net::UdpSocket;

use std::collections::HashMap;

use std::thread;

pub trait Gossiper {
    fn gossip(&self) -> Result<thread::JoinHandle<()>, ServiceInfoGossiperError>;
}

// error
#[derive(Debug)]
pub enum ServiceInfoGossiperError {
    InvalidCredentials,
}

impl fmt::Display for ServiceInfoGossiperError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            ServiceInfoGossiperError::InvalidCredentials => {
                write!(f, "Invalid credentials provided")
            }
        }
    }
}

impl std::error::Error for ServiceInfoGossiperError {}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Node {
    id: String,
    ip: String,
    port: u16,
    status: String,
    updated_at: u64,
    version: u64,
}

struct NodeTable {
    nodes: HashMap<String, Node>,
}

pub struct ServiceInfoGossiper {
    address: String,
    seed_nodes: Vec<Node>,
    gossip_interval: u8,
    gossip_peer_percent: u8,
}

impl ServiceInfoGossiper {
    fn run(&self) -> Result<thread::JoinHandle<()>, ServiceInfoGossiperError> {
        let mut table: NodeTable = NodeTable {
            nodes: HashMap::new(),
        };

        for node in self.seed_nodes {
            table.nodes.insert(node.id, node)
        }

        let socket = UdpSocket::bind(self.address).expect("Failed to bind socket");
        socket
            .set_nonblocking(true)
            .expect("Failed to set non-blocking");

        let handle = thread::spawn(move || loop {
            if let Some(recieved_node) = receive_message(&socket) {
                println!("Received: {:?}", recieved_node);

                let node = table.nodes.get(&recieved_node.id);
                if let Some(node) = node {
                    if recieved_node > node {
                        table.nodes.insert(recieved_node.id, recieved_node)
                    }
                } else {
                    table.nodes.insert(recieved_node.id, recieved_node)
                }
            }
        });
        Ok(handle)
    }

    fn periodic_status_report(&self) -> Result<thread::JoinHandle<()>, ServiceInfoGossiperError> {}
}

fn receive_message(socket: &UdpSocket) -> Option<Node> {
    let mut buf = [0; 1024];
    match socket.recv_from(&mut buf) {
        Ok((size, _src)) => {
            let msg = serde_json::from_slice(&buf[..size]).unwrap();
            Some(msg)
        }
        Err(e) => {
            if e.kind() != std::io::ErrorKind::WouldBlock {
                println!("Error receiving: {}", e);
            }
            None
        }
    }
}

fn gossip(targets: Vec<String>, socket: &UdpSocket, msg: Bytes) {
    for target in targets {
        socket.send_to(msg, target).expect("Failed to send message");
    }
}

// impl Registration for DummyRegistration {
//     fn register(&self) -> Result<(), RegistrationError> {
//         info!("Muck has been registered");
//         return Ok(());
//     }

//     fn deregister(&self) -> Result<(), RegistrationError> {
//         return Ok(());
//     }
// }
