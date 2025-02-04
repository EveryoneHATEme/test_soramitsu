use std::net::{SocketAddr, UdpSocket};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use rand::Rng;


pub struct Node {
    period: u64,
    known_nodes: Arc<Mutex<Vec<SocketAddr>>>,
    socket: Arc<UdpSocket>,
    is_alive: Arc<AtomicBool>,
}


impl Node {
    pub fn new(port: u16, period: u64) -> Self {
        let socket: Arc<UdpSocket> = Arc::new(UdpSocket::bind(SocketAddr::from(([127, 0, 0, 1], port))).expect("failed to bind host socket"));
        let known_nodes = Arc::new(Mutex::new(Vec::new()));
        let is_alive = Arc::new(AtomicBool::new(true));

        Self {
            period, 
            known_nodes,
            socket,
            is_alive: is_alive,
        }
    }
    
    pub fn start(&self, connect_to: Option<String>) {
        let _listener_thread = self.run_listener();
        
        if let Some(address) = connect_to {
            let address: SocketAddr = address.parse().unwrap();
            self.known_nodes.lock().unwrap().push(address);
            self.socket.send_to(b"list", address).unwrap();
        }

        let sender_thread = self.run_sender();

        let socket = Arc::clone(&self.socket);
        let is_alive = Arc::clone(&self.is_alive);
        let known_nodes = Arc::clone(&self.known_nodes);

        ctrlc::set_handler(move || {
            println!("Shutting down...");
            is_alive.store(false, Ordering::SeqCst);
            let known_nodes = known_nodes.lock().unwrap();
            for node_address in known_nodes.iter() {
                socket.send_to(b"stop", node_address).unwrap();
                println!("Sent stop to {}", node_address);
            }
        }).unwrap();

        sender_thread.join().unwrap();
    }

    fn run_listener(&self) -> JoinHandle<()>{
        let socket = Arc::clone(&self.socket);
        let known_nodes = Arc::clone(&self.known_nodes);
        let is_alive = Arc::clone(&self.is_alive);
        
        thread::spawn(move || {
            let mut buffer = [0u8; 1024];
            while is_alive.load(Ordering::SeqCst) {
                match socket.recv_from(&mut buffer) {
                    Ok((size, address)) => {
                        let obtained_msg = String::from_utf8_lossy(&buffer[..size]).to_string();
                        let known_nodes = Arc::clone(&known_nodes);
                        let socket = Arc::clone(&socket);
                        thread::spawn(move || {
                            execute_command(&obtained_msg, address, &known_nodes, &socket);
                        }).join().unwrap();
                    }, 
                    Err(error) => {
                        if is_alive.load(Ordering::SeqCst) {
                            eprintln!("listener failed: {}", error);
                        }
                    }
                }
            }
        })
    }

    fn run_sender(&self) -> JoinHandle<()> {
        let sender_socket = Arc::clone(&self.socket);
        let known_nodes = Arc::clone(&self.known_nodes);
        let period = self.period;
        let is_alive = Arc::clone(&self.is_alive);

        let sender_thread = thread::spawn(move || {
            while is_alive.load(Ordering::SeqCst) {
                thread::sleep(Duration::from_secs(period));

                let mut rng = rand::thread_rng();
                
                let known_nodes = known_nodes.lock().unwrap();
                let message = format!("msg {}", rng.gen::<i32>());

                for node_address in known_nodes.iter() {
                    sender_socket.send_to(message.as_bytes(), node_address).unwrap();
                }
            }
        });

        sender_thread
    }
}


fn execute_command(command: &str, sender: SocketAddr, known_nodes: &Arc<Mutex<Vec<SocketAddr>>>, socket: &Arc<UdpSocket>) {
    if let Some((command_name, command_payload)) = command.split_once(' ') {
        match command_name {
            "msg" => handle_message_command(command_payload, sender),
            "list_response" => handle_list_response(command_payload, known_nodes, socket),
            _ => println!("Unknown command {} from {}", command_name, sender),
        }
    } else {
        match command {
            "new" => handle_new_command(sender, known_nodes),
            "list" => handle_list_command(sender, known_nodes, socket),
            "stop" => handle_stop_command(sender, known_nodes), 
            _ => println!("Unknown command {} from {}", command, sender),
        }
    }
}


fn handle_new_command(sender: SocketAddr, known_nodes: &Arc<Mutex<Vec<SocketAddr>>>) {
    let mut known_nodes = known_nodes.lock().unwrap();
    known_nodes.push(sender);
}


fn handle_message_command(message: &str, sender: SocketAddr) {
    println!("Received message {} from {}", message, sender);
}


fn handle_list_command(sender: SocketAddr, known_nodes: &Arc<Mutex<Vec<SocketAddr>>>, socket: &Arc<UdpSocket>) {
    let mut known_nodes = known_nodes.lock().unwrap();

    for node_address in known_nodes.iter() {
        let node_address = node_address.to_string();
        let message = format!("list_response {}", node_address);
        socket.send_to(message.as_bytes(), sender).unwrap();
    }
    
    known_nodes.push(sender);
}


fn handle_list_response(other_address: &str, known_nodes: &Arc<Mutex<Vec<SocketAddr>>>, socket: &Arc<UdpSocket>) {
    let other_address: SocketAddr = other_address.parse().unwrap();

    let mut known_nodes = known_nodes.lock().unwrap();
    known_nodes.push(other_address);

    socket.send_to(b"new", other_address).unwrap();
}


fn handle_stop_command(sender: SocketAddr, known_nodes: &Arc<Mutex<Vec<SocketAddr>>>) {
    let mut known_nodes = known_nodes.lock().unwrap();

    known_nodes.retain(|&node_address| node_address != sender);

    println!("{} stopped", sender);
}