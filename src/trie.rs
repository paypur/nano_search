use std::cell::RefCell;
use std::cmp::{min};
use std::collections::HashMap;
use std::fmt::Display;
use std::rc::Rc;


const AUTO_COMPLETE_LIMIT: usize = 5;

pub type TrieRef = Rc<RefCell<Trie>>;

#[derive(Debug)]
pub struct TrieMatch {
    pub trie: TrieRef,
    pub len: usize,
}

// no 0 2 l v
#[derive(Debug)]
pub struct Trie {
    pub edges: HashMap<Vec<u8>, TrieRef>,
    pub values: Vec<u8>, // TODO: make boxed array // TODO: not really needed with the previous hashmap also contains it
    pub is_terminal: bool,
    pub depth: usize, // TODO: remove
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

    pub fn increment_depth(&mut self, d: usize) {
        self.depth += d;
        self.edges.values().for_each(|(trie_ref)| {
            trie_ref.as_ref().borrow_mut().increment_depth(self.depth);
        })
    }

    pub fn build(&mut self, word: &[u8]) {
        let mut chars = 0;
        let mut trie_target: Option<TrieRef> = None;

        self.edges.values().for_each(|tr| {
            let trie = tr.as_ref().borrow_mut();
            for i in 1..=min(word.len(), trie.values.len()) {
                if &word[0..i] != &trie.values[0..i] {
                    break;
                }
                if i > chars {
                    chars = i;
                    trie_target = Some(tr.clone());
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

                    trie_ref2.as_ref().borrow_mut().increment_depth(1);

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
                    trie_pre.increment_depth(1);
                    trie_pre.build(&word[chars..]);

                    // second descendants
                    // insert partially matching second trie
                    trie_pre.edges.insert(trie.values.clone(), trie_ref2);

                }
            }
        }
    }

    pub fn search(&self, prefix: &[u8]) -> Vec<String> {
        let string = prefix.iter().map(|x| char::from(*x)).collect::<String>();
        println!("Looking for addresses with prefix \"{}\"", string);
        if !prefix.is_empty() {
            let string2 = prefix[..prefix.len()-1].iter().map(|x| char::from(*x)).collect::<String>();
    
            let base = self.find_base(&prefix);
            if let Some(b) = &base {
                return b.as_ref()
                    .borrow()
                    .auto_complete(string2) // need to remove 1 char from the right
            }
        }

        vec!()
    }

    pub fn find_partial_match(&self, word: &[u8]) -> Option<TrieMatch> {
        let mut target: Option<TrieMatch> = None;

        self.edges.values().for_each(|tr| {
            let trie = tr.as_ref().borrow_mut();
            for i in 1..=min(word.len(), trie.values.len()) {
                if &word[0..i] != &trie.values[0..i] {
                    break;
                }
                if target.is_none() || i > target.as_ref().unwrap().len {
                    target = Some(TrieMatch { trie: tr.clone(), len: i });
                }
            }
        });

        target
    }
    
    pub fn find_base(&self, word: &[u8]) -> Option<TrieRef> {
        let partial = self.find_partial_match(word);
        
        match partial {
            None => None,
            Some(trie_match) => {
                // full match
                if trie_match.len == word.len() {
                    return Some(trie_match.trie.clone())
                }

                let trie = trie_match.trie
                    .as_ref()
                    .borrow();

                assert_eq!(&trie.values.as_slice()[..trie_match.len], &word[..trie_match.len]);

                // partial match
                // even though all chars don't match, the edge should be the only possible path
                return trie.find_base(&word[trie.values.len()..]);
            }
        }
    }

    fn auto_complete(&self, mut prefix: String) -> Vec<String> {
        prefix.push_str(&self.values.iter().map(|x| char::from(*x)).collect::<String>());

        if self.is_terminal {
            return vec!(prefix);
        }

        let mut collect_vec: Vec<String> = Vec::new();

        // TODO: ordering is still!! wrong
        let mut x1 = self.edges.values().collect::<Vec<&TrieRef>>();
        x1.sort_by(|a, b| a.as_ref().borrow().values.cmp(&b.as_ref().borrow().values));
        for trie_ref in x1 {
            let trie = trie_ref.as_ref().borrow();

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

        collect_vec
    }
}
