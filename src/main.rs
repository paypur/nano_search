use heed::types::DecodeIgnore;
use heed::Database;
use heed::EnvOpenOptions;
use std::error::Error;
use nanopyrs::Account;
use nano_search::Accounts;

// https://github.com/nanocurrency/nanodb-specification
fn main() -> Result<(), Box<dyn Error>> {
    let env = unsafe {
        EnvOpenOptions::new()
            .max_dbs(1000)
            .open("./")?
    };

    let mut read_tx = env.read_txn()?;
    let accounts: Database<Accounts, DecodeIgnore> = env.open_database(&mut read_tx, Some("accounts"))?.expect("accounts db should exist");

    for result in accounts.iter(&read_tx)? {
        let (accounts_key, ()) = result?;
        let account = Account::from_bytes(accounts_key).expect("failed to derive account");
        println!("{:?}", account.account);
    }

    read_tx.commit()?;

    Ok(())
}
