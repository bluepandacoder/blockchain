use super::*;
use std::thread;

// pub fn mine_block_multithreaded(block: Arc<Mutex<Block>>) {
//     thread::scope(|s| {
//         for _ in 0..10 {
//             let block_copy = block.clone();
//             s.spawn(move || {
//                 let block = block_copy.lock().unwrap();
//                 let mut mining_block = block.clone();
//                 drop(block);
//                 loop {
//                     for i in 0..100000 {
//                         if mining_block.mined() {
//                             break;
//                         }
//                         mining_block.nonce += 1;
//                     }
//                     let mut block = block_copy.lock().unwrap();
//                     if mining_block.mined() {
//                         *block = mining_block.clone();
//                     }
//                     if block.mined() {
//                         break;
//                     }
//                 }
//                 let block = block_copy.lock();
//                 println!("{:?}", block);
//             });
//         }
//     });
// }


pub fn mine_block(block: Arc<Mutex<Block>>, blockchain: Arc<Mutex<Blockchain>>) {
    loop {
        let mut mining_block = (block.lock().unwrap()).clone();
        let difficulty = (blockchain.lock().unwrap()).difficulty(&mining_block);
        if mined(&mining_block, difficulty) {
            thread::sleep(std::time::Duration::from_millis(100));
        }
        else {
            mining_block.nonce = rand::random::<u64>() / 2;
            mining_block.timestamp = now();
            let difficulty = (blockchain.lock().unwrap()).difficulty(&mining_block);

            for _ in 0..100_000 {
                if mined(&mining_block, difficulty) {
                    *block.lock().unwrap() = mining_block;
                    break;
                }
                mining_block.nonce += 1;
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
    block.hash()%(2<< difficulty) == 0.into()
}
