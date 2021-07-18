use std::time::{SystemTime, UNIX_EPOCH, Duration, SystemTimeError};
use std::thread;
use std::io;
use json::object;
use super::client::{GatewayOpcode, GatewayPayload};

pub fn get_time() -> Result<Duration, SystemTimeError> {
    SystemTime::now().
        duration_since(UNIX_EPOCH)
}

// clone of thread::spawn but with a name parameter B)
pub fn create_thread<F, T>(name: &str, func: F) -> thread::JoinHandle<T>
where
    F: std::ops::FnOnce() -> T,
    F: std::marker::Send + 'static,
    T: std::marker::Send + 'static,
{
    thread::Builder::new()
        .name(name.into())
        .spawn(func)
        .unwrap()
}

// convert &str to GatewayPayload
pub fn deserialize_gateway_object(str_input: &str) -> Result<GatewayPayload, json::Error> {
    let mut parsed = match json::parse(str_input) {
        Ok(res) => res,
        Err(e) => return Err(e)
    };
    
    let mut result = GatewayPayload { 
        op: match parsed["op"].as_i32().unwrap() {
            0 => GatewayOpcode::Dispatch,
            1 => GatewayOpcode::Heartbeat,
            7 => GatewayOpcode::Reconnect,
            9 => GatewayOpcode::InvalidSession,
            10 => GatewayOpcode::Hello,
            11 => GatewayOpcode::HeartbeatAck,
            _ => GatewayOpcode::Unknown
        },
        d: None,
        s: None,
        t: None
    };

    if !parsed["d"].is_null() {
        result.d = Option::from(parsed["d"].take());
    }

    if !parsed["s"].is_null() {
        result.s = Option::from(parsed["s"].as_i32().unwrap());
    }
    
    if !parsed["t"].is_null() {
        result.t = Option::from(parsed["t"].as_str().unwrap());
    }

    Ok(result)
}

// convert GatewayPayload to String
pub fn serialize_gateway_object(structured_object: &GatewayPayload) -> String {
    let mut data = object! {};
    
    let num_op: i32 = match structured_object.op {
        GatewayOpcode::Dispatch => 0,
        GatewayOpcode::Heartbeat => 1,
        GatewayOpcode::Identify => 2,
        GatewayOpcode::PresenceUpdate => 3,
        GatewayOpcode::VoiceStateUpdate => 4,
        GatewayOpcode::Resume => 6,
        GatewayOpcode::Reconnect => 7,
        GatewayOpcode::RequestGuildMembers => 8,
        GatewayOpcode::InvalidSession => 9,
        GatewayOpcode::Hello => 10,
        GatewayOpcode::HeartbeatAck => 11,
        GatewayOpcode::Unknown => -1
    };

    data["op"] = num_op.into();
    data["d"] = structured_object.d.as_ref().unwrap_or(&json::Null).clone();

    match structured_object.s.as_ref().ok_or(()) {
        Ok(s) => data["s"] = s.clone().into(),
        Err(_) => data["s"] = json::Null
    };

    match structured_object.t.as_ref().ok_or(()) {
        Ok(t) => data["t"] = t.clone().into(),
        Err(_) => data["t"] = json::Null
    };

    json::stringify(data)
}