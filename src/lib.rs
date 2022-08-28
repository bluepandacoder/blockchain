pub mod p2p;
pub mod blockchain;

pub use async_std::{io, task};
pub use futures::{
    prelude::{stream::StreamExt, *},
    select,
};
pub use libp2p::{
    floodsub::{self, Floodsub, FloodsubEvent},
    identity,
    mdns::{Mdns, MdnsConfig, MdnsEvent},
    swarm::SwarmEvent,
    Multiaddr, NetworkBehaviour, PeerId, Swarm,
};
pub use std::error::Error;
pub use blockchain::BlockChain;
pub use blockchain::Block;