use rocket::futures::StreamExt;
use std::iter::Iterator;
mod trie;
mod nano_to;

use heed::{Database};
use heed::EnvOpenOptions;
use std::error::Error;
use std::time::Duration;
use http::Uri;
use nanopyrs::{Account};
use regex::{Regex};
use nano_search::{AccountsKey, AccountsValue, ByteString, Bytes128};
use crate::trie::{Trie, TrieRef};

use rocket::{get, routes, State};
use rocket::futures::SinkExt;
use rocket::log::private::{debug, info, trace};
use serde_json::{json, Value};
use tokio::time::{sleep};
use tokio_websockets::{ClientBuilder, Message};

#[get("/<string>")]
fn search(string: &str, trie_root: &State<TrieRef>) -> String {
    let start = chrono::offset::Local::now().timestamp_micros();

    let regex  = Regex::new(r"^(nano_)?[13][13456789abcdefghijkmnopqrstuwxyz]{0,59}$")
        .expect("regex invalid");

    let vec = match regex.captures(string) {
        // return String::from("{\n  \"error\": {\n    \"code\": 422,\n    \"message\": \"invalid request\"\n  }\n}");
        None => nano_to::search(string), 
        Some(_) => {
            let guard = trie_root.lock().unwrap();  
            guard.search(string)
                .iter_mut()
                .map(|x| {
                    json!({
                        "aliased": false,
                        "address": x,
                        "name": ""
                    }
                )})
                .collect()
        }
    };

    if vec.len() == 0 {
        debug!("Found: nothing :( in {:} micro-seconds.", chrono::offset::Local::now().timestamp_micros() - start);
    } else {
        // TODO: fix this
        debug!("Found: [{}] in {:} micro-seconds.", "", chrono::offset::Local::now().timestamp_micros() - start);
    }

    serde_json::to_string_pretty(
        &json!({
            "data": {
                "addresses": Value::Array(vec)
            }
        })
    ).unwrap()
}

#[rocket::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let root = Trie::new_arc(&[]);
    let root_ws = root.clone();

    tokio::spawn(async move {
        sleep(Duration::from_secs(86400)).await;
        nano_to::update();
    });
    
    // tokio::spawn(async move {
    //     info!("Starting ws thread");
    // 
    //     let uri = Uri::from_static("wss://nodews.hansenjc.com");
    //     // TODO: ws will probably fail sometimes
    //     let (mut client, _) = ClientBuilder::from_uri(uri)
    //         .connect()
    //         .await
    //         .expect("Failed to connect to websocket!");
    // 
    //     // https://docs.nano.org/integration-guides/websockets/#confirmations
    //     client.send(Message::text(r#"{"action":"subscribe","topic":"confirmation"}"#))
    //         .await
    //         .unwrap();
    // 
    //     while let Some(item) = client.next().await {
    //         if let Ok(msg) = item {
    //             let val: Value = serde_json::from_str(msg.as_text().unwrap()).unwrap();
    //             if val["message"]["block"]["subtype"].as_str().unwrap() != "receive" ||
    //                 val["message"]["block"]["previous"].as_str().unwrap() != "0000000000000000000000000000000000000000000000000000000000000000" {
    //                 break;
    //             }
    //             let addr = ByteString::new(
    //                 val["message"]["account"]
    //                 .as_str()
    //                 .unwrap()
    //                 .strip_prefix("nano_")
    //                 .unwrap()
    //                 .as_bytes()
    //             );
    //             info!("WS: new account opened {}", addr);
    //             root_ws.lock()
    //                 .unwrap()
    //                 .build(&addr);
    //         }
    //     }
    // });

    // build_trie_from_db(root.clone())?;

    rocket::build()
        .mount("/api", routes![search])
        .manage(root)
        .launch()
        .await?;

    Ok(())
}

// https://github.com/nanocurrency/nanodb-specification
// https://docs.nano.org/integration-guides/the-basics/
fn build_trie_from_db(root: TrieRef) -> Result<(), Box<dyn Error>> {
    info!("Building Trie");

    let start = chrono::offset::Local::now().timestamp();

    let env = unsafe {
        EnvOpenOptions::new()
            .max_dbs(100)
            .open("./")?
    };

    let mut read_tx = env.read_txn()?;
    let accounts: Database<AccountsKey, Bytes128> = env.open_database(&mut read_tx, Some("accounts"))?
        .expect("accounts db should exist");

    let mut count = 0;
    let mut total = 0;
    for result in accounts.iter(&read_tx)? {
        // public key
        let (accounts_key, accounts_value_bytes) = result?;
        match Account::from_bytes(accounts_key) {
            Ok(acc) => {
                total += 1;
                // debug!("{}", acc.account);
                let accounts_value = AccountsValue::from_bytes(&accounts_value_bytes);
                if accounts_value.balance <= 10 {
                    trace!("{} has less than 10 raw: Skipping.", acc.account);
                    continue;
                }

                root.lock()
                    .unwrap()
                    .build(
                    &acc.account
                        .strip_prefix("nano_")
                        .unwrap()
                        .as_bytes()
                );

                count += 1;
                if total % 1000000 == 0 {
                    debug!("Trie size: {}/{}", count, total);
                }
            }
            Err(_) => {}
        }
    }

    read_tx.commit()?;
    info!("Finished building trie with {:} addresses in {:} seconds.", count, chrono::offset::Local::now().timestamp() - start);

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::sync::{Arc, Mutex};
    use crate::build_trie_from_db;
    use crate::trie::Trie;

    #[test]
    fn auto_complete_test() -> Result<(), Box<dyn Error>>{
        let arc_root = Arc::new(Mutex::new(Trie::new()));
        build_trie_from_db(arc_root.clone())?;
        let root = arc_root.lock().unwrap();

        let mut e1: Vec<String> = vec!["nano_1paypur3bf9oawtbfcm4gqeekirjy37jw3ttfsidhxw99aky8auqwaf6na9q".to_string()];

        e1.sort_by(|a, b| a.cmp(&b));
        // e2.sort_by(|a, b| a.cmp(&b));
        // e3.sort_by(|a, b| a.cmp(&b));

        let mut r1 = root.search("1paypur");
        // let mut r2 = root.search("nano_31");
        // let mut r3 = root.search("nano_3bc");
        let r4 = root.search("nano_a");
        let r5 = root.search("nano_");
        let r6 = root.search("nano_2x");

        r1.sort_by(|a, b| a.cmp(&b));
        // r2.sort_by(|a, b| a.cmp(&b));
        // r3.sort_by(|a, b| a.cmp(&b));

        assert_eq!(r1.as_slice(), e1);
        // assert_eq!(r2.as_slice(), e2);
        // assert_eq!(r3.as_slice(), e3);
        assert_eq!(r4.as_slice(), Vec::<String>::new());
        assert_eq!(r5.as_slice(), Vec::<String>::new());
        assert_eq!(r6.as_slice(), Vec::<String>::new());

        Ok(())
    }
}