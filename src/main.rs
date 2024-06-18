use std::fmt::Debug;

use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(PartialEq, Debug, Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct Level3Data {
    pub symbol: String,
    pub bids: Vec<Order>,
    pub asks: Vec<Order>,
    pub checksum: u32,
}

#[skip_serializing_none]
#[derive(PartialEq, Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct Order {
    pub event: Option<OrderEvent>,
    pub order_id: String,
    pub limit_price: f64,
    pub order_qty: f64,
    #[serde(with = "time::serde::rfc3339")]
    pub timestamp: time::OffsetDateTime,
}

impl Debug for Order {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!(
            "{}: {:12} @ {:<7}",
            self.order_id, self.order_qty, self.limit_price
        ))
    }
}
#[skip_serializing_none]
#[derive(PartialEq, Debug, Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
pub enum OrderEvent {
    #[serde(rename = "add")]
    Add,
    #[serde(rename = "modify")]
    Modify,
    #[serde(rename = "delete")]
    Delete,
}

const PRICE_PRECISION_FACTOR: f64 = 10.0;
const QTY_PRECISION_FACTOR: f64 = 100000000.0;

pub fn main() {
    // read the JSON string from file "level3-bug.json"
    let line_str = std::fs::read_to_string("level3-bug.json").unwrap();
    let snapshot: serde_json::Value = serde_json::from_str(&line_str).unwrap();
    let data_array = snapshot["data"].clone();
    let level3_data: Vec<Level3Data> = serde_json::from_value(data_array).unwrap();
    assert!(level3_data.len() == 1);
    let level3_data = &level3_data[0];

    let mut crc_str = String::new();

    println!("===============================================================================");
    {
        let asks_it = level3_data.asks.iter();
        let mut curr_price: f64 = 0.0;
        let mut price_level_count = 0;
        for ask in asks_it {
            if ask.limit_price != curr_price {
                curr_price = ask.limit_price;
                price_level_count += 1;
                if price_level_count > 10 {
                    break;
                }
            }

            let price_f = ask.limit_price * PRICE_PRECISION_FACTOR;
            let price_i = price_f.round() as i64;
            assert!((price_f - (price_i as f64)).abs() < 1e-9);
            let price_s = price_i.to_string();

            let qty_f = ask.order_qty * QTY_PRECISION_FACTOR;
            let qty_i = qty_f.round() as i64;
            assert!((qty_f - (qty_i as f64)).abs() < 1e-9);
            let qty_s = qty_i.to_string();

            crc_str.push_str(&price_s);
            crc_str.push_str(&qty_s);

            println!(
                "Ask level {:2}: {:?} | price_s: {} qty_s: {}",
                price_level_count, ask, price_s, qty_s
            );
        }
    }
    println!("--------------------------------------------------------------------------------");
    {
        let bids_it = level3_data.bids.iter();
        let mut curr_price: f64 = 0.0;
        let mut price_level_count = 0;
        for bid in bids_it {
            if bid.limit_price != curr_price {
                curr_price = bid.limit_price;
                price_level_count += 1;
                if price_level_count > 10 {
                    break;
                }
            }

            let price_f = bid.limit_price * PRICE_PRECISION_FACTOR;
            let price_i = price_f.round() as i64;
            assert!((price_f - (price_i as f64)).abs() < 1e-9);
            let price_s = price_i.to_string();

            let qty_f = bid.order_qty * QTY_PRECISION_FACTOR;
            let qty_i = qty_f.round() as i64;
            assert!((qty_f - (qty_i as f64)).abs() < 1e-9);
            let qty_s = qty_i.to_string();

            crc_str.push_str(&price_s);
            crc_str.push_str(&qty_s);

            println!(
                "Bid level {:2}: {:?} | price_s: {} qty_s: {}",
                price_level_count, bid, price_s, qty_s
            );
        }
    }
    println!("===============================================================================");
    println!("json level3_data.checksum: {}", level3_data.checksum);
    println!("crc_str: {}", crc_str);
    let crc = crc32fast::hash(crc_str.as_bytes());
    println!("crc: {}", crc);

    if level3_data.checksum != crc {
        println!("ERROR: Checksum mismatch!");
    }
}
