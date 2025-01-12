use std::cell::RefCell;
use std::rc::Rc;

const CHAR_INDEX_MAP: [usize; 128] = [
    0,0,0,0,0,0,0,0,
    0,0,0,0,0,0,0,0,
    0,0,0,0,0,0,0,0,
    0,0,0,0,0,0,0,0,
    0,0,0,0,0,0,0,0,
    0,0,0,0,0,0,0,0,
    0,0x0,0,1,2,3,4,5,
    6,7,0,0,0,0,0,0,

    0,0,0,0,0,0,0,0,
    0,0,0,0,0,0,0,0,
    0,0,0,0,0,0,0,0,
    0,0,0,0,0,0,0,0,
    0, 8, 9,10,11,12,13,14,
    15,16,17,18, 0,19,20,21,
    22,23,24,25,26,27, 0,28,
    29,30,31, 0, 0, 0, 0, 0
];
const AUTO_COMPLETE_LIMIT: usize = 5;

pub type TrieRef = Rc<RefCell<Trie>>;

#[derive(Debug)]
pub struct TrieRefVec (Box<[Option<TrieRef>]>);

impl TrieRefVec {
    pub fn new() -> Self {
        Self(Box::new([]))
    }

    pub fn hash(&self, c: u8, size: usize) -> usize {
        assert!(self.0.len() <= 32);
        CHAR_INDEX_MAP[c as usize] % size
    }

    // out of capacity
    // or collision
    pub fn reallocate(&mut self) {
        let double = if self.0.len() != 0 { self.0.len() * 2 } else { 1 } ;
        let mut new_vec: Vec<Option<TrieRef>> = Vec::with_capacity(double);

        // fill with None
        new_vec.resize(double, None);

        self.0.iter()
            .flatten()
            .for_each(|tr| {
                let hash = self.hash(tr.borrow().value, new_vec.len());
                // all these existing values should not collide
                new_vec[hash] = Some(tr.clone());
            }
        );

        self.0 = new_vec.into_boxed_slice();
    }

    pub fn contains(&self, char: u8) -> bool {
        if self.0.len() == 0 {
            return false;
        }

        let hash = self.hash(char, self.0.len());
        if let Some(tr) = self.0.get(hash).unwrap() {
            return tr.borrow().value == char;
        }

        false
    }

    pub fn insert(&mut self, trie: TrieRef) {
        if self.0.len() == 0 {
            self.reallocate();
        }

        let hash = self.hash(trie.borrow().value, self.0.len());
        match self.0.get(hash).unwrap() {
            None => {
                // just insert
                self.0[hash] = Some(trie);
            },
            Some(t) => {
                if t.borrow().value != trie.borrow().value {
                    // collision
                    self.reallocate();
                    // recursive to ensure no collision after reallocating
                    self.insert(trie);
                }
                // shouldn't insert when it already is in array
            },
        }
    }

    pub fn get(&self, char: u8) -> Option<TrieRef> {
        let hash = self.hash(char, self.0.len());
        if let Some(tr) = self.0.get(hash).unwrap() {
            if tr.borrow().value == char {
                return Some(tr.clone());
            }
        }
        None
    }

    pub fn iter(&self) -> std::slice::Iter<'_, Option<Rc<RefCell<Trie>>>> {
        self.0.iter()
    }
}


// no 0 2 l v
#[derive(Debug)]
pub struct Trie {
    pub edges: TrieRefVec,
    pub value: u8,
    pub is_terminal: bool,
}

/*
$ rustc +nightly -Zprint-type-sizes src/trie.rs
*/

impl Trie {
    pub fn new() -> Trie {
        Self::from(0)
    }

    pub fn from(c: u8) -> Trie {
        Trie {
            edges: TrieRefVec::new(),
            value: c,
            is_terminal: false,
        }
    }

    pub fn from_ref(c: u8) -> TrieRef {
        Rc::new(RefCell::new(Trie::from(c)))
    }

    pub fn build(&mut self, word: &str) {
        let c = word.chars().nth(0).unwrap() as u8;
        // let i = Self::index_of(c);

        if !self.edges.contains(c) {
            self.edges.insert(Trie::from_ref(c));
        }

        let tr = self.edges.get(c)
            .unwrap();

        let mut trie = tr
            .as_ref()
            .borrow_mut();

        if word.len() == 1 {
            trie.is_terminal = true;
        } else {
            trie.build(&word[1..word.len()]);
        }
    }

    pub fn search(&self, prefix: String) -> Vec<String> {
        println!("Looking for addresses with prefix \"{}\"", prefix);

        let base = self.find_base(&prefix);

        if let Some(b) = &base {
            let mut dup = prefix.clone();
            let _ = dup.pop();
            return b.as_ref()
                .borrow()
                .auto_complete(dup)
        }

        vec!()
    }


    fn find_base(&self, word: &str) -> Option<TrieRef> {
        let c = word.chars().nth(0)? as u8;

        match self.edges.get(c) {
            None => {
                None
            }
            Some(trie) => {
                if word.len() == 1 {
                    Some(trie.clone())
                } else {
                    trie.as_ref()
                        .borrow()
                        .find_base(&word[1..word.len()])
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

        collect_vec
    }
}

fn main() {
}
