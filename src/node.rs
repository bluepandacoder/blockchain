use super::*;
use std::thread::{JoinHandle, Thread};

pub struct Node {
    pub active_blockchain: Arc<Mutex<Blockchain>>,
    pub active_block: Arc<Mutex<Block>>,
    pub mining_handle: JoinHandle<()>,
}

impl Node {
    pub async fn start(
        blockchain_topic: gossipsub::IdentTopic,
        transaction_topic: gossipsub::IdentTopic,
        rew_pkey: PublicKey,
    ) -> Result<Self, Box<dyn Error>> {
        let mut network_manager =
            NetworkManager::start(vec![blockchain_topic.clone(), transaction_topic.clone()])
                .await?;

        let active_blockchain = Arc::new(Mutex::new(Blockchain::default()));
        let active_block = Arc::new(Mutex::new(
            active_blockchain.lock().unwrap().generate_block(rew_pkey),
        ));

        let mut block_miner =
            mining::BlockMiner::new(active_block.clone(), active_blockchain.clone());
        block_miner.start();

        let active_block_copy = active_block.clone();
        let active_blockchain_copy = active_blockchain.clone();

        let mining_handle = thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let active_block = active_block_copy;
                let active_blockchain = active_blockchain_copy;
                loop {
                    let active_block_copy = active_block.clone();
                    let active_blockchain_copy = active_blockchain.clone();
                    select! {
                        _ = block_miner => {
                            let mut block = active_block.lock().unwrap();
                            let mut blockchain = active_blockchain.lock().unwrap();
                            match blockchain.add_block(block.clone()) {
                                Ok(()) => {
//                                    println!("Sending to peers: {:?}.", blockchain);

                                    network_manager.swarm.behaviour_mut().gossipsub
                                    .publish(blockchain_topic.clone(), bincode::serialize(&blockchain.blocks).unwrap());
                                }
                                Err(e) => println!("Error encountered when adding mined block: {:?}", e)
                            }
                            *block = blockchain.generate_block(rew_pkey);
                        },
                        event = network_manager.swarm.select_next_some() => match event {
                            SwarmEvent::NewListenAddr { address, .. } => {
                                //println!("Listening on {:?}", address);
                            }
                            SwarmEvent::Behaviour(p2p::OutEvent::Gossipsub(
                                libp2p::gossipsub::GossipsubEvent::Message{
                                    propagation_source: _,
                                    message_id: _,
                                    message
                                }
                            )) => {
                                let topic = &message.topic;
                                //println!("Message on {:?}.", topic);
                                if topic == &blockchain_topic.hash() {
                                    let rew_pkey = rew_pkey.clone();
                                    thread::spawn(move || handle_blockchain(active_blockchain_copy, active_block_copy, &message.data, rew_pkey));
                                }
                                else if topic == &transaction_topic.hash() {
                                    thread::spawn(move || handle_transaction(active_blockchain_copy, active_block_copy, &message.data));
                                }
                            }
                            SwarmEvent::Behaviour(p2p::OutEvent::Mdns(
                                MdnsEvent::Discovered(list)
                            )) => {
                                //println!("NEW PEER DISCOVERED");
                                for (peer, _) in list {
                                    network_manager.swarm
                                        .behaviour_mut()
                                        .gossipsub
                                        .add_explicit_peer(&peer);
                                }
                            }
                            SwarmEvent::Behaviour(p2p::OutEvent::Mdns(MdnsEvent::Expired(
                                list
                            ))) => {
                                //println!("PEER EXPIRED");
                                for (peer, _) in list {
                                    if !network_manager.swarm.behaviour_mut().mdns.has_node(&peer) {
                                        network_manager.swarm
                                            .behaviour_mut()
                                            .gossipsub
                                            .remove_explicit_peer(&peer);
                                    }
                                }
                            },
                            _ => {}
                        }
                    }
                }
            });
        });

        Ok(Self {
            active_blockchain,
            active_block,
            mining_handle,
        })
    }
}

fn handle_blockchain(
    active_blockchain: Arc<Mutex<Blockchain>>,
    mining_block: Arc<Mutex<Block>>,
    data: &[u8],
    pub_key: PublicKey,
) {
    let mut mining_block = mining_block.lock().unwrap();
    let mut active_blockchain = active_blockchain.lock().unwrap();

    //println!("Processing new blockchain.");

    if let Ok(blocks) = bincode::deserialize::<Vec<Block>>(data) {
        match Blockchain::construct(blocks) {
            Ok(new_blockchain) => {
                if new_blockchain.weight > active_blockchain.weight {
                    *active_blockchain = new_blockchain;
                    *mining_block = active_blockchain.generate_block(pub_key);
                    //println!("{:?} accepted and replaced.", active_blockchain);
                } else {
                    //println!("Discarded blockchain, lighter than our own.");
                }
            }
            Err(e) => {
                //println!("Couldn't construct blockchain encountered: {:?}", e)
            }
        }
    }
}

fn handle_transaction(
    active_blockchain: Arc<Mutex<Blockchain>>,
    mining_block: Arc<Mutex<Block>>,
    data: &[u8],
) {
    let active_blockchain = active_blockchain.lock().unwrap();
    let mut mining_block = mining_block.lock().unwrap();
    let difficulty = active_blockchain.difficulty(&mining_block);
    if mining::mined(&mining_block, difficulty) {
        return;
    }

    if let Ok(transaction) = bincode::deserialize::<Transaction>(data) {
        //println!("Processing {:?}", transaction);
        if transaction.valid() {
            let user_spendings = &mining_block.spendings(&transaction.data.from);
            let user_balance = active_blockchain
                .balances
                .get(transaction.data.from.as_bytes())
                .unwrap_or(&0);
            if user_balance >= &(user_spendings + transaction.data.amount) {
                mining_block.transactions.push(transaction);
                //println!("Transaction successfully added to mining block");
                //println!(
                //    "Block now has {:?} transactions",
                //    mining_block.transactions.len()
                //);
            } else {
                println!("Not enough coins to make transaction");
            }
        } else {
            println!("Transaction has invalid signature");
        }
    };
}
