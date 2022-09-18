pub type Hash = U256;
pub const MINING_REQ: U256 = U256 {
    0: [2 << 20, 0, 0, 0],
};
pub const MINING_REW: u64 = 100;

pub mod blockchain;
pub mod mining;
pub mod p2p;
pub mod transaction;

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
use serde::ser::SerializeStruct;
pub use serde::{Deserialize, Serialize};
pub use std::error::Error;
pub use std::sync::{Arc, Mutex};
pub use std::thread;
pub use transaction::Transaction;

pub use ed25519_dalek::{Signature, Signer, Verifier, PublicKey, Keypair};

trait Hashable {
    fn hash(&self) -> Hash;
}
