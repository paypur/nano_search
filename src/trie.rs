use std::cell::RefCell;
use std::rc::Rc;
use nano_search::CHAR_INDEX_MAP;

const AUTO_COMPLETE_LIMIT: usize = 5;

type TrieRef = Rc<RefCell<Trie>>;


// no 0 2 l v
#[derive(Debug)]
pub struct Trie {
    pub edges: [Option<TrieRef>; 32],
    pub value: u8,
    pub is_terminal: bool,
}

/*
$ rustc +nightly -Zprint-type-sizes src/test.rs
print-type-size type: `Trie`: 264 bytes, alignment: 8 bytes
print-type-size     field `.edges`: 256 bytes
print-type-size     field `.value`: 1 bytes
print-type-size     field `.is_terminal`: 1 bytes
print-type-size     end padding: 6 bytes
*/

impl Trie {
    pub fn new() -> Trie {
        Trie {
            edges: [const { None }; 32],
            value: 0,
            is_terminal: false,
        }
    }

    pub fn from(c: u8) -> Trie {
        Trie {
            edges: [const { None }; 32],
            value: c,
            is_terminal: false,
        }
    }

    pub fn index_of(c: u8) -> usize {
        CHAR_INDEX_MAP[c as usize] as usize
    }

    pub fn build(&mut self, word: &str) {
        let c = word.chars().nth(0).unwrap() as u8;
        let i = Self::index_of(c);

        if self.edges[i].is_none() {
            self.edges[i] = Some(Rc::new(RefCell::new(Trie::from(c))));
        }

        if word.len() == 1 {
            self.edges[i].as_ref()
                .unwrap()
                .borrow_mut().is_terminal = true;
        } else {
            self.edges[i].as_ref()
                .unwrap()
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
        let i = Self::index_of(c);

        match &self.edges[i] {
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
            if opt.is_some() {
                let trie = opt.as_ref()
                    .unwrap()
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
        }

        return collect_vec;
    }
}

