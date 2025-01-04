use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;

const AUTO_COMPLETE_LIMIT: usize = 5;

type TrieRef = Rc<RefCell<Trie>>;

#[derive(Debug)]
pub struct Trie {
    pub edges: [Option<TrieRef>; 36],
    pub value: char,
    pub is_terminal: bool,
}

impl Trie {
    pub fn new() -> Trie {
        Trie {
            edges: [const { None }; 36],
            value: '\0',
            is_terminal: false,
        }
    }

    pub fn from(c: char) -> Trie {
        Trie {
            edges: [const { None }; 36],
            value: c,
            is_terminal: false,
        }
    }

    pub fn index_of(c: char) -> usize {
        let mut i = c as usize - '0' as usize;
        if i > '9' as usize - b'0' as usize {
            i -= 'a' as usize - ':' as usize;
        }
        i
    }

    pub fn build(&mut self, word: &str) {
        let c = word.chars().nth(0).unwrap();
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
        let c = word.chars().nth(0).unwrap();
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

