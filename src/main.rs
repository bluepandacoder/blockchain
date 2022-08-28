
mod p2p;


use crypto_lib::{*, future::FusedFuture};
use p2p::NetworkManager;

#[async_std::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mining_block = Arc::new(Mutex::new(Block {mined: false}));
    let active_blockchain = Arc::new(Mutex::new(BlockChain::default()));

    let blockchain_topic = floodsub::Topic::new("blockchain");
    let transactions_topic = floodsub::Topic::new("transactions");

    let mut network_manager = NetworkManager::start(vec![blockchain_topic.clone(), transactions_topic.clone()]).await?;

    let mut stdin = io::BufReader::new(io::stdin()).lines().fuse();

    let mut block_miner = BlockMiner::start(mining_block.clone());

    // Kick it off
    loop {
        select! {
            _ = block_miner => {
                println!("block mined");
                let block = mining_block.lock().unwrap();
                let mut blockchain = active_blockchain.lock().unwrap();
                blockchain.push(block.clone());
                network_manager.swarm.behaviour_mut().floodsub
                .publish(blockchain_topic.clone(), bincode::serialize(&(blockchain.clone())).unwrap())
            },
            line = stdin.select_next_some() => network_manager.swarm
                .behaviour_mut()
                .floodsub
                .publish(blockchain_topic.clone(), line.expect("Stdin not to close").as_bytes()),
            event = network_manager.swarm.select_next_some() => match event {
                SwarmEvent::NewListenAddr { address, .. } => {
                    println!("Listening on {:?}", address);
                }
                SwarmEvent::Behaviour(p2p::OutEvent::Floodsub(
                    FloodsubEvent::Message(message)
                )) => {
                    println!(
                        "Received: '{:?}' from {:?}, on topic {:?}",
                        String::from_utf8_lossy(&message.data),
                        message.source,
                        message.topics
                    );
                    let topic = &message.topics[0];
                    let mining_block_copy = mining_block.clone();
                    if topic == &blockchain_topic {
                        let active_blockchain_copy = active_blockchain.clone();
                        thread::spawn(move || handle_blockchain(active_blockchain_copy, mining_block_copy, message.data));
                        println!("message on blockchain topic");
                    }
                    else if topic == &transactions_topic {
                        thread::spawn(move || handle_transaction(mining_block_copy, message.data));
                        println!("message on transactions topic");
                    }
                }
                SwarmEvent::Behaviour(p2p::OutEvent::Mdns(
                    MdnsEvent::Discovered(list)
                )) => {
                    for (peer, _) in list {
                        network_manager.swarm
                            .behaviour_mut()
                            .floodsub
                            .add_node_to_partial_view(peer);
                    }
                }
                SwarmEvent::Behaviour(p2p::OutEvent::Mdns(MdnsEvent::Expired(
                    list
                ))) => {
                    for (peer, _) in list {
                        if !network_manager.swarm.behaviour_mut().mdns.has_node(&peer) {
                            network_manager.swarm
                                .behaviour_mut()
                                .floodsub
                                .remove_node_from_partial_view(&peer);
                        }
                    }
                },
                _ => {}
            }
        }
    }
}

struct BlockMiner {
    block: Arc<Mutex<Block>>
}

impl BlockMiner {
    pub fn start(block: Arc<Mutex<Block>>) -> Self {
        mining::mine_block_multithreaded(block.clone());
        Self {
            block
        }
    }
}

impl Future for BlockMiner {
    type Output=();

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut task::Context<'_>) -> task::Poll<Self::Output> {
        let block = self.block.lock().unwrap();
        if block.mined { task::Poll::Ready(()) } else { task::Poll::Pending }
    }
}

impl FusedFuture for BlockMiner {
    fn is_terminated(&self) -> bool {
        false
    }
}


struct App {
    mining_block: Arc<Mutex<Block>>,
    active_blockchain: BlockChain,
    network_manager: NetworkManager,
    blockchain_topic: floodsub::Topic,
    transactions_topic: floodsub::Topic,
}

async fn handle_blockchain(active_blockchain: Arc<Mutex<BlockChain>>, mining_block: Arc<Mutex<Block>>, data: Vec<u8>) {
    println!("runnning once");
}

async fn handle_transaction(mining_block: Arc<Mutex<Block>>, data: Vec<u8>) {

}
