use crdts::CmRDT;
use crdts::{map::Op as CrdtMapOp, MVReg, Map as CrdtMap, VClock};
use rand::seq::SliceRandom;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

#[derive(Debug, Default)]
struct NodeState {
    data: CrdtMap<String, MVReg<String, Uuid>, Uuid>,
    ops: Vec<CrdtMapOp<String, MVReg<String, Uuid>, Uuid>>,
}

#[derive(Debug)]
struct Node {
    id: Uuid,
    state: Mutex<NodeState>,
}

impl Node {
    fn push_version(&self, other: &Node) {
        let curr_version = self.state.lock().unwrap().data.len().add_clock;
        other.receive_version(self, curr_version);
    }

    fn receive_version(&self, from: &Node, version: VClock<Uuid>) {
        let curr_version = self.state.lock().unwrap().data.len().add_clock;
        dbg!(&curr_version);
        dbg!(&version);
        if curr_version < version || curr_version.concurrent(&version) {
            let ops = from.get_ops_after(curr_version);
            let mut state = self.state.lock().unwrap();
            for op in ops {
                state.data.apply(op.clone());
                state.ops.push(op)
            }
        }
    }

    fn get_ops_after(
        &self,
        version: VClock<Uuid>,
    ) -> Vec<CrdtMapOp<String, MVReg<String, Uuid>, Uuid>> {
        self.state
            .lock()
            .unwrap()
            .ops
            .iter()
            .filter(|op| match op {
                CrdtMapOp::Rm { clock, keyset } => clock > &version || clock.concurrent(&version),
                CrdtMapOp::Up { dot, key, op } => dot > &version.dot(dot.actor),
            })
            .map(|op| op.clone())
            .collect()
    }

    fn update(&self, key: String, value: String) {
        let mut state = self.state.lock().unwrap();

        let ctx = state.data.len();
        let op = state
            .data
            .update(key, ctx.derive_add_ctx(self.id), |v, ctx| {
                v.write(value, ctx)
            });
        state.data.apply(op.clone());
        state.ops.push(op);
    }

    fn get(&self, key: String) -> Option<Vec<String>> {
        let mut state = self.state.lock().unwrap();

        let ctx = state.data.len();
        state
            .data
            .get(&key)
            .val
            .and_then(|item| Some(item.read().val))
    }
}

// use std::sync::mpsc;
// use std::thread;

// fn simulate_gossip(nodes: Arc<Vec<Node>>, rounds: usize) {
//     let (tx, rx) = mpsc::channel();

//     // Spawn a thread for each node to simulate the gossip protocol
//     for node in nodes.iter() {
//         let tx = tx.clone();
//         let node = node.clone();
//         let nodes_ownen = nodes.clone();
//         thread::spawn(move || {
//             for _ in 0..rounds {
//                 // Push state to a random peer
//                 let peer = select_random_peer(&nodes_ownen, node.id);
//                 node.push_state(&peer);

//                 // Pull state from a random peer
//                 let peer = select_random_peer(&nodes_ownen, node.id);
//                 node.pull_state(&peer);

//                 // Send the node's state to the main thread for monitoring
//                 tx.send((node.id, node.get_state())).unwrap();
//             }
//         });
//     }

//     // Monitor the state of each node
//     for _ in 0..rounds * nodes.len() {
//         let (node_id, state) = rx.recv().unwrap();
//         println!("Node {}: {:?}", node_id, state);
//     }
// }

fn select_random_peer(nodes: &[Node], current_node_id: usize) -> &Node {
    let mut rng = rand::thread_rng();
    nodes.choose(&mut rng).unwrap_or_else(|| {
        panic!(
            "Failed to select a random peer. Current node ID: {}",
            current_node_id
        )
    })
}

fn main() {
    let num_nodes = 5;
    let rounds = 3;

    // Create nodes
    // let nodes: Vec<_> = (0..num_nodes)
    //     .map(|id| Node {
    //         id,
    //         state: Arc::new(Mutex::new(HashSet::new())),
    //     })
    //     .collect();

    let nodeA = Node {
        id: Uuid::new_v4(),
        state: Mutex::new(Default::default()),
    };

    let nodeB = Node {
        id: Uuid::new_v4(),
        state: Mutex::new(Default::default()),
    };

    nodeA.update("abc".to_string(), "efg".to_string());

    dbg!(nodeA.get("abc".to_string()));

    nodeB.update("abc".to_string(), "eee".to_string());

    nodeA.push_version(&nodeB);

    dbg!(nodeB.get("abc".to_string()));

    nodeB.push_version(&nodeA);
    nodeA.push_version(&nodeB);

    dbg!(nodeB.get("abc".to_string()));

    nodeB.update("abc".to_string(), "hello".to_string());

    nodeB.push_version(&nodeA);
    nodeA.push_version(&nodeB);

    dbg!(nodeA.get("abc".to_string()));
    dbg!(nodeB.get("abc".to_string()));


    // // Initialize the state of the first node
    // nodes[0]
    //     .state
    //     .lock()
    //     .unwrap()
    //     .insert("Hello, world!".to_string());

    // // Simulate the gossip protocol
    // simulate_gossip(Arc::new(nodes), rounds);
}
