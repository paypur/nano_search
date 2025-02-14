mod trie;

use std::sync::{Arc, Mutex};
use heed::{Database};
use heed::EnvOpenOptions;
use std::error::Error;
use heed::types::{DecodeIgnore};
use nanopyrs::{Account};
use regex::Regex;
use nano_search::{Accounts, ByteString};
use crate::trie::{Trie, TrieRef};

use rocket::{get, routes, State};
use rocket::tokio::join;

#[get("/<string>")]
fn search(string: &str, trie_root: &State<TrieRef>) -> String {
    let start = chrono::offset::Local::now().timestamp_micros();

    let regex  = Regex::new(r"^(nano_)?[13][13456789abcdefghijkmnopqrstuwxyz]{0,59}$")
        .expect("regex invalid");

    if let None = regex.captures(string) {
        return String::from("{\n  \"error\": {\n    \"code\": 422,\n    \"message\": \"invalid request\"\n  }\n}");
    }

    let guard = trie_root.lock().unwrap();
    let vec = guard.search(string);

    if vec.len() == 0 {
        println!("Found: nothing :( in {:} micro-seconds.", chrono::offset::Local::now().timestamp_micros() - start);
        return String::from("{\n  \"data\": {\n    \"addresses\": []\n  }\n}");
    }

    println!("Found: [{}] in {:} micro-seconds.", vec.join(", "), chrono::offset::Local::now().timestamp_micros() - start);
    let json_vec = vec.iter().map(|s| format!("      \"{}\"", s)).collect::<Vec<String>>().join(",\n");
    format!("{{\n  \"data\": {{\n    \"addresses\": [\n{}\n    ]\n  }}\n}}", json_vec)
}

#[rocket::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let root = build_trie()?;

    rocket::build()
        .mount("/api", routes![search])
        .manage(root)
        .launch()
        .await?;

    Ok(())
}

// https://github.com/nanocurrency/nanodb-specification
// https://docs.nano.org/integration-guides/the-basics/
fn build_trie() -> Result<TrieRef, Box<dyn Error>> {
    println!("Building Trie");

    let start = chrono::offset::Local::now().timestamp();

    let env = unsafe {
        EnvOpenOptions::new()
            .max_dbs(100)
            .open("./")?
    };

    let mut read_tx = env.read_txn()?;
    let accounts: Database<Accounts, DecodeIgnore> = env.open_database(&mut read_tx, Some("accounts"))?
        .expect("accounts db should exist");

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
                );
                count += 1;
                if count % 1000000 == 0 {
                    println!("{}", count);
                }
            }
            Err(_) => {}
        }
    }

    read_tx.commit()?;
    println!("Finished building trie with {:} addresses in {:} seconds.", count, chrono::offset::Local::now().timestamp() - start);

    Ok(Arc::new(Mutex::new(root)))
}

// Main net test
// Finished building trie with 36886987 addresses in 135 seconds
// Looking for addresses with prefix "1pay"
// 1pay131j4o7ybno9bh9ymx6dfuf5cp38fjkbcudcyhqcmbugpb3u
// 1pay175ycu9cbupwdy3nd6f1kobm4d1a5ug9hg3thfsqrx8q4g9k
// 1pay19xc16cuyo3w86c4rxjn7gsddoxw1qpahpy8arfuj5qaqpr1
// 1pay1b68cd8io6qt7jiy9cu9n7fjh4fjacxi3ydfec755tr46mni
// 1pay1bb5sjkxuycuzhis47g3z1d9nqpkqud8tecrzrp78xnqceet
// 1pay1bs7iuiirnabzz7hsj8mzgkso9857uq4o6rwehwx9czirj9m
// 1pay1ger4tkqwmyeqndq7zz149dmxcy1oq4tk9hzrim7x1w16ciq
// 1pay1gen41unujxb4ced9fkagwczy7b565tfkbueyys7hk9ihbbp
// 1pay1gm8tishb41cj4ge95miecmhczo9r18qyi5uuhiz81ccrhgh
// 1pay1gwjcdqh76diiewiypcdsydytyq17si13kfy3biyfqhoopmu
// Finished searching in 11 micro-seconds

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
                            .expect("Address should prefixed with 'nano_'!")
                            .as_bytes()
                            [0..52] // drop 8 char checksum
                    );
                }
                Err(_) => {}
            }
        }

        read_tx.commit()?;
        
        let mut e1: Vec<String> = vec!["nano_11114w1fcd1suigthy87ymi5rqo3sky7fqkbjpdih5han5tp83tb4hoxgmy4".to_string(), "nano_11116yqsgoxg67cpbabzdqq3tc3s3cmnxpt7cb3b77sb6ddj3ztnfqq4ygkp".to_string(), "nano_1111gasqh5tfpi7ndj4qr847jnzcrhbcnq9rt71e7yfx4ucsbhihd3cxo68t".to_string(), "nano_1111j7m4pgxikpd9ngzyae3hnz19x8h7ouconc81dwzqrpackhysqyfouzxq".to_string(), "nano_1111nm1crbdkh1xx1nmkf976s6exnaery8ripbytpnxorz4mgss67pmgf88s".to_string()];
        let mut e2: Vec<String> = vec!["nano_31114csst1h94diax547k11so3ojiwn6g8osp48wh5mhzgirzi1rz8z1e1ms".to_string(), "nano_31114nprjd6y8kt8xihr5irkudufxwbarybmbj73hpjnyj3ok5rcqer1rc1f".to_string(), "nano_31116hrcowsegst8jxkidqjqi1kza5bisbh1yni575cyi5eowein5ycgew4t".to_string(), "nano_31116mh6pajix5zc1rpg3xy1t5jdb1wz3u4joqu1nq1fk3kup9pu39qbnhi7".to_string(), "nano_31118fqe53bteho64ome7k1sznb1qpwuokqumb56sdfmch8zxfc7wioxpjcc".to_string()];
        let mut e3: Vec<String> = vec!["nano_3bc11asob97osigir7na3k117znhgzh5bcyqm5srpgtgqibdqfm4wjyd4g4p".to_string(), "nano_3bc138ebwudk89hu3k5dxghs59y6u5mjpwurjrfmyasj46r78bfs79cgncw4".to_string(), "nano_3bc13wjf9awysqgkdgzf5diduy89srbxkdd6trcb3hd3scw1edea8dtmrjih".to_string(), "nano_3bc144nkhfdiiis5oxuy8jyg54gywryxuc1sf7p1w5jioechntsaprfxkffy".to_string(), "nano_3bc14emf3bwgfyoba4bsmk9hij11em9yaff9rbjni97fz3txqu7u631h5git".to_string()];
    
        e1.sort_by(|a, b| a.cmp(&b));
        e2.sort_by(|a, b| a.cmp(&b));
        e3.sort_by(|a, b| a.cmp(&b));
    
        let mut r1 = root.search("nano_1111");
        let mut r2 = root.search("nano_31");
        let mut r3 = root.search("nano_3bc");
        let r4 = root.search("nano_a");
        let r5 = root.search("nano_");
        let r6 = root.search("nano_2x");

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