use std::cell::RefCell;
use std::cmp::min;
use std::collections::HashMap;
use std::fmt::{Display};
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
    pub edges: HashMap<Box<[u8]>, TrieRef>,
    pub values: Box<[u8]>,
    pub is_terminal: bool,
}

impl Display for Trie {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let vec = self.full_tree();
        
        vec.iter().for_each(|s| {
            write!(f, "{}\n", s).unwrap();
        });

        // let vec = self.edges.values().collect::<Vec<_>>();
        // 
        // vec.iter().for_each(|s| {
        //     write!(f, "{}\n", s.as_ref().borrow().values.iter().map(|x| char::from(*x)).collect::<String>()).unwrap();
        // });

        Ok(())
    }
}

impl Trie {
    pub fn new() -> Self {
        Self::from(&[])
    }

    pub fn from(word: &[u8]) -> Self {
        Trie {
            edges: HashMap::with_capacity(1),
            values: Vec::from(word).into_boxed_slice(),
            is_terminal: false,
        }
    }

    pub fn to_ref(self) -> TrieRef {
        Rc::new(RefCell::new(self))
    }

    pub fn full_tree(&self) -> Vec<String> {
        let mut vec: Vec<String> = vec!();

        vec.push(format!("[{}|\"{}\"|{}]", self.values.len(), self.values.iter().map(|x| char::from(*x)).collect::<String>(), self.is_terminal));

        if !self.is_terminal {
            vec.append(
                &mut self.edges.values()
                    .flat_map(|tr| {
                    tr.as_ref()
                        .borrow()
                        .full_tree()
                        .into_iter()
                        .map(|string| {
                            let mut pad = "   ".to_string();
                            pad.push_str(&string);
                            pad
                        })
                }).collect::<Vec<String>>()
            );
        }

        vec
    }
    
    pub fn find_partial_match(&self, word: &[u8]) -> Option<TrieMatch> {
        for i in 0..word.len()  {
            if let Some(trie) = self.edges.get(&word[0..word.len()-i]) {
                // println!("yay matched {}", word[0..word.len()-i].iter().map(|x| char::from(*x)).collect::<String>());
                return Some(
                    TrieMatch {
                        trie: trie.clone(),
                        len: word.len()-i,
                    }
                )
            }
        }
        let mut target: Option<TrieMatch> = None;

        // sucks
        self.edges.values().for_each(|tr| {
            // println!("fail");
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

    pub fn build(&mut self, word: &[u8]) {
        match self.find_partial_match(word) {
            // no match
            None => {
                // insert whole vec
                let mut trie = Trie::from(&word);
                trie.is_terminal = true;
                self.edges.insert(Box::from(word), trie.to_ref());
            }
            Some(wrap) => {
                if wrap.len == wrap.trie.as_ref().borrow().values.len() {
                    wrap.trie.as_ref()
                        .borrow_mut()
                        .build(&word[wrap.len..]);
                } else {
                    // partial match
                    let trie_ref2 = wrap.trie.clone();

                    let mut trie = wrap.trie
                        .as_ref()
                        .borrow_mut();

                    self.edges.remove(&trie.values);

                    trie.values = Vec::from(&trie.values[wrap.len..]).into_boxed_slice();
                    
                    // direct descendant
                    let trie_ref_pre = Trie::from(&word[0..wrap.len]).to_ref();
                    self.edges.insert(Box::from(&word[0..wrap.len]), trie_ref_pre.clone());
                    
                    let mut trie_pre = trie_ref_pre.as_ref().borrow_mut();
                    
                    // second descendants
                    // slice of original word
                    trie_pre.build(&word[wrap.len..]);
                    
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
    
    pub fn find_base(&self, word: &[u8]) -> Option<TrieRef> {
        let partial = self.find_partial_match(word);
        
        match partial {
            Some(wrap) => {
                // full match
                if wrap.len == word.len() {
                    return Some(wrap.trie.clone())
                }
    
                let trie = wrap.trie
                    .as_ref()
                    .borrow();
    
                assert_eq!(&trie.values[..wrap.len], &word[..wrap.len]);
    
                // partial match
                // even though all closest.len don't match, the edge should be the only possible path
                return trie.find_base(&word[trie.values.len()..]);
            }
            None => None,
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
