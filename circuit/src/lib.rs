mod merkle;
pub mod pool;

use serde::{Deserialize, Serialize};
use sha3::Digest;
use std::error::Error as StdError;
use std::fmt::{Display, Formatter, Result as FmtResult};

#[cfg(test)]
use rand::Rng;

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Hash([u8; 32]);

impl Hash {
    pub fn from_hex(s: String) -> Hash {
        let mut bytes = [0; 32];
        bytes.copy_from_slice(hex::decode(s).unwrap().as_slice());
        Hash(bytes)
    }

    pub fn to_hex(self) -> String {
        hex::encode(self.0)
    }

    pub fn inner(&self) -> &[u8; 32] {
        &self.0
    }
}

#[cfg(test)]
impl Hash {
    pub fn random<R: Rng>(rng: &mut R) -> Self {
        Hash(rng.gen::<[u8; 32]>())
    }
}

pub fn to_bits(num: &[u8]) -> Vec<bool> {
    let len = num.len() * 8;
    let mut bits = Vec::new();
    for i in 0..len {
        let bit = num[i / 8] & (1 << (i % 8)) != 0;
        bits.push(bit);
    }
    bits
}

pub fn num_to_bits_vec(num: u64) -> Vec<bool> {
    let bits = to_bits(&num.to_le_bytes());

    bits[..u32::BITS as usize].to_vec()
}

fn next_index(i: u64) -> u64 {
    if i % 2 == 1 {
        (i - 1) / 2
    } else {
        i / 2
    }
}

pub fn hash_two<H: Digest>(left: Hash, right: Hash) -> Hash {
    let mut hasher = H::new();
    hasher.update(left.0);
    hasher.update(right.0);
    let hash = hasher.finalize().to_vec();
    let mut bytes: [u8; 32] = [0; 32];
    bytes.copy_from_slice(&hash);
    Hash(bytes)
}

pub fn hash_leaf<H: Digest>(preimage: Vec<u8>) -> Hash {
    let mut hasher = H::new();
    hasher.update(preimage);
    let hash = hasher.finalize().to_vec();
    let mut bytes: [u8; 32] = [0; 32];
    bytes.copy_from_slice(&hash);
    Hash(bytes)
}

#[derive(Debug)]
pub enum Error {
    RootNotFound,
    NodesNotFound,
}

impl StdError for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            Self::RootNotFound => write!(f, "RootNotFound"),
            Self::NodesNotFound => write!(f, "NodesNotFound"),
        }
    }
}
