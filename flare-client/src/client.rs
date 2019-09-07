
use super::sampler_client::*;
use std::{io, thread};
use std::sync::{Arc, Mutex};
use websocket::sync::Server;
use websocket::OwnedMessage;
use websocket::sync::sender::Sender;
use websocket::sender::Writer;
use websocket::server::upgrade::WsUpgrade;
use websocket::server::upgrade::sync::Buffer;

type JsonValue = serde_json::Value;

pub struct Profiler {
    self_ref: Option<Arc<Mutex<Profiler>>>,
    sample_client: Option<Arc<Mutex<SamplerClient>>>,
    agent_addr: String,
    analysis_bind_addr: String,
    running: bool,
}

impl Profiler {
    pub fn new() -> Arc<Mutex<Profiler>> {
        let mut inst = Arc::new(Mutex::new(Profiler {
            self_ref: None,
            sample_client: None,
            agent_addr: "127.0.0.1:3333".to_string(),
            analysis_bind_addr: "0.0.0.0:3344".to_string(),
            running: true
        }));
        inst.lock().unwrap().self_ref = Some(inst.clone());
        inst
    }

    pub fn connect_agent(&mut self, agent_addr: &str) -> io::Result<()> {
        self.agent_addr = agent_addr.to_string();
        let mut client = SamplerClient::new(agent_addr)?;
        self.sample_client = Some(client.clone());

        thread::spawn(move|| {
            client.lock().unwrap().subscribe_events();
        });

        Ok(())
    }

    pub fn get_dashboard(&mut self) {
        self.sample_client.as_ref().unwrap().lock().unwrap().get_dashboard();
    }

    fn start_ws_server(&mut self) {
        let self_ref = self.self_ref.as_ref().unwrap().clone();
        let bind_addr = self.analysis_bind_addr.clone();
        thread::spawn(move || {
            println!("analysis server binding: {}", bind_addr);
            let server = Server::bind(bind_addr).unwrap();
            for request in server.filter_map(Result::ok) {
                if !self_ref.lock().unwrap().is_running() {
                    println!("Shutting down analysis ws server ...");
                    return;
                }
                Profiler::handle_connection(self_ref.clone(), request);
            }
        });
    }

    fn handle_connection(self_ref: Arc<Mutex<Profiler>>, request: WsUpgrade<std::net::TcpStream, core::option::Option<Buffer>>) {
        // Spawn a new thread for each connection.
        thread::spawn(move || {
            let ws_protocol = "flare-ws";
//            if !request.protocols().contains(&ws_protocol.to_string()) {
//                request.reject().unwrap();
//                return;
//            }

//            let mut client = request.use_protocol(ws_protocol).accept().unwrap();
            let mut client = request.accept().unwrap();

            let ip = client.peer_addr().unwrap();

            println!("Connection from {}", ip);

            let message = OwnedMessage::Text("Hello".to_string());
            client.send_message(&message).unwrap();

            let (mut receiver, mut sender) = client.split().unwrap();

            for message in receiver.incoming_messages() {
                let message = message.unwrap();

                match message {
                    OwnedMessage::Close(_) => {
                        let message = OwnedMessage::Close(None);
                        sender.send_message(&message).unwrap();
                        println!("Client {} disconnected", ip);
                        return;
                    }
                    OwnedMessage::Ping(ping) => {
                        let message = OwnedMessage::Pong(ping);
                        sender.send_message(&message).unwrap();
                    }
                    OwnedMessage::Text(json) => {
                        Profiler::handle_request(self_ref.clone(), &mut sender, json);
                    }
                    _ => sender.send_message(&message).unwrap(),
                }
            }
        });
    }

    fn handle_request(self_ref: Arc<Mutex<Profiler>>, sender: &mut Writer<std::net::TcpStream>, json_str: String) -> io::Result<()> {
        println!("recv: {}", json_str);
//        let message = OwnedMessage::Text(json_str);
//        sender.send_message(&message);

        //TODO parse request to json
        let request: JsonValue = serde_json::from_str(&json_str)?;
        if let JsonValue::String(cmd) = &request["cmd"] {
            match cmd {
                "dashboard" => {
                    let result = self_ref.lock().unwrap().get_dashboard();
                    sender.send_message(OwnedMessage::Text(result));
                }
            }
        }else {
            println!("unknown request: {}", json_str);
        }

        Ok(())
    }

    pub fn startup(&mut self) {
        self.start_ws_server();
    }

    pub fn shutdown(&mut self) {
        self.running = false;
    }

    pub fn is_running(&self) -> bool {
        self.running
    }
}