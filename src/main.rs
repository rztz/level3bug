// Copyright (c) 2024 Reinhard Zitzmann (reinhard@zitzmann.io)
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

use std::{collections::BTreeMap, fmt::Debug};

use serde::{Deserialize, Serialize};
use serde_this_or_that::as_f64;
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
    #[serde(deserialize_with = "as_f64")]
    pub limit_price: f64,
    #[serde(deserialize_with = "as_f64")]
    pub order_qty: f64,
    #[serde(with = "time::serde::rfc3339")]
    pub timestamp: time::OffsetDateTime,
}

impl Debug for Order {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!(
            "{}: {:12.8} @ {:<7.1} {:.6}",
            self.order_id,
            self.order_qty,
            self.limit_price,
            self.timestamp.unix_timestamp_nanos() as f64 / 1.0e9
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

const PRICE_PRECISION_FACTOR: f64 = 10i64.pow(1) as f64;
const QTY_PRECISION_FACTOR: f64 = 10i64.pow(8) as f64;

pub fn main() {
    // to parse buggy json run: cargo run
    // to parse api docs reference json run: cargo run -- ref

    let use_reference = std::env::args().nth(1).map_or(false, |arg| arg == "ref");

    let line_str = if use_reference {
        // read the JSON string from file "level3-doc.json" (example from kraken api website)
        std::fs::read_to_string("level3-doc.json").unwrap()
    } else {
        // read the JSON string from file "level3-bug.json"
        std::fs::read_to_string("level3-bug.json").unwrap()
    };
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
                if price_level_count > 13 {
                    break;
                }
            }

            let price_f = ask.limit_price * PRICE_PRECISION_FACTOR;
            let price_i = price_f.round() as i64;
            let price_if = price_i as f64;
            assert!((price_f - price_if).abs() < 1e-3);
            let price_s = price_i.to_string();

            let qty_f = ask.order_qty * QTY_PRECISION_FACTOR;
            let qty_i = qty_f.round() as i64;
            let qty_if = qty_i as f64;
            assert!((qty_f - qty_if).abs() < 1e-3);
            let qty_s = qty_i.to_string();

            if use_reference {
                if price_level_count < 11 {
                    crc_str.push_str(&price_s);
                    crc_str.push_str(&qty_s);
                    println!(
                        "Ask level {:2}: {:?} | price_s: {} qty_s: {:>11}",
                        price_level_count, ask, price_s, qty_s
                    );
                } else {
                    println!("(ignore Ask level {:2}: {:?})", price_level_count, ask);
                }
            } else if price_level_count < 13 && price_level_count != 10 && price_level_count != 11 {
                crc_str.push_str(&price_s);
                crc_str.push_str(&qty_s);
                println!(
                    "Ask level {:2}: {:?} | price_s: {} qty_s: {:>11}",
                    price_level_count, ask, price_s, qty_s
                );
            } else {
                println!("(ignore Ask level {:2}: {:?})", price_level_count, ask);
            }
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
                if price_level_count > 12 {
                    break;
                }
            }

            let price_f = bid.limit_price * PRICE_PRECISION_FACTOR;
            let price_i = price_f.round() as i64;
            let price_if = price_i as f64;
            assert!((price_f - price_if).abs() < 1e-3);
            let price_s = price_i.to_string();

            let qty_f = bid.order_qty * QTY_PRECISION_FACTOR;
            let qty_i = qty_f.round() as i64;
            let qty_if = qty_i as f64;
            assert!((qty_f - qty_if).abs() < 1e-3);
            let qty_s = qty_i.to_string();

            if price_level_count < 11 {
                crc_str.push_str(&price_s);
                crc_str.push_str(&qty_s);
                println!(
                    "Bid level {:2}: {:?} | price_s: {} qty_s: {:>11}",
                    price_level_count, bid, price_s, qty_s
                );
            } else {
                println!("(ignore Bid level {:2}: {:?})", price_level_count, bid);
            }
        }
    }
    println!("===============================================================================");
    println!("json level3_data.checksum: {}", level3_data.checksum);
    println!("crc_str: {}", crc_str);
    if use_reference {
        let ref_str = "44939545230839344939511126144939510000044939510000004495001033492644953064537449550250000449596356300004495963563000044960133807244960288967575449670314392283449785677896044979235630000449394889686994493944521000044939410000000449394142963234493942500000044939410292988449394338800004493941281408604493713346877449347356300004493022273429944930210000004493025550000449302700000004493021500000044928010524044919633870000449195761000044912035630000449097669000044901988982";
        println!("ref_str: {}", ref_str);
        if crc_str == ref_str {
            println!("crc_str matches reference string!");
        } else {
            println!("ERROR: crc_str does not match reference string!");
        }
    }
    let crc = crc32fast::hash(crc_str.as_bytes());
    println!("crc: {}", crc);

    if level3_data.checksum != crc {
        println!("ERROR: Checksum mismatch!");
    } else {
        println!("Checksum OK!");
    }
}
