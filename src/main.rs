mod trie;

use std::cell::RefCell;
use heed::Database;
use heed::EnvOpenOptions;
use std::error::Error;
use std::rc::Rc;
use heed::types::DecodeIgnore;
use nanopyrs::Account;
use sorted_vec::SortedSet;
use nano_search::{Accounts};
use crate::trie::{Trie, TrieRef};

// https://github.com/nanocurrency/nanodb-specification
fn main() -> Result<(), Box<dyn Error>> {
    let start = chrono::offset::Local::now().timestamp();

    let env = unsafe {
        EnvOpenOptions::new()
            .max_dbs(100)
            .open("./")?
    };

    let mut read_tx = env.read_txn()?;
    let accounts: Database<Accounts, DecodeIgnore> = env.open_database(&mut read_tx, Some("accounts"))?.expect("accounts db should exist");

    let mut root = Trie::new();

    let mut count = 0;
    for result in accounts.iter(&read_tx)? {
        // public key
        let (accounts_key, ()) = result?;
        let account = Account::from_bytes(accounts_key).expect("failed to derive account");
        // println!("account: {:?}", account.account);

        // https://docs.nano.org/integration-guides/the-basics/
        // TODO: remove check sum, should be able to calculate
        root.build(
            &account.account
            .strip_prefix("nano_")
            .unwrap()
            [0..52]
        );

        count += 1;
        if count % 100000 == 0 {
            println!("{}", count);
        }
    }

    read_tx.commit()?;

    println!("Finished building trie with {:} addresses in {:} seconds", count, chrono::offset::Local::now().timestamp() - start);

    println!("Found {:?}", root.search("1111".to_string()));

    Ok(())
}
