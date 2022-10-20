use super::*;
use std::{thread, time::Duration};
use crate::future::FusedFuture;

pub struct BlockMiner {
    block: Arc<Mutex<Block>>,
    blockchain: Arc<Mutex<Blockchain>>,
}

impl BlockMiner {
    pub fn new(block: Arc<Mutex<Block>>, blockchain: Arc<Mutex<Blockchain>>) -> Self {
        Self { block, blockchain }
    }
    pub fn start(&self) {
        let block_copy = self.block.clone();
        let blockchain_copy = self.blockchain.clone();
        thread::spawn(|| mine_block(block_copy, blockchain_copy));
    }
}

impl Future for BlockMiner {
    type Output = ();

    fn poll(self: std::pin::Pin<&mut Self>, _: &mut task::Context<'_>) -> task::Poll<Self::Output> {
        let block = self.block.lock().unwrap();
        let blockchain = self.blockchain.lock().unwrap();

        let difficulty = blockchain.difficulty(&block);

        if mined(&block, difficulty) {
            task::Poll::Ready(())
        } else {
            task::Poll::Pending
        }
    }
}

impl FusedFuture for BlockMiner {
    fn is_terminated(&self) -> bool {
        false
    }
}

pub fn mine_block_multithreaded(block: Arc<Mutex<Block>>, blockchain: Arc<Mutex<Blockchain>>) {
    let block_id = Arc::new(Mutex::new(0usize));

    thread::scope(|s| {
        for _ in 0..8 {
            let block_copy = block.clone();
            let blockchain_copy = blockchain.clone();
            let block_id_copy = block_id.clone();
            s.spawn(move || {
                let block = block_copy;
                let blockchain = blockchain_copy;
                let block_id = block_id_copy;

                loop {
                    let mining_block_id = block_id.lock().unwrap().clone();
                    let mut mining_block = (block.lock().unwrap()).clone();
                    let difficulty = (blockchain.lock().unwrap()).difficulty(&mining_block);

                    if mined(&mining_block, difficulty) {
                        thread::sleep(std::time::Duration::from_millis(100))
                    } else {
                        mining_block.nonce = rand::random::<u64>() / 2;
                        mining_block.timestamp = now();
                        let difficulty = (blockchain.lock().unwrap()).difficulty(&mining_block);

                        for _ in 0..100_000 {
                            if mined(&mining_block, difficulty) {
                                let mut block_id = block_id.lock().unwrap();
                                if mining_block_id == *block_id {
                                    *block_id += 1;
                                    *block.lock().unwrap() = mining_block;
                                }
                                break;
                            }
                            mining_block.nonce += 1;
                        }
                    }
                }
            });
        }
    });
}

pub fn mine_block(block: Arc<Mutex<Block>>, blockchain: Arc<Mutex<Blockchain>>) {
    loop {
        let mut mining_block = (block.lock().unwrap()).clone();
        let difficulty = (blockchain.lock().unwrap()).difficulty(&mining_block);
        if mined(&mining_block, difficulty) {
            thread::sleep(std::time::Duration::from_millis(100));
        } else {
            mining_block.nonce = rand::random::<u64>() / 2;
            mining_block.timestamp = now();
            let difficulty = (blockchain.lock().unwrap()).difficulty(&mining_block);

            for _ in 0..100 {
                if mined(&mining_block, difficulty) {
                    *block.lock().unwrap() = mining_block;
                    break;
                }
                mining_block.nonce += 1;

                // to decrease cpu consumption
                thread::sleep(Duration::from_millis(10));
            }
        }
    }
}

pub fn calculate_dif_offset(time_dif: u64) -> i32 {
    let time_dif = time_dif as f64;

    let mut answer = 1;
    let mut time_offset = TIME_BASE as f64 / 2.;
    let next_offset = time_offset / 2.;

    while time_offset < time_dif && answer > -256 {
        time_offset += next_offset;
        answer -= 1;
    }

    answer
}

pub fn mined(block: &Block, difficulty: u32) -> bool {
    block.hash() % (2 << difficulty) == 0.into()
}
