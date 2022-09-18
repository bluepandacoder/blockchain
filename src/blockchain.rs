use super::*;
use crypto_hash::{digest, Algorithm};
use ed25519_dalek::PUBLIC_KEY_LENGTH;
use std::collections::HashMap;

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct Block {
    pub transactions: Vec<Transaction>,
    pub nonce: u64,
    mined_by: PublicKey,
    prev_hash: Hash,
}

impl core::fmt::Debug for Block {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Block with {} transactions, mined by {}", self.transactions.len(), hex::encode(self.mined_by))
    }
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
    pub fn spendings(&self, user: &PublicKey) -> u64 {
        self.transactions.iter().
            filter(|t| &t.data.from==user).
            map(|t| t.data.amount).sum()
    }
}

#[derive(Debug)]
pub enum BlockValidationError {
    PrevHashMismatch,
    InsufficientDifficulty,
    ExcessiveTransactionAmount,
    InvalidTransactionSignature
}

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct Blockchain {
    pub blocks: Vec<Block>,
    pub balances: HashMap<[u8; PUBLIC_KEY_LENGTH], u64>,
}

impl core::fmt::Debug for Blockchain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Blockchain with {} blocks and {:?} balances", self.blocks.len(), self.balances.values())
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

    pub fn generate_block(&self, mined_by: PublicKey) -> Block {
        Block {
            nonce: 0,
            transactions: vec![],
            prev_hash: if let Some(lblock) = self.blocks.last() {
                lblock.hash()
            } else {
                0.into()
            },
            mined_by
        }
    }

    pub fn add_block(&mut self, block: Block) -> Result<(), BlockValidationError> {

        if block.hash()%MINING_REQ != 0.into() {
            return Err(BlockValidationError::InsufficientDifficulty);
        }
        if let Some(lblock) = self.blocks.last() {
            if lblock.hash() != block.prev_hash {
                return Err(BlockValidationError::PrevHashMismatch);
            }
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
            }
            else {
                self.balances.insert(*transaction.data.to.as_bytes(), transaction.data.amount);
            }
        }

        let cur_bal = *self.balances.get(block.mined_by.as_bytes()).unwrap_or(&0);
        self.balances.insert(*block.mined_by.as_bytes(), cur_bal+MINING_REW);

        self.blocks.push(block);

        Ok(())   
    }

    pub fn verify_transaction(&self, transaction: &Transaction) -> bool {
        if let Some(balance) = self.balances.get(transaction.data.from.as_bytes()) {
            balance >= &transaction.data.amount
        }
        else {
            false
        }
    }
}
