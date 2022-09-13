pub type Hash = U256;
pub const MINING_REQ: U256 = U256 {
    0: [2 << 12, 0, 0, 0],
};

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
pub use serde::{Deserialize, Serialize};
pub use std::error::Error;
pub use std::sync::{Arc, Mutex};
pub use std::thread;
pub use transaction::Transaction;

trait Hashable {
    fn hash(&self) -> Hash;
}
