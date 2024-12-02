use crate::{hash_two, next_index, num_to_bits_vec, Error as MerkleError, Hash};
use serde::Serialize;
use sha3::{Digest, Keccak256};
use std::{collections::HashMap, marker::PhantomData};

pub fn to_bits(num: &[u8]) -> Vec<bool> {
    let len = num.len() * 8;
    let mut bits = Vec::new();
    for i in 0..len {
        let bit = num[i / 8] & (1 << (i % 8)) != 0;
        bits.push(bit);
    }
    bits
}

pub fn num_to_bits(num: u64) -> Vec<bool> {
    to_bits(num.to_be_bytes().as_ref())
}

#[derive(Debug, Clone, Serialize)]
pub struct Path {
    index: u64,
    neighbours: Vec<Hash>,
    leaf: Hash,
}

impl Path {
    pub fn default() -> Self {
        Self {
            index: 0,
            neighbours: Vec::new(),
            leaf: Hash::default(),
        }
    }

    pub fn verify_against(&self, root: Hash) -> bool {
        let sides = num_to_bits(self.index);
        let mut next = self.leaf.clone();
        for (n, left) in self
            .neighbours
            .iter()
            .zip(sides[..self.neighbours.len()].as_ref())
        {
            let new_next = if *left {
                hash_two::<Keccak256>(n.clone(), next)
            } else {
                hash_two::<Keccak256>(next, n.clone())
            };
            next = new_next;
        }
        root == next
    }
}

#[derive(Clone, Debug)]
pub struct DenseIncrementalMerkleTree<H>
where
    H: Digest,
{
    nodes: HashMap<(u32, u64), Hash>,
    default: HashMap<(u32, u64), Hash>,
    index: u64,
    _h: PhantomData<H>,
}

impl<H> DenseIncrementalMerkleTree<H>
where
    H: Digest,
{
    pub fn new() -> Self {
        let mut default: HashMap<(u32, u64), Hash> = HashMap::new();
        default.insert((0, 0), Hash::default());
        for i in 0..Self::num_levels() {
            let h = hash_two::<H>(default[&(i, 0u64)].clone(), default[&(i, 0u64)].clone());
            default.insert(((i + 1), 0), h);
        }

        Self {
            nodes: default.clone(),
            default,
            index: 0,
            _h: PhantomData,
        }
    }

    pub fn num_levels() -> u32 {
        u64::BITS
    }

    pub fn root(&self) -> Result<Hash, MerkleError> {
        self.nodes
            .get(&(Self::num_levels(), 0))
            .cloned()
            .ok_or(MerkleError::RootNotFound)
    }

    pub fn insert_leaf(&mut self, leaf: Hash) {
        let max_size = 2u64.pow(Self::num_levels()) - 1;
        let index = self.index;
        assert!(index + 1 < max_size);
        let bits = num_to_bits_vec(index);

        self.nodes.insert((0, index), leaf.clone());

        let mut curr_index = index;
        let mut curr_node = leaf;
        for i in 0..Self::num_levels() {
            let (left, right) = if bits[i as usize] {
                let n_key = (i, curr_index - 1);
                let n = self.nodes.get(&n_key).unwrap_or(&self.default[&(i, 0)]);
                (n.clone(), curr_node)
            } else {
                let n_key = (i, curr_index + 1);
                let n = self.nodes.get(&n_key).unwrap_or(&self.default[&(i, 0)]);
                (curr_node, n.clone())
            };

            let h = hash_two::<H>(left, right);
            curr_node = h;
            curr_index = next_index(curr_index);

            self.nodes.insert((i + 1, curr_index), curr_node.clone());
        }

        self.index += 1;
    }

    #[cfg(test)]
    pub fn insert_batch(&mut self, leaves: Vec<Hash>) {
        for leaf in leaves {
            self.insert_leaf(leaf);
        }
    }
}

#[cfg(test)]
mod test {
    use super::{DenseIncrementalMerkleTree, Hash};
    use sha3::Keccak256;

    #[test]
    fn should_build_incremental_tree() {
        // Testing build_tree and find_path functions with arity 2
        let leaves = vec![
            Hash::default(),
            Hash::default(),
            Hash::default(),
            Hash::default(),
            Hash::default(),
            Hash::default(),
            Hash::default(),
            Hash::default(),
            Hash::default(),
            Hash::default(),
            Hash::default(),
            Hash::default(),
            Hash::default(),
            Hash::default(),
            Hash::default(),
            Hash::default(),
            Hash::default(),
            Hash::default(),
            Hash::default(),
            Hash::default(),
        ];
        let mut merkle = DenseIncrementalMerkleTree::<Keccak256>::new();
        merkle.insert_batch(leaves);
        let root = merkle.root().unwrap();

        assert_eq!(
            root.to_hex(),
            "27ae5ba08d7291c96c8cbddcc148bf48a6d68c7974b94356f53754ef6171d757".to_string()
        );
    }
}
