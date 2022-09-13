use super::*;
use crypto_hash::{digest, Algorithm};
use std::collections::HashMap;

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct Block {
    pub transactions: Vec<Transaction>,
    pub nonce: u64,
    prev_hash: Hash,
}

impl Block {
    pub fn mined(&self) -> bool {
        self.hash() % MINING_REQ == 0.into()
    }
    pub fn hash(&self) -> Hash {
        let block_binary = bincode::serialize(&self).unwrap();
        digest(Algorithm::SHA256, &block_binary)[0..32].into()
    }
    pub fn valid(&self) -> bool {
        self.mined()
    }
}

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct Blockchain {
    pub blocks: Vec<Block>,
    pub balances: HashMap<Hash, u64>,
}

impl core::fmt::Debug for Blockchain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Blockchain with {} blocks and {} balances", self.blocks.len(), self.balances.len())
    }
}

impl Blockchain {
    pub fn valid(&self) -> bool {
        for block in &self.blocks {
            if !block.valid() {
                return false;
            }
        }
        for i in 1..self.blocks.len() {
            if self.blocks[i - 1].hash() != self.blocks[i].prev_hash {
                return false;
            }
        }
        return true;
    }
    pub fn generate_block(&self) -> Block {
        Block {
            nonce: 0,
            transactions: vec![],
            prev_hash: if let Some(lblock) = self.blocks.last() {
                lblock.hash()
            } else {
                0.into()
            },
        }
    }
    pub fn verify_transaction(&self, transaction: &Transaction) -> bool {
        self.balances[&transaction.from] > transaction.amount
    }
}
