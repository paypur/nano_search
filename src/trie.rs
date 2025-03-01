use std::cell::RefCell;
use std::cmp::min;
use std::fmt::{Display};
use std::rc::Rc;
use std::slice::Iter;
use nano_search::ByteString;

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

#[derive(Debug, PartialEq)]
pub struct TrieRefEdges(Box<[Option<TrieRef>]>);

impl TrieRefEdges {
    pub fn new() -> Self {
        Self(Box::new([]))
    }

    fn hash(&self, c: u8, size: usize) -> usize {
        assert!(self.0.len() <= 32);
        CHAR_INDEX_MAP[c as usize] % size
    }

    // out of capacity or collision
    fn reallocate(&mut self) {
        let double = if self.0.len() != 0 { self.0.len() * 2 } else { 1 } ;
        let mut new_vec: Vec<Option<TrieRef>> = Vec::with_capacity(double);

        // fill with None
        new_vec.resize(double, None);

        self.0.iter()
            .flatten()
            .for_each(|tr| {
                let hash = self.hash(tr.borrow().values[0], new_vec.len());
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
            return tr.borrow().values[0] == char;
        }

        false
    }

    pub fn insert(&mut self, trie: TrieRef) {
        if self.0.len() == 0 {
            self.reallocate();
        }

        let c = trie.borrow().values[0];
        let hash = self.hash(c, self.0.len());

        match &self.0[hash] {
            None => {
                // just insert
                self.0[hash] = Some(trie);
            },
            Some(tr) => {
                if tr.borrow().values[0] != c {
                    // println!("Collision between {} and {}", tr.borrow().values, char::from(c));
                    // collision
                    self.reallocate();
                    // recursive to ensure no collision after reallocating
                    self.insert(trie.clone());
                }
                // shouldn't insert when it already is in array
            },
        }
    }

    pub fn remove(&mut self, char: u8) -> bool {
        let hash = self.hash(char, self.0.len());
        if self.0[hash].is_some() {
            self.0[hash] = None;
            return true;
        }
        false
    }

    pub fn get(&self, char: u8) -> Option<TrieRef> {
        if self.0.len() == 0 {
            return None;
        }

        let hash = self.hash(char, self.0.len());
        if let Some(tr) = self.0.get(hash).unwrap() {
            if tr.borrow().values[0] == char {
                return Some(tr.clone());
            }
        }
        None
    }

    pub fn iter(&self) -> Iter<'_, Option<TrieRef>> {
        self.0.iter()
    }
}

impl<Idx> std::ops::Index<Idx> for TrieRefEdges
where
    Idx: std::slice::SliceIndex<[Option<TrieRef>]>,
{
    type Output = Idx::Output;

    fn index(&self, index: Idx) -> &Self::Output {
        &self.0[index]
    }
}

impl Display for TrieRefEdges {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[\n")?;
        let mut count = 0;
        self.iter().for_each(|o| {
            match o {
                None => write!(f, "  {}:None\n", count).unwrap(),
                Some(tr) => write!(f, "  {}:{}\n", count, tr.borrow()).unwrap(),
            }
            count += 1;
        });
        write!(f, "]\n")?;

        Ok(())
    }
}

#[derive(Debug)]
pub struct TrieMatch {
    pub trie: TrieRef,
    pub len: usize,
}

// no 0 2 l v
#[derive(Debug)]
#[derive(PartialEq)]
pub struct Trie {
    // edges keys are prefix free
    pub edges: TrieRefEdges,
    pub values: ByteString,
    pub is_terminal: bool,
}

impl Display for Trie {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let vec = self.full_tree();

        write!(f, "{}", vec.join("\n"))?;

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
            edges: TrieRefEdges::new(),
            values: ByteString::new(word),
            is_terminal: false,
        }
    }

    pub fn rc(word: &[u8]) -> TrieRef {
        Rc::new(RefCell::new(Trie::from(word)))
    }

    pub fn full_tree(&self) -> Vec<String> {
        let mut vec: Vec<String> = vec!();

        vec.push(format!("[{}|\"{}\"|{}]", self.values.len(), self.values.iter().map(|x| char::from(*x)).collect::<String>(), self.is_terminal));

        if !self.is_terminal {
            vec.append(
                &mut self.edges.iter()
                    .filter(|x| x.is_some())
                    .flat_map(|tr| {
                        tr.as_ref()
                            .unwrap()
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
        let mut target: Option<TrieMatch> = None;

        if let Some(trie) = self.edges.get(word[0]) {
            for i in 1..=min(word.len(), trie.borrow().values.len()) {
                if &word[0..i] != &trie.borrow().values[0..i] {
                    break;
                }
                if target.is_none() || i > target.as_ref().unwrap().len {
                    target = Some(
                        TrieMatch {
                            trie: trie.clone(),
                            len: i
                        }
                    );
                }
            }
        }

        target
    }

    pub fn build(&mut self, word: &[u8]) {
        match self.find_partial_match(word) {
            // no match
            None => {
                // insert whole vec
                let trie = Trie::rc(&word);
                trie.borrow_mut().is_terminal = true;
                self.edges.insert(trie);
            }
            Some(wrap) => {
                if wrap.len == wrap.trie.as_ref().borrow().values.len() {
                    wrap.trie.as_ref()
                        .borrow_mut()
                        .build(&word[wrap.len..]);
                } else {
                    // partial match
                    self.edges.remove(wrap.trie.borrow().values[0]);

                    // direct descendant
                    let trie_ref_prefix = Trie::rc(&word[0..wrap.len]);
                    self.edges.insert(trie_ref_prefix.clone());

                    // second descendants
                    let mut trie_mut_prefix = trie_ref_prefix.borrow_mut();
                    // slice of original word
                    trie_mut_prefix.build(&word[wrap.len..]);

                    // second descendants
                    // insert partially matching second trie
                    {
                        // fixes "already mutably borrowed"
                        let mut ref_mut = wrap.trie.borrow_mut();
                        ref_mut.values = ByteString::new(&ref_mut.values[wrap.len..]);
                    }

                    trie_mut_prefix.edges.insert(wrap.trie);
                }
            }
        }
    }

    pub fn search(&self, prefix: &[u8]) -> Vec<String> {
        let string = ByteString::string(prefix);
        println!("Looking for addresses with prefix \"{}\"", string);

        if !prefix.is_empty() {
            let string2 = string.clone();
            let mut chars = string2.chars();
            chars.next_back();

            let base = self.find_base(&prefix);
            if let Some(b) = &base {
                return b.as_ref()
                    .borrow()
                    .auto_complete(chars.as_str().to_string()) // need to remove 1 char from the right
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

        for trie_ref in self.edges.iter().filter(|x| x.is_some()) {
            let trie = trie_ref.as_ref().unwrap().borrow();

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
