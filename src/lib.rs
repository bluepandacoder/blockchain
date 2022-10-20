pub type Hash = U256;
pub const MINING_REQ: U256 = U256 {
    0: [2 << 20, 0, 0, 0],
};
pub const MINING_REW: u64 = 100;
pub const TIME_BASE: u64 = 10;

pub mod blockchain;
pub mod mining;
pub mod p2p;
pub mod transaction;
pub mod node;

use std::time::{SystemTime, UNIX_EPOCH};

pub use node::Node;

pub use p2p::NetworkManager;

pub use async_std::{io, task};
pub use blockchain::Block;
pub use blockchain::Blockchain;
pub use ethereum_types::U256;
pub use futures::{
    prelude::{stream::StreamExt, *},
    select,
};
pub use libp2p::{
    gossipsub::{self, Gossipsub, GossipsubConfig, GossipsubEvent},
    identity,
    mdns::{Mdns, MdnsConfig, MdnsEvent},
    swarm::SwarmEvent,
    Multiaddr, NetworkBehaviour, PeerId, Swarm,
};
pub use serde::{Deserialize, Serialize};
pub use std::error::Error;
pub use std::sync::{Arc, Mutex};
pub use std::thread;
pub use transaction::Transaction;

pub use ed25519_dalek::{Keypair, PublicKey, Signature, Signer, Verifier};

trait Hashable {
    fn hash(&self) -> Hash;
}

pub fn now() -> u64 {
    let duration = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();

    duration.as_secs()
}