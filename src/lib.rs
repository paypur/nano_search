use std::hash::{Hash};
use heed::{BoxedError, BytesDecode};

pub const CHAR_INDEX_MAP: [usize; 128] = [
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

/*
println!("{}", crate::CHAR_BYTE_MAP['1' as usize]);
println!("{}", crate::CHAR_BYTE_MAP['3' as usize]);
println!("{}", crate::CHAR_BYTE_MAP['4' as usize]);
println!("{}", crate::CHAR_BYTE_MAP['5' as usize]);
println!("{}", crate::CHAR_BYTE_MAP['6' as usize]);
println!("{}", crate::CHAR_BYTE_MAP['7' as usize]);
println!("{}", crate::CHAR_BYTE_MAP['8' as usize]);
println!("{}", crate::CHAR_BYTE_MAP['9' as usize]);
println!("{}", crate::CHAR_BYTE_MAP['a' as usize]);
println!("{}", crate::CHAR_BYTE_MAP['b' as usize]);
println!("{}", crate::CHAR_BYTE_MAP['c' as usize]);
println!("{}", crate::CHAR_BYTE_MAP['d' as usize]);
println!("{}", crate::CHAR_BYTE_MAP['e' as usize]);
println!("{}", crate::CHAR_BYTE_MAP['f' as usize]);
println!("{}", crate::CHAR_BYTE_MAP['g' as usize]);
println!("{}", crate::CHAR_BYTE_MAP['h' as usize]);
println!("{}", crate::CHAR_BYTE_MAP['i' as usize]);
println!("{}", crate::CHAR_BYTE_MAP['j' as usize]);
println!("{}", crate::CHAR_BYTE_MAP['k' as usize]);
println!("{}", crate::CHAR_BYTE_MAP['m' as usize]);
println!("{}", crate::CHAR_BYTE_MAP['n' as usize]);
println!("{}", crate::CHAR_BYTE_MAP['o' as usize]);
println!("{}", crate::CHAR_BYTE_MAP['p' as usize]);
println!("{}", crate::CHAR_BYTE_MAP['q' as usize]);
println!("{}", crate::CHAR_BYTE_MAP['r' as usize]);
println!("{}", crate::CHAR_BYTE_MAP['s' as usize]);
println!("{}", crate::CHAR_BYTE_MAP['t' as usize]);
println!("{}", crate::CHAR_BYTE_MAP['u' as usize]);
println!("{}", crate::CHAR_BYTE_MAP['w' as usize]);
println!("{}", crate::CHAR_BYTE_MAP['x' as usize]);
println!("{}", crate::CHAR_BYTE_MAP['y' as usize]);
println!("{}", crate::CHAR_BYTE_MAP['z' as usize]);
*/

pub struct Accounts(pub [u8; 32]);

impl BytesDecode<'_> for Accounts {
    type DItem = [u8; 32];
    fn bytes_decode(bytes: &'_ [u8]) -> Result<Self::DItem, BoxedError> {
        Ok(bytes.try_into().expect("array must fit in a `[u8; 32]`"))
    }
}