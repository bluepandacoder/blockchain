use std::hash::Hash;
use crypto_lib::{future::FusedFuture, *};

use rand::rngs::OsRng;

#[async_std::main]
async fn main() -> Result<(), Box<dyn Error>> {

    let blockchain_topic = gossipsub::IdentTopic::new("blockchain");
    let transactions_topic = gossipsub::IdentTopic::new("transactions");


    let key_pair = Keypair::generate(&mut OsRng {});

    let mut node = Node::start(blockchain_topic, transactions_topic,
                               key_pair.public).await?;

    println!("PUBLIC KEY: {}", hex::encode(key_pair.public));

    node.mining_handle.join();

    Ok(())
}



