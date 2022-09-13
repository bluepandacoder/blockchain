mod p2p;

use crypto_lib::{future::FusedFuture, *};
use p2p::NetworkManager;

#[async_std::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let new_blockchain = Blockchain::default();

    let active_blockchain = Arc::new(Mutex::new(new_blockchain.clone()));
    let mining_block = Arc::new(Mutex::new(new_blockchain.generate_block()));

    let blockchain_topic = libp2p::gossipsub::IdentTopic::new("blockchain");
    let transactions_topic = libp2p::gossipsub::IdentTopic::new("transactions");

    let mut network_manager =
        NetworkManager::start(vec![blockchain_topic.clone(), transactions_topic.clone()]).await?;

    let mut block_miner = BlockMiner::new(mining_block.clone());
    block_miner.start();

    loop {
        select! {
            _ = block_miner => {
                let mut block = mining_block.lock().unwrap();
                println!("{:?} has been mined.", block);
                let mut blockchain = active_blockchain.lock().unwrap();
                println!("{:?}.", blockchain);
                blockchain.blocks.push(block.clone());
                network_manager.swarm.behaviour_mut().gossipsub
                .publish(blockchain_topic.clone(), bincode::serialize(&(blockchain.clone())).unwrap());
                *block = blockchain.generate_block();
            },
            event = network_manager.swarm.select_next_some() => match event {
                SwarmEvent::NewListenAddr { address, .. } => {
                    println!("Listening on {:?}", address);
                }
                SwarmEvent::Behaviour(p2p::OutEvent::Gossipsub(
                    libp2p::gossipsub::GossipsubEvent::Message{
                        propagation_source: _,
                        message_id: _,
                        message
                    }
                )) => {
                    let topic = &message.topic;
                    println!("Message on {:?}.", topic);
                    let mining_block_copy = mining_block.clone();
                    let active_blockchain_copy = active_blockchain.clone();
                    if topic == &blockchain_topic.hash() {
                        thread::spawn(move || handle_blockchain(active_blockchain_copy, mining_block_copy, &message.data));
                    }
                    else if topic == &transactions_topic.hash() {
                        thread::spawn(move || handle_transaction(active_blockchain_copy, mining_block_copy, &message.data));
                    }
                }
                SwarmEvent::Behaviour(p2p::OutEvent::Mdns(
                    MdnsEvent::Discovered(list)
                )) => {
                    println!("NEW PEER DISCOVERED");
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
                    println!("PEER EXPIRED");
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
}

struct BlockMiner {
    block: Arc<Mutex<Block>>,
}

impl BlockMiner {
    pub fn new(block: Arc<Mutex<Block>>) -> Self {
        Self { block }
    }
    pub fn start(&self) {
        let block_copy = self.block.clone();
        thread::spawn(|| mining::mine_block(block_copy));
    }
}

impl Future for BlockMiner {
    type Output = ();

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut task::Context<'_>,
    ) -> task::Poll<Self::Output> {
        let block = self.block.lock().unwrap();
        if block.mined() {
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

fn handle_blockchain(
    active_blockchain: Arc<Mutex<Blockchain>>,
    mining_block: Arc<Mutex<Block>>,
    data: &[u8],
) {
    let mut mining_block = mining_block.lock().unwrap();
    let mut active_blockchain = active_blockchain.lock().unwrap();

    println!("Processing new blockchain.");

    match bincode::deserialize::<Blockchain>(data) {
        Ok(candidate_blockchain) => {
            if candidate_blockchain.blocks.len() > active_blockchain.blocks.len() {
                if candidate_blockchain.valid() {
                    *active_blockchain = candidate_blockchain;
                    *mining_block = active_blockchain.generate_block();
                    println!("{:?} accepted and replaced.", active_blockchain);
                } else {
                    println!("Blockchain discarded. Didn't pass validation tests.")
                }
            } else {
                println!("Blockchain discarded. Shorter than our own.");
            }
        }
        Err(_) => {
            println!("Invalid blockchain format received.");
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
    if mining_block.mined() {
        return;
    }

    match bincode::deserialize::<Transaction>(data) {
        Ok(transaction) => {
            println!("Processing transaction {:?}", transaction);
            if active_blockchain.verify_transaction(&transaction) {
                mining_block.transactions.push(transaction);
            }
        }
        Err(_) => {
            println!("Received invalid transaction format");
        }
    };
}
