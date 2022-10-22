use super::*;
use crate::mining::calculate_dif_offset;
use crypto_hash::{digest, Algorithm};
use ed25519_dalek::PUBLIC_KEY_LENGTH;
use std::collections::HashMap;

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct Block {
    pub transactions: Vec<Transaction>,
    pub nonce: u64,
    pub timestamp: u64,
    mined_by: PublicKey,
    prev_hash: Hash,
}

impl core::fmt::Debug for Block {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Block with {} transactions, mined by {}",
            self.transactions.len(),
            hex::encode(self.mined_by)
        )
    }
}

impl Block {
    pub fn hash(&self) -> Hash {
        let block_binary = bincode::serialize(&self).unwrap();
        digest(Algorithm::SHA256, &block_binary)[0..32].into()
    }
    pub fn spendings(&self, user: &PublicKey) -> u64 {
        self.transactions
            .iter()
            .filter(|t| &t.data.from == user)
            .map(|t| t.data.amount)
            .sum()
    }
}

#[derive(Debug)]
pub enum BlockValidationError {
    PrevHashMismatch,
    BlockNotMinedCorrectly,
    ExcessiveTransactionAmount,
    InvalidTransactionSignature,
    InvalidTimestamp,
}

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct Blockchain {
    pub blocks: Vec<Block>,
    pub balances: HashMap<[u8; PUBLIC_KEY_LENGTH], u64>,
    pub cur_dif: u32,
    pub weight: u32,
}

impl core::fmt::Debug for Blockchain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Blockchain with {} blocks, {:?} balances, active difficulty {} and weight of {}",
            self.blocks.len(),
            self.balances.values(),
            self.cur_dif,
            self.weight
        )
    }
}

impl Blockchain {
    pub fn construct(blocks: Vec<Block>) -> Result<Self, BlockValidationError> {
        let mut result = Self::default();
        for block in blocks {
            result.add_block(block)?;
        }
        Ok(result)
    }

    pub fn difficulty(&self, block: &Block) -> u32 {
        if let Some(lblock) = self.blocks.last() {
            let dif_offset = calculate_dif_offset(block.timestamp - lblock.timestamp);
            let answer = (self.cur_dif as i32 + dif_offset).max(0);
            answer as u32
        } else {
            0
        }
    }

    pub fn generate_block(&self, mined_by: PublicKey) -> Block {
        Block {
            nonce: 0,
            transactions: vec![],
            prev_hash: if let Some(lblock) = self.blocks.last() {
                lblock.hash()
            } else {
                0.into()
            },
            mined_by,
            timestamp: now(),
        }
    }
    pub fn add_block(&mut self, block: Block) -> Result<(), BlockValidationError> {
        if let Some(lblock) = self.blocks.last() {
            if lblock.hash() != block.prev_hash {
                return Err(BlockValidationError::PrevHashMismatch);
            }
            if block.timestamp > now() || lblock.timestamp > block.timestamp {
                return Err(BlockValidationError::InvalidTimestamp);
            }
        }

        let new_difficulty = self.difficulty(&block);

        if !mining::mined(&block, new_difficulty) {
            return Err(BlockValidationError::BlockNotMinedCorrectly);
        }

        // check transactions
        for transaction in &block.transactions {
            let pub_kb = transaction.data.from.as_bytes();

            if !transaction.valid() {
                return Err(BlockValidationError::InvalidTransactionSignature);
            }
            if &transaction.data.amount > self.balances.get(pub_kb).unwrap_or(&0) {
                return Err(BlockValidationError::ExcessiveTransactionAmount);
            }
            match self.balances.get_mut(pub_kb) {
                Some(balance) => {
                    *balance -= transaction.data.amount;
                }
                None => {
                    return Err(BlockValidationError::ExcessiveTransactionAmount);
                }
            }
        }

        for transaction in &block.transactions {
            if let Some(balance) = self.balances.get_mut(transaction.data.from.as_bytes()) {
                *balance += transaction.data.amount;
            } else {
                self.balances
                    .insert(*transaction.data.to.as_bytes(), transaction.data.amount);
            }
        }

        self.weight += new_difficulty;
        self.cur_dif = new_difficulty;

        let cur_bal = *self.balances.get(block.mined_by.as_bytes()).unwrap_or(&0);
        self.balances
            .insert(*block.mined_by.as_bytes(), cur_bal + MINING_REW);

        self.blocks.push(block);

        Ok(())
    }

    pub fn verify_transaction(&self, transaction: &Transaction) -> bool {
        if let Some(balance) = self.balances.get(transaction.data.from.as_bytes()) {
            balance >= &transaction.data.amount
        } else {
            false
        }
    }
}
