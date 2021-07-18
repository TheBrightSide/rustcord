pub mod discord;

use discord::client::GatewayClient;
use log::{info};
use std::thread;
use std::sync::mpsc;
use std::time::Duration;

fn main() {
    env_logger::init();
    info!("starting threads...");
    
    let client = /* std::boxed::Box::new( */ GatewayClient::new(String::from("AAAAAAAAAAA")) /* ) */;
    
    client.start_ws_worker().unwrap().join().unwrap();
}