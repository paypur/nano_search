use std::fmt::Display;
use heed::{BoxedError, BytesDecode};

pub struct Accounts(pub [u8; 32]);

impl BytesDecode<'_> for Accounts {
    type DItem = [u8; 32];
    fn bytes_decode(bytes: &'_ [u8]) -> Result<Self::DItem, BoxedError> {
        Ok(bytes.try_into().expect("array must fit in a `[u8; 32]`"))
    }
}

#[derive(Debug, PartialEq)]
pub struct ByteString(Box<[u8]>);

impl ByteString {
    pub fn new(bytes: &[u8]) -> Self {
        Self (
            Vec::from(bytes).into_boxed_slice()
        )
    }

    pub fn string(word: &[u8]) -> String {
        word.iter().map(|x| char::from(*x)).collect::<String>()
    }

    pub fn to_string(&self) -> String {
        self.iter().map(|x| char::from(*x)).collect::<String>()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn iter(&self) -> std::slice::Iter<u8> {
        self.0.iter()
    }
}

impl<Idx> std::ops::Index<Idx> for ByteString
where
    Idx: std::slice::SliceIndex<[u8]>,
{
    type Output = Idx::Output;

    fn index(&self, index: Idx) -> &Self::Output {
        &self.0[index]
    }
}

impl Display for ByteString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())?;
        Ok(())
    }
}
