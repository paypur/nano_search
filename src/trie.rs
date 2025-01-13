use std::cell::RefCell;
use std::rc::Rc;

const AUTO_COMPLETE_LIMIT: usize = 5;

pub type TrieRef = Rc<RefCell<Trie>>;


#[derive(Debug)]
pub struct TrieRefVec(Vec<TrieRef>);

impl TrieRefVec {
    pub fn new() -> Self {
        TrieRefVec(Vec::with_capacity(0))
    }

    pub fn pos(&self, c: u8) -> usize {
        if self.0.len() == 0 {
            return 0;
        }

        let mut l: i32 = 0;
        let mut h: i32 = self.0.len() as i32 - 1;

        while l <= h {
            let m: usize = (l + (h - l) / 2).try_into().unwrap();
            let v = self.0.as_slice()[m].borrow().value;
            if v < c {
                l = m as i32 + 1;
            } else if v > c {
                h = m as i32 - 1;
            } else {
                return m;
            }
        }

        l as usize
    }

    pub fn contains(&mut self, char: u8) -> bool {
        let i = self.pos(char);

        if self.0.len() <= i {
            return false;
        }

        self.0.as_slice()[i].borrow().value == char
    }

    pub fn insert(&mut self, char: u8) {
        if self.0.capacity() == 0 {
            self.0 = Vec::with_capacity(1);
        }

        self.0.insert(self.pos(char), Rc::new(RefCell::new(Trie::from(char))))
    }

    pub fn as_mut_slice(&mut self) -> &mut [TrieRef] {
        self.0.as_mut_slice()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

}



// no 0 2 l v
#[derive(Debug)]
pub struct Trie {
    pub edges: TrieRefVec,
    pub value: u8,
    pub is_terminal: bool,
}

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

    pub fn build(&mut self, word: &str) {
        let c = word.chars().nth(0).unwrap() as u8;

        if self.edges.len() == 0 || !self.edges.contains(c) {
            self.edges.insert(c);
        }

        let index = self.edges.pos(c);

        if word.len() == 1 {
            self.edges.as_mut_slice()[index]
                .as_ref()
                .borrow_mut()
                .is_terminal = true;
        } else {
            self.edges.as_mut_slice()[index]
                .as_ref()
                .borrow_mut()
                .build(&word[1..word.len()]);
        }

    }

    // pub fn search(&self, prefix: String) -> Vec<String> {
    //     println!("Looking for addresses with prefix \"{}\"", prefix);
    //
    //     let base = self.find(&prefix);
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
    //
    //
    // fn find(&self, word: &str) -> Option<TrieRef> {
    //     let c = word.chars().nth(0).unwrap() as u8;
    //
    //     match &self.binary_search(c) {
    //         None => {
    //             None
    //         }
    //         Some(trie) => {
    //             if word.len() == 1 {
    //                 Some(trie.clone())
    //             } else {
    //                 trie.as_ref()
    //                     .borrow()
    //                     .find(&word[1..word.len()])
    //             }
    //
    //         }
    //     }
    // }
    //
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
    //         let trie = opt.as_ref()
    //             .borrow();
    //
    //         let mut vec = trie.auto_complete(prefix.clone());
    //
    //         if vec.len() == 0 {
    //             continue;
    //         }
    //
    //         collect_vec.append(&mut vec);
    //
    //         if collect_vec.len() == AUTO_COMPLETE_LIMIT {
    //             return collect_vec;
    //         } else if collect_vec.len() > AUTO_COMPLETE_LIMIT {
    //             return collect_vec.drain(AUTO_COMPLETE_LIMIT..collect_vec.len()).collect();
    //         }
    //     }
    //
    //     return collect_vec;
    // }
}

