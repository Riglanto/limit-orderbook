mod orderbook;

use orderbook::Data;
use serde_json::json;
use std::{collections::HashMap, net::TcpStream, process::exit};
use tungstenite::{connect, stream::MaybeTlsStream, Message, WebSocket};
use url::Url;

fn main() {
    println!("Limit Order Book");

    // Three ways of structures
    // 1. Hashmap, easy access, but doesn't support floats
    // 2. Multiple vectors, difficult access
    // 3. Sorted queue
    let mut bids = HashMap::<u32, f32>::new();
    let mut asks = HashMap::<u32, f32>::new();
    let mut best_bid: u32 = 0;
    let mut best_ask: u32 = 0;

    let subscription = String::from("book.BTC-PERPETUAL.100ms");

    let (mut socket, _response) =
        connect(Url::parse("wss://test.deribit.com/ws/api/v2").unwrap()).expect("Can't connect");

    init_sub(&mut socket, &subscription);

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

        if counter > 0 {
            let record: orderbook::Record =
                serde_json::from_str(&msg).expect("Can't parse to JSON");

            let data = record.params.data;

            match data.prev_change_id {
                Some(prev_change_id) => {
                    if change_id > 0 && prev_change_id != change_id {
                        // Reconnect
                        // change_id = 0;
                        println!("Reconnecting....");
                    }
                }
                None => {}
            }

            change_id = data.change_id;

            match data.r#type.as_str() {
                "snapshot" => {
                    (best_bid, best_ask) = load_snapshot(&data, &mut bids, &mut asks);
                }
                "change" => {
                    // Process update

                    for trade in data.asks.iter() {
                        match trade.0.as_str() {
                            "new" => {
                                let price = (trade.1 * 10.0) as u32;
                                asks.insert(price, trade.2);
                                if price < best_ask {
                                    best_ask = price;
                                }
                            }
                            "change" => {}
                            "delete" => {}

                            unknown => {
                                println!("Not supported type {:?}", unknown);
                            }
                        }
                    }

                    for trade in data.bids.iter() {
                        match trade.0.as_str() {
                            "new" => {
                                let price = (trade.1 * 10.0) as u32;
                                bids.insert(price, trade.2);
                                if price > best_bid {
                                    best_bid = price;
                                }
                            }
                            "change" => {}
                            "delete" => {}

                            unknown => {
                                println!("Not supported type {:?}", unknown);
                            }
                        }
                    }

                    println!("{:?} {:?}", counter, data.r#type);
                }
                other => {
                    println!("Not supported {:?}", other);
                }
            }

            println!("{0:<10} {1:<10}", "Bid", "Offer");
            println!("-------------------");
            println!("{0:<10.1} {1:<10.1}", "-", f64::from(best_ask) / 10.0);
            println!("{0:<10.1} {1:<10.1}", f64::from(best_bid) / 10.0, "-");
            println!("{0:<10} {1:<10}", bids[&best_bid], asks[&best_ask]);

            // For simplification writing best bid/ask on every 10th update
            // instead of spawning different process to do it every second
            if counter % 10 == 1 {
                // print
            }
        }

        if counter == 0 {
            // Process confirmation

            let confirmation: orderbook::SubConfirmation =
                serde_json::from_str(&msg).expect("Can't parse to JSON");

            if !confirmation.result.contains(&subscription) {
                println!("Sub failed");
                exit(0)
            }

            continue;
        }
    }
}

fn load_snapshot(
    data: &Data,
    bids: &mut HashMap<u32, f32>,
    asks: &mut HashMap<u32, f32>,
) -> (u32, u32) {
    println!("Load snapshot");

    bids.clear();
    bids.extend(data.bids.iter().map(|x| ((x.1 * 10.0) as u32, x.2)));

    asks.clear();
    asks.extend(data.asks.iter().map(|x| ((x.1 * 10.0) as u32, x.2)));

    return (*bids.keys().max().unwrap(), *asks.keys().min().unwrap());
}

fn init_sub(socket: &mut WebSocket<MaybeTlsStream<TcpStream>>, subscription: &str) {
    let sub = json!({
        "jsonrpc" : "2.0",
        "id" : 0,
        "method" : "public/subscribe",
        "params" : {
            "channels": [subscription]
        }
    });

    socket.send(Message::Text(sub.to_string())).unwrap();
}

// Alternative websocket setup that didn't work for me
// https://github.com/snapview/tungstenite-rs/
//
//
// fn main() {
//     let server = TcpListener::bind("wss://test.deribit.com/ws/api/v2:443").unwrap();
// }
