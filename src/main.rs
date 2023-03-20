mod gossip;

use crdts::CmRDT;
use crdts::{map::Op as CrdtMapOp, MVReg, Map as CrdtMap, VClock};
use gossip::{GossipNode, GossipSimulator};
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

impl KvNode {
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
        let state = self.state.lock().unwrap();

        state
            .data
            .get(&key)
            .val
            .and_then(|item| Some(item.read().val))
    }
}

impl GossipNode for KvNode {
    type Id = Uuid;

    type PushMsg = KvGossipPush;

    type PullMsg = KvGossipPull;

    fn prepare(&self) -> KvGossipPush {
        self.state.lock().unwrap().data.len().add_clock
    }

    fn push(&self, target_clock: KvGossipPush) -> KvGossipPull {
        let state = self.state.lock().unwrap();
        let current_clock = state.data.len().add_clock;
        if current_clock > target_clock || current_clock.concurrent(&target_clock) {
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

    // Create nodes
    let nodes: Vec<_> = (0..num_nodes)
        .map(|_| KvNode {
            id: Uuid::new_v4(),
            state: Mutex::new(Default::default()),
        })
        .collect();

    nodes[0].update("abc".to_string(), "efg".to_string());

    let simulator = GossipSimulator { nodes: &nodes };

    simulator.round();

    dbg!(nodes[0].get("abc".to_string()));
    dbg!(nodes[1].get("abc".to_string()));
    dbg!(nodes[2].get("abc".to_string()));
    dbg!(nodes[3].get("abc".to_string()));
    dbg!(nodes[4].get("abc".to_string()));

    // // Initialize the state of the first node
    // nodes[0]
    //     .state
    //     .lock()
    //     .unwrap()
    //     .insert("Hello, world!".to_string());

    // // Simulate the gossip protocol
    // simulate_gossip(Arc::new(nodes), rounds);
}
