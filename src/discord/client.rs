use websocket::ClientBuilder;
use websocket::Message;
use json::JsonValue;
use std::thread::JoinHandle;
use std::sync::mpsc::{sync_channel, SyncSender, Receiver};
use std::time::{SystemTime, UNIX_EPOCH, Duration, SystemTimeError};
use super::util;

#[allow(unused_imports)]
use log::{info, trace, warn};

const GATEWAY_ENDPOINT: &str = "wss://gateway.discord.gg/?v=9&encoding=json";

#[derive(Debug, Copy, Clone)]
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

#[derive(Debug, Copy, Clone)]
pub struct GatewayPayload {
    pub op: GatewayOpcode,
    pub d: Option<JsonValue>,
    pub s: Option<i32>,
    pub t: Option<&'static str>
}

#[derive(Debug)]
enum ThreadWebSocketCode {
    Shutdown,
    Message
}

#[derive(Debug)]
struct ThreadWebSocketPacket {
    op_code: ThreadWebSocketCode,
    data: Option<GatewayPayload>
}

pub struct GatewayClient {
    ws_worker: Option<JoinHandle<()>>,
    ws_worker_comm: Option<(
        SyncSender<
            ThreadWebSocketPacket
        >, Receiver<
            ThreadWebSocketPacket
        >
    )>,
    auth_token: String
}

type TimedGatewayPayload = (GatewayPayload, Duration, bool);

impl GatewayClient {
    pub const fn new(auth_token: String) -> GatewayClient {
        GatewayClient {
            ws_worker: None,
            ws_worker_comm: None,
            auth_token: auth_token
        }
    }

    pub fn ws_worker_started(mut self) -> bool {
        match self.ws_worker.ok_or(()) {
            Ok(_) => true,
            Err(_) => false
        }
    }

    pub fn get_ws_worker(self) -> Option<JoinHandle<()>> {
        self.ws_worker
    }

    pub fn start_ws_worker(mut self) -> Option<JoinHandle<()>> {
        let (tx_worker, rx_worker) = sync_channel::<ThreadWebSocketPacket>(0);
        let (tx_main, rx_main) = sync_channel::<ThreadWebSocketPacket>(0);

        self.ws_worker_comm = Option::from((tx_worker, rx_main));

        self.ws_worker = Option::from(
            util::create_thread("ws_worker", move || {
                let mut stop_flag = false;
                let mut received_gateway_packets: Vec<GatewayPayload> = vec![];
                let mut pending_gateway_packets: Vec<TimedGatewayPayload> = vec![];
                let mut current_client_packet: Option<(ThreadWebSocketPacket, i32)> = None;

                let mut client = ClientBuilder::new(GATEWAY_ENDPOINT)
                    .unwrap()
                    .connect(None)
                    .unwrap();

                client.set_nonblocking(true).unwrap();

                let (tx, rx) = (&tx_main, &rx_worker);

                loop {
                    if stop_flag {
                        match client.shutdown() {
                            Ok(_) => break,
                            Err(_) => break
                        }
                    } else {
                        for packet in received_gateway_packets.iter() {
                            let data_ref = packet.d.as_ref().unwrap();

                            match packet.op {
                                GatewayOpcode::Hello => {
                                },
                                _ => continue
                            };
                        }

                        for (packet, send_at, _) in pending_gateway_packets.iter().clone() {
                            if send_at > &util::get_time().unwrap() {
                                match client.send_message(&Message::text(util::serialize_gateway_object(packet))) {
                                    Ok(_) => trace!("sent {:?}", packet.op),
                                    Err(_) => warn!("error sending {:?}", packet.op)
                                };
                            }
                        }
                        
                        match client.recv_message() {
                            Ok(msg) => {
                                let str_data = String::from_utf8(Message::from(msg).payload.to_vec()).unwrap();
    
                                match util::deserialize_gateway_object(&str_data) {
                                    Ok(packet) => {
                                        received_gateway_packets.push(packet);
    
                                        match received_gateway_packets.last().ok_or(()) {
                                            Ok(packet) => {
                                                info!("pushed packet with op {:?}", packet.op);
                                                trace!("sequence number {:?}", packet.s);
                                                trace!("event name {:?}", packet.t);
                                            },
                                            Err(_) => continue
                                        };
                                    },
                                    Err(_) => continue
                                }
                            },
                            Err(_) => continue
                        }

                        current_client_packet = Option::from((rx.recv().unwrap(), 1));
                    }
                }
            })
        );

        info!("started ws_worker thread");

        self.ws_worker
    }

    pub fn stop_ws_worker(self) -> bool {
        match self.ws_worker_comm.ok_or(()) {
            Ok((rx, _)) => {
                match rx.send(ThreadWebSocketPacket {
                    op_code: ThreadWebSocketCode::Shutdown,
                    data: None
                }) {
                    Ok(_) => true,
                    Err(_) => false
                }
            }
            Err(_) => false
        }
    }
}