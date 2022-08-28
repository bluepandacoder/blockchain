use super::*;

#[derive(Clone, Serialize, Deserialize)]
pub struct Block {
    pub mined: bool
}

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct BlockChain {
    blocks: Vec<Block>
}

impl BlockChain {
    pub fn push(&mut self, block: Block) {
        self.blocks.push(block);
    }
}