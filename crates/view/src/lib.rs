use gossip::{
    gossip::{
        GossipNode, GossipProtocolClient, GossipProtocolOption, GossipSimulator,
        GossipSimulatorOptions,
    },
    kv::KvNode,
};
use uuid::Uuid;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    // Use `js_namespace` here to bind `console.log(..)` instead of just
    // `log(..)`
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

macro_rules! console_log {
    // Note that this is using the `log` function imported above during
    // `bare_bones`
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

#[wasm_bindgen]
pub struct ExportedSimulator {
    inner: GossipSimulator<KvNode>,
}

#[wasm_bindgen]
impl ExportedSimulator {
    #[wasm_bindgen]
    pub fn tick(&mut self, t: f64) {
        self.inner.tick(t)
    }

    #[wasm_bindgen]
    pub fn debug(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&self.inner.debug()).unwrap()
    }

    #[wasm_bindgen]
    pub fn set_kv(&self, id: &str, k: &str, v: &str) {
        let uuid = Uuid::parse_str(id).unwrap();
        self.inner
            .clients
            .iter()
            .find(|c| c.id() == uuid)
            .unwrap()
            .update(k.to_string(), v.to_string())
    }

    #[wasm_bindgen]
    pub fn debug_client(&self, id: &str) -> JsValue {
        serde_wasm_bindgen::to_value(&self.inner.debug_client(Uuid::parse_str(id).unwrap()))
            .unwrap()
    }
}

#[wasm_bindgen]
pub fn createSimulator() -> ExportedSimulator {
    let num_nodes = 50;

    // Create nodes
    let nodes: Vec<_> = (0..num_nodes)
        .map(|_| KvNode::new(Uuid::new_v4()))
        .collect();

    let options = GossipProtocolOption { fanout: 3 };

    let simulator = GossipSimulator::new(
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

    ExportedSimulator { inner: simulator }
}
