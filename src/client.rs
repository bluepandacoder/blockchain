use super::*;
use rand::rngs::OsRng;
use std::collections::VecDeque;

pub struct Client {
    pub key_pair: Keypair,
    transactions: Arc<Mutex<VecDeque<Transaction>>>,
}

impl Client {
    pub async fn start(transaction_topic: gossipsub::IdentTopic) -> Result<Self, Box<dyn Error>> {
        let mut network_manager = NetworkManager::start(vec![transaction_topic.clone()]).await?;

        let transactions: Arc<Mutex<VecDeque<Transaction>>> = Default::default();
        let transactions_copy = transactions.clone();

        rayon::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let mut transaction_stream = TransactionStream {
                    transactions: transactions_copy,
                };
                loop {
                    select! {
                       transaction = transaction_stream => {
                           network_manager.swarm.behaviour_mut().gossipsub.publish(transaction_topic.clone(), bincode::serialize(&transaction).unwrap());
                       },
                       event = network_manager.swarm.select_next_some() => match event {
                           SwarmEvent::NewListenAddr { address, .. } => {
                               //println!("Listening on {:?}", address);
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
            key_pair: Keypair::generate(&mut OsRng {}),
            transactions,
        })
    }

    pub fn send_transaction(&mut self, payee: PublicKey, amount: u64) {
        let transaction = Transaction::new(payee, amount, &self.key_pair);
        self.transactions.lock().unwrap().push_back(transaction);
    }
}

struct TransactionStream {
    transactions: Arc<Mutex<VecDeque<Transaction>>>,
}

impl Future for TransactionStream {
    type Output = Transaction;

    fn poll(self: std::pin::Pin<&mut Self>, _: &mut task::Context<'_>) -> task::Poll<Self::Output> {
        let mut transactions = self.transactions.lock().unwrap();

        if let Some(transaction) = transactions.pop_front() {
            task::Poll::Ready(transaction)
        } else {
            task::Poll::Pending
        }
    }
}

impl crate::future::FusedFuture for TransactionStream {
    fn is_terminated(&self) -> bool {
        false
    }
}
