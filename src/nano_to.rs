use std::fs::File;
use std::path::Path;
use git2::Repository;
use serde_json::map::Values;
use serde_json::{json, Value};

const REPO_PATH: &str = "~/.cache/nano_search/nano_to";

// https://docs.rs/git2/latest/git2/index.html
pub fn update() {
    match Path::new(REPO_PATH).exists() {
        true => {
            // https://stackoverflow.com/questions/58768910/how-to-perform-git-pull-with-the-rust-git2-crate
            let repo = Repository::open(REPO_PATH).unwrap();

            repo.find_remote("origin")
                .unwrap()
                .fetch(&["master"], None, None)
                .expect("Could not fetch origin:master");

            let fetch_head = repo.find_reference("FETCH_HEAD").unwrap();
            let fetch_commit = repo.reference_to_annotated_commit(&fetch_head).unwrap();
            let analysis = repo.merge_analysis(&[&fetch_commit]).unwrap();
            
            if analysis.0.is_fast_forward() {
                let refname = format!("refs/heads/{}", "master");
                let mut reference = repo.find_reference(&refname).unwrap();
                reference.set_target(fetch_commit.id(), "Fast-Forward").unwrap();
                repo.set_head(&refname).unwrap();
                repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force())).expect("Failed to fast-forward");
            }
        }
        false => {
            match Repository::clone("https://github.com/fwd/nano-to", REPO_PATH) {
                Err(e) => panic!("Failed to clone: {}", e),
                Ok(_) => (),
            };
        }
    }
}

// TODO: not sorted
pub fn search(alias: &str) -> Vec<Value> {
    let repo = Repository::open(REPO_PATH).unwrap();
    // do a prefix search on the hundred or so aliases
    // can probably get away with a sequential search
    let mut vec = Vec::new();

    let alias_lower = alias.to_ascii_lowercase();

    let file = File::open("~/.cache/nano_search/nano_to/known.json").expect("Failed to open known.json");
    let json: Value = serde_json::from_reader(file).expect("Failed to parse known.json");
    
    for acc in json.as_array().unwrap() {
        let name = acc["name"].as_str().unwrap().to_string();
        if name.to_ascii_lowercase().starts_with(&alias_lower) {
            let addr = acc["address"].as_str().unwrap().to_string();
            let val = json!({
                "aliased": true,
                "address": addr,
                "name": name
            });
            vec.push(val);
            if vec.len() >= 8 { break; }
        }
    }

    vec

}
