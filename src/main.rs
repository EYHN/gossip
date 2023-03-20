mod gossip;

use crdts::CmRDT;
use crdts::{map::Op as CrdtMapOp, MVReg, Map as CrdtMap, VClock};
use gossip::GossipNode;
use rand::seq::SliceRandom;
use std::sync::Mutex;
use uuid::Uuid;

type KvGossipPush = VClock<Uuid>;
type KvGossipPull = Vec<CrdtMapOp<String, MVReg<String, Uuid>, Uuid>>;

#[derive(Debug, Default)]
struct KvState {
    data: CrdtMap<String, MVReg<String, Uuid>, Uuid>,
    ops: Vec<CrdtMapOp<String, MVReg<String, Uuid>, Uuid>>,
}

impl KvState {
    fn ops_after(&self, after: VClock<Uuid>) -> Vec<CrdtMapOp<String, MVReg<String, Uuid>, Uuid>> {
        self.ops
            .iter()
            .filter(|op| match op {
                CrdtMapOp::Rm { clock, keyset: _ } => clock > &after || clock.concurrent(&after),
                CrdtMapOp::Up { dot, key: _, op: _ } => dot > &after.dot(dot.actor),
            })
            .map(|op| op.clone())
            .collect()
    }
}

#[derive(Debug)]
struct KvNode {
    id: Uuid,
    state: Mutex<KvState>,
}

impl GossipNode for KvNode {
    type Id = Uuid;

    type PushMsg = KvGossipPush;

    type PullMsg = KvGossipPull;

    fn prepare(&self) -> KvGossipPush {
        self.state.lock().unwrap().data.len().add_clock
    }

    fn push(&self, target_clock: KvGossipPush) -> KvGossipPull {
        let mut state = self.state.lock().unwrap();
        let current_clock = state.data.len().add_clock;
        if current_clock < target_clock || current_clock.concurrent(&target_clock) {
            state.ops_after(target_clock)
        } else {
            Vec::new()
        }
    }

    fn pull(&self, ops: KvGossipPull) {
        let mut state = self.state.lock().unwrap();
        for op in ops {
            state.data.apply(op.clone());
            state.ops.push(op)
        }
    }

    fn id(&self) -> Self::Id {
        self.id
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

    let nodeA = KvNode {
        id: Uuid::new_v4(),
        state: Mutex::new(Default::default()),
    };

    let nodeB = KvNode {
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
