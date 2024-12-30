#[derive(Debug)]
pub struct Trie {
    pub edges: [Option<Box<Trie>>; 36],
    pub value: u8,
    pub is_terminal: bool,
}

impl Trie {
    pub fn new() -> Trie {
        Trie {
            edges: [const { None }; 36],
            value: 0,
            is_terminal: false,
        }
    }

    pub fn from(c: u8) -> Trie {
        Trie {
            edges: [const { None }; 36],
            value: c,
            is_terminal: false,
        }
    }

    pub fn index_of(char: u8) -> usize {
        let mut i = char - b'0';
        if i > b'9' - b'0' {
            i -= b'a' - b':';
        }
        i.try_into().unwrap()
    }

    pub fn build(&mut self, word: &[u8]) {
        let i = Self::index_of(word[0]);

        if self.edges[i].is_none() {
            let mut b = Box::new(Trie::from(word[0]));
            if word.len() == 1 {
                b.is_terminal = true;
            } else {
                b.build(&word[1..word.len()]);
            }
            self.edges[i] = Some(b);
        }
    }

    pub fn lookup(&self, word: &[u8]) -> Option<Trie> {
        let i = Self::index_of(word[0]);
        if self.edges[i].is_some() {
            if word.len() == 1 {
                self.edges[i]
            } else {
                self.edges[i]?.lookup(&word[1..word.len()])
            }
        } else {
            None
        }
    }

    pub fn auto_complete(&self, word: &[u8]) -> () {}
}

