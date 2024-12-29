use heed::Database;
use std::error::Error;
use byteorder::LittleEndian;
use heed::EnvOpenOptions;
use heed::types::{DecodeIgnore, U128, U32};


// https://github.com/nanocurrency/nanodb-specification
fn main() -> Result<(), Box<dyn Error>> {
    let env = unsafe {
        EnvOpenOptions::new()
            .max_dbs(1000)
            .open("./")?
    };

    let mut read_tx = env.read_txn()?;
    let accounts: Database<U128<LittleEndian>, DecodeIgnore> = env.open_database(&mut read_tx, Some("accounts"))?.expect("accounts db should exist");


    let mut count = 0;
    for result in accounts.iter(&read_tx)? {
        let (accounts_key, ()) = result?;
        println!("{:#32x}", accounts_key);
        count += 1;
    }

    println!("count: {}", count);

    read_tx.commit()?;

    Ok(())
}
