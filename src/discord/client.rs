use websocket::ClientBuilder;
use json::JsonValue;
use std::thread;
use std::sync::mpsc::{sync_channel, SyncSender, Receiver};

const GATEWAY_ENDPOINT: &str = "wss://gateway.discord.gg/?v=9&encoding=json";

#[derive(Debug)]
pub enum GatewayOpcode {
    Dispatch,
    Heartbeat,
    Identify,
    PresenceUpdate,
    VoiceStateUpdate,
    Resume,
    Reconnect,
    RequestGuildMembers,
    InvalidSession,
    Hello,
    HeartbeatAck,
    Unknown
}

#[derive(Debug)]
enum ThreadWebSocketCode {
    Shutdown,
    Message
}

#[derive(Debug)]
struct ThreadWebSocketPacket {
    op: ThreadWebSocketCode,
    d: JsonValue
}

#[derive(Debug)]
pub struct GatewayPayload {
    pub op: GatewayOpcode,
    pub d: Option<JsonValue>,
    pub s: Option<i32>,
    pub t: Option<String>
}

struct GatewayClient {
    ws_worker: Option<thread::JoinHandle<()>>,
    ws_worker_com: (SyncSender<ThreadWebSocketPacket>, Receiver<ThreadWebSocketPacket>),
    auth_token: String
}

impl GatewayClient {
    pub fn new(mut self, auth_token: &str) -> Self {
        self.auth_token = String::from(auth_token);
        self
    }

    pub fn ws_worker_started(mut self) -> bool {
        match self.ws_worker.ok_or(()) {
            Ok(_) => true,
            Err(_) => false
        }
    }

    pub fn start_ws_worker(mut self) {
        let (tx_worker, rx_worker) = sync_channel::<ThreadWebSocketPacket>(0);
        let (tx_main, rx_main) = sync_channel::<ThreadWebSocketPacket>(0);

        self.ws_worker_com = (tx_worker, rx_main);

        self.ws_worker = Option::from(
            thread::spawn(move || {
                let client = ClientBuilder::new(GATEWAY_ENDPOINT)
                    .unwrap()
                    .connect(None)
                    .unwrap();

                loop {
                    
                }
            })
        );
    }

    pub fn stop_ws_worker(mut self) {

    }
}

trait RestClient {
    
}