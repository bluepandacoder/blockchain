
mod p2p;

use std::sync::{Mutex, Arc};

use crypto_lib::*;
use p2p::NetworkManager;

#[async_std::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut app = App::new().await?;

    // Kick it off
    loop {
        app.update().await;
    }
}


struct App {
    mining_block: Arc<Mutex<Block>>,
    active_blockchain: BlockChain,
    network_manager: NetworkManager,
    blockchain_topic: floodsub::Topic,
    transactions_topic: floodsub::Topic,
}

impl App {
    async fn new() -> Result<Self, Box<dyn Error>> {
        let mining_block = Arc::new(Mutex::new(Block {}));
        let active_blockchain = BlockChain {};

        let blockchain_topic = floodsub::Topic::new("blockchain");
        let transactions_topic = floodsub::Topic::new("transactions");

        let network_manager = NetworkManager::start(vec![blockchain_topic.clone(), transactions_topic.clone()]).await?;


        Ok (
            Self {
                mining_block,
                active_blockchain,
                blockchain_topic,
                transactions_topic,
                network_manager,
            }
        )
    }

    async fn update(&mut self) {
        let mut stdin = io::BufReader::new(io::stdin()).lines().fuse();

        select! {
            line = stdin.select_next_some() => self.network_manager.swarm
                .behaviour_mut()
                .floodsub
                .publish(self.blockchain_topic.clone(), line.expect("Stdin not to close").as_bytes()),
            _ = mine_block(self.mining_block.clone()) => {
                
            },
            event = self.network_manager.swarm.select_next_some() => match event {
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
                    if topic == &self.blockchain_topic {
                        self.handle_blockchain(message.data);
                        println!("message on blockchain topic");
                    }
                    else if topic == &self.transactions_topic {
                        self.handle_transaction(message.data);
                        println!("message on transactions topic");
                    }
                }
                SwarmEvent::Behaviour(p2p::OutEvent::Mdns(
                    MdnsEvent::Discovered(list)
                )) => {
                    for (peer, _) in list {
                        self.network_manager.swarm
                            .behaviour_mut()
                            .floodsub
                            .add_node_to_partial_view(peer);
                    }
                }
                SwarmEvent::Behaviour(p2p::OutEvent::Mdns(MdnsEvent::Expired(
                    list
                ))) => {
                    for (peer, _) in list {
                        if !self.network_manager.swarm.behaviour_mut().mdns.has_node(&peer) {
                            self.network_manager.swarm
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

    fn handle_transaction(&mut self, data: Vec<u8>) {

    }

    fn handle_blockchain(&mut self, data: Vec<u8>) {

    }

}

async fn mine_block(block: Arc<Mutex<Block>>) {

}

