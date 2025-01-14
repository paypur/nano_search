use std::cell::RefCell;
use std::cmp::{max, min};
use std::collections::HashMap;
use std::fmt::Display;
use std::rc::Rc;


const AUTO_COMPLETE_LIMIT: usize = 5;

pub type TrieRef = Rc<RefCell<Trie>>;

// no 0 2 l v
#[derive(Debug)]
pub struct Trie {
    pub edges: HashMap<Vec<u8>, TrieRef>,
    pub values: Vec<u8>,
    pub is_terminal: bool,
    pub depth: usize,
}

// TODO: depth doesnt work properly
impl Display for Trie {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = self.values.iter().map(|x| char::from(*x)).collect::<String>();
        let mut vec: Vec<_> = self.edges.values().map(|e| e.as_ref().borrow()).collect();
        vec.sort_by(|a, b| a.values.len().cmp(&b.values.len()));
        
        write!(f, "[d:{}|l:{}|\"{}\"|{}]", self.depth, string.len(), string, self.is_terminal)?;
        
        vec.iter().for_each(|trie| {
            write!(f, "\n{}{}", "  ".repeat(self.depth + 1), trie).unwrap();
        });
        Ok(())
    }
}

/*
$ rustc +nightly -Zprint-type-sizes src/trie.rs
*/

impl Trie {
    pub fn new() -> Trie {
        Self::from(&[])
    }

    pub fn from(word: &[u8]) -> Trie {
        Trie {
            edges: HashMap::with_capacity(1),
            values: Vec::from(word),
            is_terminal: false,
            depth: 0,
        }
    }

    pub fn to_ref(self) -> TrieRef {
        Rc::new(RefCell::new(self))
    }

    pub fn build(&mut self, word: &[u8]) {
        let mut chars = 0;
        let mut trie_target: Option<TrieRef> = None;

        self.edges.values().for_each(|tr| {
            let trie = tr.as_ref().borrow_mut();
            for i in 1..=min(word.len(), trie.values.len()) {
                if &word[0..i] == &trie.values[0..i] {
                    if i > chars {
                        chars = i;
                        trie_target = Some(tr.clone());
                    }
                } else {
                    break;
                }
            }
        });
        

        match trie_target {
            // no match
            None => {
                // insert whole vec
                let mut trie = Trie::from(&word);
                trie.is_terminal = true;
                trie.depth = self.depth + 1;
                self.edges.insert(word.to_vec(), trie.to_ref());
            }
            Some(trie_ref) => {
                // full match
                if chars == trie_ref.as_ref().borrow().values.len() {
                    trie_ref.as_ref()
                        .borrow_mut()
                        .build(&word[chars..]);
                } else {
                    // partial match
                    let trie_ref2 = trie_ref.clone();
    
                    trie_ref2.as_ref().borrow_mut().depth = self.depth + 2;
    
                    let mut trie = trie_ref
                        .as_ref()
                        .borrow_mut();
    
                    self.edges.remove(&trie.values);
    
                    trie.values.drain(0..chars);
    
                    // direct descendant
                    let mut trie_ref_pre = Trie::from(&word[0..chars]).to_ref();
                    self.edges.insert(word[0..chars].to_vec(), trie_ref_pre.clone());
    
                    let mut trie_pre = trie_ref_pre.as_ref().borrow_mut();
    
                    // second descendants
                    // slice of original word
                    trie_pre.depth = self.depth + 1;
                    trie_pre.build(&word[chars..]);
    
                    // second descendants
                    // manually insert second trie
                    trie_pre.edges.insert(trie.values.clone(), trie_ref2);
                    
                }
            }
        }
    }

    // pub fn search(&self, prefix: String) -> Vec<String> {
    //     println!("Looking for addresses with prefix \"{}\"", prefix);
    //
    //     let base = self.find_base(&prefix);
    //
    //     if let Some(b) = &base {
    //         let mut dup = prefix.clone();
    //         let _ = dup.pop();
    //         return b.as_ref()
    //             .borrow()
    //             .auto_complete(dup)
    //     }
    //
    //     vec!()
    // }

    pub fn find_base(&self, word: &[u8]) -> Option<TrieRef> {
        // find key that contains the most chars
        let mut l = 0;
        let mut h = word.len() - 1;

        while l < h {
            let m = (h + l) / 2;

            if self.edges.contains_key(&word[0..m]) {
                l = m;
            } else {
                h = m - 1;
            }

        }

        if l == 0 {
            return None;
        }

        if l == word.len() {
            return self.edges.get(&word[0..l].to_vec()).cloned();
        }

        let trie_ref = self.edges.get(&word[0..h]).unwrap();

        trie_ref.as_ref()
            .borrow()
            .find_base(&word[l..])
    }

    // fn auto_complete(&self, mut prefix: String) -> Vec<String> {
    //     prefix.push(self.value as char);
    //
    //     if self.is_terminal {
    //         return vec!(prefix);
    //     }
    //
    //     let mut collect_vec: Vec<String> = Vec::new();
    //
    //     for opt in self.edges.iter() {
    //         if opt.is_some() {
    //             let trie = opt.as_ref()
    //                 .unwrap()
    //                 .borrow();
    //
    //             let mut vec = trie.auto_complete(prefix.clone());
    //
    //             if vec.len() == 0 {
    //                 continue;
    //             }
    //
    //             collect_vec.append(&mut vec);
    //
    //             if collect_vec.len() == AUTO_COMPLETE_LIMIT {
    //                 return collect_vec;
    //             } else if collect_vec.len() > AUTO_COMPLETE_LIMIT {
    //                 return collect_vec.drain(AUTO_COMPLETE_LIMIT..collect_vec.len()).collect();
    //             }
    //         }
    //     }
    //
    //     collect_vec
    // }
}

fn main() {
}
