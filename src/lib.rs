use std::fmt::{Display, Formatter};
use std::ops::Deref;
use heed::{BoxedError, BytesDecode};

// https://docs.rs/heed/latest/heed/
pub struct AccountsKey([u8; 32]);

impl BytesDecode<'_> for AccountsKey {
    type DItem = [u8; 32];
    fn bytes_decode(bytes: &'_ [u8]) -> Result<Self::DItem, BoxedError> {
        Ok(bytes.try_into().expect("AccountsKey must fit in a `[u8; 32]`"))
    }
}

pub struct Bytes128([u8; 128]);

impl BytesDecode<'_> for Bytes128 {
    type DItem = [u8; 128];
    fn bytes_decode(bytes: &'_ [u8]) -> Result<Self::DItem, BoxedError> {
        Ok(
            bytes[0..128]
                .try_into()
                .expect(format!("Array of size {} must fit in a `[u8; 128]`", bytes.len()).as_str())
        )
    }
}

// not actually needed but whatever
pub struct NanoBlock([u8; 32]);

impl PartialEq<[u8; 32]> for NanoBlock {
    fn eq(&self, other: &[u8; 32]) -> bool {
        self.0 == *other
    }
}

impl NanoBlock {
    fn to_string(&self) -> String {
        self.0.iter()
            .map(|b| format!("{:02X}", b))
            .collect()
    }
}

impl From<&[u8]> for NanoBlock {
    fn from(bytes: &[u8]) -> Self {
        Self(bytes.try_into().expect("Block size must fit in a `[u8; 32]`"))
    }
}

impl Display for NanoBlock {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())?;
        Ok(())
    }
}

pub struct AccountsValue {
    pub head: [u8; 32],
    pub representative: [u8; 32],
    pub open_block: NanoBlock,
    pub balance: u128,
    pub modified: u64,
    pub block_count: u64,
}

impl AccountsValue {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        assert_eq!(bytes.len(), 128);
        // TODO: worry about byte order for struct member order
        Self {
            head: bytes[0..32].try_into().unwrap(),
            representative: bytes[32..64].try_into().unwrap(),
            open_block: NanoBlock::from(&bytes[64..96]),
            balance: u128::from_le_bytes(bytes[96..112].try_into().unwrap()),
            modified: u64::from_le_bytes(bytes[112..120].try_into().unwrap()),
            block_count: u64::from_le_bytes(bytes[120..128].try_into().unwrap()),
        }
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
           Self::string(&self)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn iter(&self) -> std::slice::Iter<u8> {
        self.0.iter()
    }
}

impl Deref for ByteString {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        &self.0
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
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())?;
        Ok(())
    }
}