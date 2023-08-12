use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Trade(String, f32, f32);

#[derive(Serialize, Deserialize, Debug)]
pub struct Data {
    pub r#type: String,
    pub timestamp: u64,
    pub prev_change_id: u64,
    pub instrument_name: String,
    pub change_id: u64,
    pub bids: Vec<Trade>,
    pub asks: Vec<Trade>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Params {
    pub channel: String,
    pub data: Data,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct Record {
    pub jsonrpc: String,
    pub method: String,
    pub params: Params,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SubConfirmation {
    pub id: u32,
    pub jsonrpc: String,
    pub result: Vec<String>,
    pub testnet: bool,
}
