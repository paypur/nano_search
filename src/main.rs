mod trie;

use heed::Database;
use heed::EnvOpenOptions;
use std::error::Error;
use heed::types::DecodeIgnore;
use nanopyrs::{Account};
use nano_search::{Accounts};
use crate::trie::{Trie};

// https://github.com/nanocurrency/nanodb-specification
fn main() -> Result<(), Box<dyn Error>> {
    let start = chrono::offset::Local::now().timestamp();

    let mut root = Trie::new();

    let env = unsafe {
        EnvOpenOptions::new()
            .max_dbs(100)
            .open("./")?
    };

    let mut read_tx = env.read_txn()?;
    let accounts: Database<Accounts, DecodeIgnore> = env.open_database(&mut read_tx, Some("accounts"))?.expect("accounts db should exist");

    let mut count = 0;
    for result in accounts.iter(&read_tx)? {
        // public key
        let (accounts_key, ()) = result?;

        match Account::from_bytes(accounts_key) {
            Ok(acc) => {
                // println!("{}", acc.account);

                root.build(
                    &acc.account
                        .strip_prefix("nano_")
                        .unwrap()
                        .as_bytes()
                        [0..52] // drop 8 char checksum
                );

                count += 1;
                if count % 100000 == 0 {
                    println!("{}", count);
                }
            }
            Err(_) => {}
        }

        // https://docs.nano.org/integration-guides/the-basics/
    }

    read_tx.commit()?;

    println!("Finished building trie with {:} addresses in {:} seconds", count, chrono::offset::Local::now().timestamp() - start);

    // TODO: recalculate check sum,

    println!("Found {:?}", root.search("1111".to_string()));
    println!("Found {:?}", root.search("31".to_string()));
    println!("Found {:?}", root.search("3bc".to_string()));
    println!("Found {:?}", root.search("a".to_string()));
    println!("Found {:?}", root.search("".to_string()));
    println!("Found {:?}", root.search("2x".to_string()));

    Ok(())
}
