pub mod gossip;
pub mod kv;

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use crate::{
        gossip::{
            GossipProtocolClient, GossipProtocolOption, GossipSimulator, GossipSimulatorOptions,
        },
        kv::KvNode,
    };

    #[test]
    fn test_simulator() {
        let num_nodes = 5;

        // Create nodes
        let nodes: Vec<_> = (0..num_nodes)
            .map(|_| KvNode::new(Uuid::new_v4()))
            .collect();

        nodes[0].update("abc".to_string(), "efg".to_string());

        let options = GossipProtocolOption { fanout: 1 };

        let mut simulator = GossipSimulator::new(
            nodes
                .into_iter()
                .map(|n| GossipProtocolClient::new(n, options.clone()))
                .collect(),
            GossipSimulatorOptions {
                message_delay: 1.0,
                client_timer: 3.0,
                client_timer_random: 1.0,
            },
        );

        simulator.tick(10.0);

        dbg!(simulator.debug());
    }
}
