use crdts::CmRDT;
use crdts::{map::Op as CrdtMapOp, MVReg, Map as CrdtMap, VClock};
use std::hash::Hash;
use std::sync::Mutex;
use uuid::Uuid;

use crate::gossip::GossipNode;

pub type KvGossipPush = VClock<Uuid>;
pub type KvGossipPull = Vec<CrdtMapOp<String, MVReg<String, Uuid>, Uuid>>;

#[derive(Debug, Default)]
pub struct KvState {
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
pub struct KvNode {
    id: Uuid,
    state: Mutex<KvState>,
}

impl KvNode {
    pub fn new(id: Uuid) -> Self {
        Self {
            id,
            state: Default::default(),
        }
    }
    pub fn update(&self, key: String, value: String) {
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

    pub fn get(&self, key: String) -> Option<Vec<String>> {
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

    fn push(&self, target_clock: KvGossipPush) -> Option<KvGossipPull> {
        let state = self.state.lock().unwrap();
        let current_clock = state.data.len().add_clock;
        if current_clock > target_clock || current_clock.concurrent(&target_clock) {
            Some(state.ops_after(target_clock))
        } else {
            None
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

    fn debug_hash(&self, hasher: &mut impl std::hash::Hasher) {
        let state = self.state.lock().unwrap();
        let current_clock = state.data.len().add_clock;
        current_clock.hash(hasher);
    }

    fn debug_state(&self) -> Option<serde_json::Value> {
        let state = self.state.lock().unwrap();
        let mut map = serde_json::Map::new();

        for item_ctx in state.data.iter() {
            let mut item = item_ctx.val.1.read().val.clone();
            map.insert(
                item_ctx.val.0.clone(),
                if item.len() == 1 {
                    serde_json::to_value(std::mem::take(&mut item[0])).unwrap()
                } else {
                    serde_json::to_value(item).unwrap()
                },
            );
        }
        Some(serde_json::Value::Object(map))
    }
}
