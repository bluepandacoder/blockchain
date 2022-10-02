mod p2p;

use crypto_lib::{future::FusedFuture, *};
use p2p::NetworkManager;

use rand::rngs::OsRng;

#[async_std::main]
async fn main() -> Result<(), Box<dyn Error>> {

    let blockchain_topic = gossipsub::IdentTopic::new("blockchain");
    let transactions_topic = gossipsub::IdentTopic::new("transactions");

    let mut network_manager =
        NetworkManager::start(vec![blockchain_topic.clone(), transactions_topic.clone()]).await?;

    let key_pair = Keypair::generate(&mut OsRng {});
    let mut stdin = io::BufReader::new(io::stdin()).lines().fuse();

    println!("PUBLIC KEY: {}", hex::encode(key_pair.public));

    let new_blockchain = Blockchain::default();

    let active_blockchain = Arc::new(Mutex::new(new_blockchain.clone()));
    let mining_block = Arc::new(Mutex::new(new_blockchain.generate_block(key_pair.public)));

    let mut block_miner = BlockMiner::new(mining_block.clone(), active_blockchain.clone());
    block_miner.start();

    loop {
        let mining_block_copy = mining_block.clone();
        let active_blockchain_copy = active_blockchain.clone();
        select! {
            line = stdin.select_next_some() => {
                match line {
                    Ok(text) => {
                        let mut sp = text.trim().split(' ');
                        if let Some(to_user_hex) = sp.next() {
                            match hex::decode(to_user_hex) {
                                Ok(to_user) => {
                                    if let Ok(to_user) = PublicKey::from_bytes(&to_user) {
                                        if let Some(amount_str) = sp.next() {
                                            match amount_str.parse::<u64>() {
                                                Ok(amount) => {
                                                    let new_transaction = bincode::serialize(&Transaction::new(to_user, amount, &key_pair))?;
                                                    network_manager.swarm.behaviour_mut().gossipsub
                                                    .publish(transactions_topic.clone(), new_transaction.clone());

                                                    thread::spawn(move || handle_transaction(active_blockchain_copy, mining_block_copy, &new_transaction));
                                                }
                                                Err(e) => println!("{:?}", e)
                                            }
                                        }
                                        else {
                                            println!("No amount for transaction specified.");
                                        }
                                    }
                                }
                                Err(e) => println!("{:?}", e)
                            }
                        } 
                    }
                    Err(e) => println!("{:?}", e)
                }
            }
            _ = block_miner => {
                let mut block = mining_block.lock().unwrap();
                println!("{:?} has been mined.", block);

                let mut blockchain = active_blockchain.lock().unwrap();
                blockchain.add_block(block.clone()).unwrap();
                println!("{:?}.", blockchain);

                network_manager.swarm.behaviour_mut().gossipsub
                .publish(blockchain_topic.clone(), bincode::serialize(&blockchain.blocks).unwrap());
                *block = blockchain.generate_block(key_pair.public);
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
                    if topic == &blockchain_topic.hash() {
                        thread::spawn(move || handle_blockchain(active_blockchain_copy, mining_block_copy, &message.data, key_pair.public));
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
    blockchain: Arc<Mutex<Blockchain>>
}

impl BlockMiner {
    pub fn new(block: Arc<Mutex<Block>>,blockchain: Arc<Mutex<Blockchain>>) -> Self {
        Self { block, blockchain }
    }
    pub fn start(&self) {
        let block_copy = self.block.clone();
        let blockchain_copy = self.blockchain.clone();
        thread::spawn(|| mining::mine_block_multithreaded(block_copy, blockchain_copy));
    }
}

impl Future for BlockMiner {
    type Output = ();

    fn poll(self: std::pin::Pin<&mut Self>, _: &mut task::Context<'_>) -> task::Poll<Self::Output> {
        let block = self.block.lock().unwrap();
        let blockchain = self.blockchain.lock().unwrap();

        let difficulty = blockchain.difficulty(&block);

        if mining::mined(&block, difficulty) {
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
    pub_key: PublicKey
) {
    let mut mining_block = mining_block.lock().unwrap();
    let mut active_blockchain = active_blockchain.lock().unwrap();

    println!("Processing new blockchain.");

    match bincode::deserialize::<Vec<Block>>(data) {
        Ok(blocks) => {
            match Blockchain::construct(blocks) {
                Ok(new_blockchain) => {
                    if new_blockchain.weight > active_blockchain.weight {
                        *active_blockchain = new_blockchain;
                        *mining_block = active_blockchain.generate_block(pub_key);
                        println!("{:?} accepted and replaced.", active_blockchain);
                    }
                    else {
                        println!("Discarded blockchain, lighter than our own.");
                    }
                }
                Err(e) => println!("Couldn't construct blockchain encountered: {:?}", e),
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
    let difficulty = active_blockchain.difficulty(&mining_block);
    if mining::mined(&mining_block, difficulty) {
        return;
    }

    match bincode::deserialize::<Transaction>(data) {
        Ok(transaction) => {
            println!("Processing {:?}", transaction);
            if transaction.valid() {
                let user_spendings = &mining_block.spendings(&transaction.data.from);
                let user_balance = active_blockchain.balances.get(transaction.data.from.as_bytes()).unwrap_or(&0);
                if user_balance >= &(user_spendings+transaction.data.amount) {
                    mining_block.transactions.push(transaction);
                    println!("Transaction successfully added to mining block")
                }
                else {
                    println!("Not enough coins to make transaction");
                }
            }
            else {
                println!("Transaction has invalid signature");
            }
        }
        Err(_) => {
            println!("Received invalid transaction format");
        }
    };
}
