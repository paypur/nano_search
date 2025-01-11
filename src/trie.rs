use std::cell::RefCell;
use std::cmp::Ordering;
use std::rc::Rc;
use sorted_vec::FindOrInsert::{Found, Inserted};
use sorted_vec::SortedSet;

const AUTO_COMPLETE_LIMIT: usize = 5;

pub type TrieRef = Rc<RefCell<Trie>>;

// no 0 2 l v
#[derive(Debug)]
pub struct Trie {
    pub edges: SortedSet<TrieRef>,
    pub value: u8,
    pub is_terminal: bool,
}

impl Ord for Trie {
    fn cmp(&self, other: &Self) -> Ordering {
        self.value.cmp(&other.value)
    }
}

impl PartialOrd for Trie {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Trie {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl Eq for Trie {

}

/*
$ rustc +nightly -Zprint-type-sizes src/test.rs
*/

impl Trie {
    pub fn new() -> Trie {
        Self::from(0)
    }

    pub fn from(c: u8) -> Trie {
        Trie {
            edges: SortedSet::with_capacity(1),
            value: c,
            is_terminal: false,
        }
    }

    pub fn binary_search(&self, c: u8) -> Option<TrieRef> {
        let mut l = 0;
        let mut h = self.edges.len() - 1;

        while l <= h {
            let m = l + (h - l) >> 1;
            if self.edges[m].borrow().value < c {
                l = m + 1;
            } else if self.edges[m].borrow().value > c {
                h = m - 1;
            } else {
                return Some(self.edges[m].clone());
            }
        }

        None
    }

    pub fn build(&mut self, word: &str) {
        let c = word.chars().nth(0).unwrap() as u8;

        let index = match self.edges.find_or_insert(Rc::new(RefCell::new(Trie::from(c)))) {
            Found(i) => i,
            Inserted(i) => i,
            _ => 0,
        };

        if word.len() == 1 {
            self.edges[index]
                .as_ref()
                .borrow_mut()
                .is_terminal = true;
        } else {
            self.edges[index]
                .as_ref()
                .borrow_mut()
                .build(&word[1..word.len()]);
        }

    }


    pub fn search(&self, prefix: String) -> Vec<String> {
        println!("Looking for addresses with prefix \"{}\"", prefix);

        let base = self.find(&prefix);

        if let Some(b) = &base {
            let mut dup = prefix.clone();
            let _ = dup.pop();
            return b.as_ref()
                .borrow()
                .auto_complete(dup)
        }

        vec!()
    }


    fn find(&self, word: &str) -> Option<TrieRef> {
        let c = word.chars().nth(0).unwrap() as u8;

        match &self.binary_search(c) {
            None => {
                None
            }
            Some(trie) => {
                if word.len() == 1 {
                    Some(trie.clone())
                } else {
                    trie.as_ref()
                        .borrow()
                        .find(&word[1..word.len()])
                }

            }
        }
    }

    fn auto_complete(&self, mut prefix: String) -> Vec<String> {
        prefix.push(self.value as char);

        if self.is_terminal {
            return vec!(prefix);
        }

        let mut collect_vec: Vec<String> = Vec::new();

        for opt in self.edges.iter() {
            let trie = opt.as_ref()
                .borrow();

            let mut vec = trie.auto_complete(prefix.clone());

            if vec.len() == 0 {
                continue;
            }

            collect_vec.append(&mut vec);

            if collect_vec.len() == AUTO_COMPLETE_LIMIT {
                return collect_vec;
            } else if collect_vec.len() > AUTO_COMPLETE_LIMIT {
                return collect_vec.drain(AUTO_COMPLETE_LIMIT..collect_vec.len()).collect();
            }
        }

        return collect_vec;
    }
}

