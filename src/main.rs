mod orderbook;

use serde_json::json;
use std::process::exit;
use tungstenite::{connect, Message};
use url::Url;

fn main() {
    println!("Limit Order Book");

    let subscription = String::from("book.BTC-PERPETUAL.100ms");

    let (mut socket, _response) =
        connect(Url::parse("wss://test.deribit.com/ws/api/v2").unwrap()).expect("Can't connect");

    let sub = json!({
        "jsonrpc" : "2.0",
        "id" : 0,
        "method" : "public/subscribe",
        "params" : {
            "channels": [subscription]
        }
    });

    socket.send(Message::Text(sub.to_string())).unwrap();

    let mut counter = -1;
    let mut change_id = 0;

    loop {
        let msg = socket.read().expect("Error reading message");
        let msg = match msg {
            tungstenite::Message::Text(s) => s,
            _ => {
                panic!()
            }
        };

        counter += 1;

        if counter == 0 {
            // Process confirmation

            let confirmation: orderbook::SubConfirmation =
                serde_json::from_str(&msg).expect("Can't parse to JSON");

            if !confirmation.result.contains(&subscription) {
                println!("Sub failed");
                exit(0)
            }

            continue;
        } else if counter == 1 {
            // Process snapshoy

            continue;
        }

        let record: orderbook::Record = serde_json::from_str(&msg).expect("Can't parse to JSON");

        // Validation block

        if change_id > 0 && record.params.data.prev_change_id != change_id {
            // Reconnect
            // change_id = 0;
            println!("Reconnecting....");
        }

        change_id = record.params.data.change_id;

        println!("{:?} {:?}", counter, record.params.data.r#type);

        // For simplification writing best bid/ask on update
        // instead of spawning different process to do it every second

        // exit(0);
        // println!("{:?}", parsed["params"]);
        // println!("{:?}", parsed["params"]);
    }
}

// Alternative websocket setup that didn't work for me
// https://github.com/snapview/tungstenite-rs/
//
//
// fn main() {
//     let server = TcpListener::bind("wss://test.deribit.com/ws/api/v2:443").unwrap();
// }
