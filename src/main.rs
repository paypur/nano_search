mod trie;

use heed::{Database};
use heed::EnvOpenOptions;
use std::error::Error;
use heed::types::{DecodeIgnore};
use nanopyrs::{Account};
use nano_search::{Accounts};
use crate::trie::{Trie};

// https://github.com/nanocurrency/nanodb-specification
// https://docs.nano.org/integration-guides/the-basics/
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
        match Account::from_bytes(accounts_key) {
            Ok(acc) => {
                // println!("{}", acc.account);
                root.build(
                    &acc.account
                        .strip_prefix("nano_")
                        .unwrap()
                        .as_bytes()
                        [0..52] // drop 8 char checksum, only saves like 20 MB
                );
                count += 1;
                if count % 100000 == 0 {
                    println!("{}", count);
                }
            }
            Err(_) => {}
        }
    }
    
    read_tx.commit()?;
    
    println!("Finished building trie with {:} addresses in {:} seconds", count, chrono::offset::Local::now().timestamp() - start);
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::error::Error;
    use heed::{Database, EnvOpenOptions};
    use heed::types::DecodeIgnore;
    use nanopyrs::Account;
    use nano_search::Accounts;
    use crate::trie::Trie;

    #[test]
    fn auto_complete_test() -> Result<(), Box<dyn Error>>{
        let mut root = Trie::new();
        
        let env = unsafe {
            EnvOpenOptions::new()
                .max_dbs(100)
                .open("./")?
        };
        
        let mut read_tx = env.read_txn()?;
        let accounts: Database<Accounts, DecodeIgnore> = env.open_database(&mut read_tx, Some("accounts"))?.expect("accounts db should exist");
        for result in accounts.iter(&read_tx)? {
            // public key
            let (accounts_key, ()) = result?;
            match Account::from_bytes(accounts_key) {
                Ok(acc) => {
                    root.build(
                        &acc.account
                            .strip_prefix("nano_")
                            .expect("Address should prefix with 'nano_'!")
                            .as_bytes()
                            [0..52] // drop 8 char checksum
                    );
                }
                Err(_) => {}
            }
        }

        read_tx.commit()?;
        
        let mut e1: Vec<String> = vec!["11114w1fcd1suigthy87ymi5rqo3sky7fqkbjpdih5han5tp83tb".to_string(), "11116yqsgoxg67cpbabzdqq3tc3s3cmnxpt7cb3b77sb6ddj3ztn".to_string(), "1111gasqh5tfpi7ndj4qr847jnzcrhbcnq9rt71e7yfx4ucsbhih".to_string(), "1111j7m4pgxikpd9ngzyae3hnz19x8h7ouconc81dwzqrpackhys".to_string(), "1111nm1crbdkh1xx1nmkf976s6exnaery8ripbytpnxorz4mgss6".to_string()];
        let mut e2: Vec<String> = vec!["31114nprjd6y8kt8xihr5irkudufxwbarybmbj73hpjnyj3ok5rc".to_string(), "31114csst1h94diax547k11so3ojiwn6g8osp48wh5mhzgirzi1r".to_string(), "31116mh6pajix5zc1rpg3xy1t5jdb1wz3u4joqu1nq1fk3kup9pu".to_string(), "31116hrcowsegst8jxkidqjqi1kza5bisbh1yni575cyi5eowein".to_string(), "31118fqe53bteho64ome7k1sznb1qpwuokqumb56sdfmch8zxfc7".to_string()];
        let mut e3: Vec<String> = vec!["3bc11asob97osigir7na3k117znhgzh5bcyqm5srpgtgqibdqfm4".to_string(), "3bc13wjf9awysqgkdgzf5diduy89srbxkdd6trcb3hd3scw1edea".to_string(), "3bc138ebwudk89hu3k5dxghs59y6u5mjpwurjrfmyasj46r78bfs".to_string(), "3bc14emf3bwgfyoba4bsmk9hij11em9yaff9rbjni97fz3txqu7u".to_string(), "3bc144nkhfdiiis5oxuy8jyg54gywryxuc1sf7p1w5jioechntsa".to_string()];
    
        e1.sort_by(|a, b| a.cmp(&b));
        e2.sort_by(|a, b| a.cmp(&b));
        e3.sort_by(|a, b| a.cmp(&b));
    
        let mut r1 = root.search(b"1111");
        let mut r2 = root.search(b"31");
        let mut r3 = root.search(b"3bc");
        let r4 = root.search(b"a");
        let r5 = root.search(b"");
        let r6 = root.search(b"2x");

        r1.sort_by(|a, b| a.cmp(&b));
        r2.sort_by(|a, b| a.cmp(&b));
        r3.sort_by(|a, b| a.cmp(&b));

        assert_eq!(r1.as_slice(), e1);
        assert_eq!(r2.as_slice(), e2);
        assert_eq!(r3.as_slice(), e3);
        assert_eq!(r4.as_slice(), Vec::<String>::new());
        assert_eq!(r5.as_slice(), Vec::<String>::new());
        assert_eq!(r6.as_slice(), Vec::<String>::new());

        Ok(())
    }
}