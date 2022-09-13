use super::*;
use std::thread;

pub fn mine_block_multithreaded(block: Arc<Mutex<Block>>) {
    thread::scope(|s| {
        for _ in 0..10 {
            let block_copy = block.clone();
            s.spawn(move || {
                let block = block_copy.lock().unwrap();
                let mut mining_block = block.clone();
                drop(block);
                loop {
                    for i in 0..100000 {
                        if mining_block.mined() {
                            break;
                        }
                        mining_block.nonce += 1;
                    }
                    let mut block = block_copy.lock().unwrap();
                    if mining_block.mined() {
                        *block = mining_block.clone();
                    }
                    if block.mined() {
                        break;
                    }
                }
                let block = block_copy.lock();
                println!("{:?}", block);
            });
        }
    });
}

pub fn mine_block(block: Arc<Mutex<Block>>) {
    loop {
        while !block.lock().unwrap().mined() {
            let mut mining_block = (block.lock().unwrap()).clone();
            mining_block.nonce = rand::random::<u64>() / 2;
            let mut i = 0;
            while i < 1_000_000 && !mining_block.mined() {
                mining_block.nonce += 1;
                i += 1;
            }
            block.lock().unwrap().nonce = mining_block.nonce;
        }
        thread::sleep(std::time::Duration::from_millis(100));
    }
}
