use std::collections::HashMap;

use crate::{
    hash_leaf, hash_two,
    merkle::{self, DenseIncrementalMerkleTree},
    Hash,
};
use serde::Serialize;
use sha3::Keccak256;

const CONTRACT_ADDRESS: u64 = 123948573;
const DEFAULT_ACCOUNT: u64 = 123;
const DEFAULT_AMOUNT: u64 = 1000;

#[derive(Debug, Clone, Serialize)]
pub struct Note {
    secret: u64,
    topic: u64,
    recipiant: u64,
    merkle_path: merkle::Path,
}

impl Note {
    pub fn commitment(&self) -> Hash {
        let secret_hash = hash_leaf::<Keccak256>(self.secret.to_be_bytes().to_vec());
        hash_two::<Keccak256>(secret_hash.clone(), secret_hash)
    }

    pub fn nullifier(&self) -> Hash {
        let secret_hash = hash_leaf::<Keccak256>(self.secret.to_be_bytes().to_vec());
        let topic_hash = hash_leaf::<Keccak256>(self.topic.to_be_bytes().to_vec());
        hash_two::<Keccak256>(secret_hash.clone(), topic_hash)
    }

    pub fn recipiant(&self) -> u64 {
        self.recipiant
    }
}

pub struct AnonymityPool {
    tree: DenseIncrementalMerkleTree<Keccak256>,
    nullifiers: HashMap<Hash, bool>,
    balances: HashMap<u64, u64>,
    root_history: Vec<Hash>,
}

impl Default for AnonymityPool {
    fn default() -> Self {
        Self::new()
    }
}

impl AnonymityPool {
    pub fn new() -> Self {
        let tree = DenseIncrementalMerkleTree::<Keccak256>::new();
        let mut balances = HashMap::new();
        balances.insert(DEFAULT_ACCOUNT, DEFAULT_AMOUNT * 10);
        Self {
            tree,
            nullifiers: HashMap::new(),
            balances,
            root_history: Vec::new(),
        }
    }

    pub fn account() -> u64 {
        DEFAULT_ACCOUNT
    }

    pub fn amount() -> u64 {
        DEFAULT_AMOUNT
    }

    pub fn nullifiers(&self) -> HashMap<Hash, bool> {
        self.nullifiers.clone()
    }

    pub fn balances(&self) -> HashMap<u64, u64> {
        self.balances.clone()
    }

    pub fn get_balance(&self, account: u64) -> u64 {
        *self.balances.get(&account).unwrap_or(&0)
    }

    pub fn deposit(&mut self, sender: u64, secret: u64, topic: u64, recipiant: u64) -> Note {
        let secret_hash = hash_leaf::<Keccak256>(secret.to_be_bytes().to_vec());
        let topic_hash = hash_leaf::<Keccak256>(topic.to_be_bytes().to_vec());

        let nullifier = hash_two::<Keccak256>(secret_hash.clone(), topic_hash);
        let commitment = hash_two::<Keccak256>(secret_hash.clone(), secret_hash);

        assert!(*self.balances.get(&sender).unwrap_or(&0) > DEFAULT_AMOUNT);

        let index = self.tree.insert_leaf(commitment);
        self.nullifiers.insert(nullifier, false);

        let root = self.tree.root().unwrap();
        self.root_history.push(root);

        // Deposit amount to contract
        self.balances
            .entry(sender)
            .and_modify(|x| *x -= DEFAULT_AMOUNT);
        self.balances
            .entry(CONTRACT_ADDRESS)
            .and_modify(|x| *x += DEFAULT_AMOUNT);

        let merkle_path = self.tree.find_path(index);

        Note {
            secret,
            topic,
            recipiant,
            merkle_path,
        }
    }

    pub fn withdraw(&mut self, note: Note) -> bool {
        let secret_hash = hash_leaf::<Keccak256>(note.secret.to_be_bytes().to_vec());
        let topic_hash = hash_leaf::<Keccak256>(note.topic.to_be_bytes().to_vec());
        let nullifier = hash_two::<Keccak256>(secret_hash.clone(), topic_hash);

        if let Some(&is_nullifier_taken) = self.nullifiers.get(&nullifier) {
            if is_nullifier_taken {
                return false;
            }
        }
        let root = note.merkle_path.construct_root();
        if !self.root_history.contains(&root) {
            return false;
        }

        self.balances
            .entry(CONTRACT_ADDRESS)
            .and_modify(|x| *x -= DEFAULT_AMOUNT);
        self.balances
            .entry(note.recipiant)
            .and_modify(|x| *x += DEFAULT_AMOUNT);

        self.nullifiers.insert(nullifier, true);

        true
    }
}
