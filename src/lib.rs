use heed::{BoxedError, BytesDecode};

pub struct Accounts(pub [u8; 32]);

impl BytesDecode<'_> for Accounts {
    type DItem = [u8; 32];
    fn bytes_decode(bytes: &'_ [u8]) -> Result<Self::DItem, BoxedError> {
        Ok(bytes.try_into().expect("array must fit in a `[u8; 32]`"))
    }
}