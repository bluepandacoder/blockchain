use libp2p::identity::Keypair;

use super::*;

// We create a custom network behaviour that combines floodsub and mDNS.
// Use the derive to generate delegating NetworkBehaviour impl.
#[derive(NetworkBehaviour)]
#[behaviour(out_event = "OutEvent")]
pub struct PeerBehaviour {
    pub floodsub: Floodsub,
    pub mdns: Mdns,

    // Struct fields which do not implement NetworkBehaviour need to be ignored
    #[behaviour(ignore)]
    #[allow(dead_code)]
    ignored_member: bool,
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
pub enum OutEvent {
    Floodsub(FloodsubEvent),
    Mdns(MdnsEvent),
}

impl From<MdnsEvent> for OutEvent {
    fn from(v: MdnsEvent) -> Self {
        Self::Mdns(v)
    }
}

impl From<FloodsubEvent> for OutEvent {
    fn from(v: FloodsubEvent) -> Self {
        Self::Floodsub(v)
    }
}

pub struct NetworkManager {
    pub swarm: Swarm<PeerBehaviour>,
    pub local_key: Keypair,
}

impl NetworkManager {
    pub async fn start(topics: Vec<floodsub::Topic>) -> Result<Self, Box<dyn Error>> {

        // Create a random PeerId
        let local_key = identity::Keypair::generate_ed25519();
        let local_peer_id = PeerId::from(local_key.public());
        println!("Local peer id: {:?}", local_peer_id);

        // Set up an encrypted DNS-enabled TCP Transport over the Mplex and Yamux protocols
        let transport = libp2p::development_transport(local_key.clone()).await?;

        // Create a Swarm to manage peers and events
        let mut swarm = {
            let mdns = task::block_on(Mdns::new(MdnsConfig::default()))?;
            let mut behaviour = PeerBehaviour {
                floodsub: Floodsub::new(local_peer_id),
                mdns,
                ignored_member: false,
            };

            for floodsub_topic in &topics {
                behaviour.floodsub.subscribe(floodsub_topic.clone());
            }
            Swarm::new(transport, behaviour, local_peer_id)
        };

        // Listen on all interfaces and whatever port the OS assigns
        swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

        Ok(
            Self {
                swarm,
                local_key,
            }
        )

    }
}