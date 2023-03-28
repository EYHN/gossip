use gossip::{
    gossip::{
        GossipNode, GossipProtocolClient, GossipProtocolMode, GossipProtocolOption,
        GossipSimulator, GossipSimulatorOptions,
    },
    kv::KvNode,
};
use serde::{Deserialize, Serialize};
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

#[derive(Debug, Serialize, Deserialize)]
pub struct SimulatorOptions {
    num_nodes: u32,
    fanout: u32,
    message_delay: f64,
    client_timer: f64,
    client_timer_random: f64,
    protocol_mode: String,
}

#[wasm_bindgen]
pub fn create_simulator(options: JsValue) -> ExportedSimulator {
    let options = serde_wasm_bindgen::from_value::<SimulatorOptions>(options).unwrap();
    let num_nodes = options.num_nodes;

    // Create nodes
    let nodes: Vec<_> = (0..num_nodes)
        .map(|_| KvNode::new(Uuid::new_v4()))
        .collect();

    nodes[0].update("hello".to_string(), "world".to_string());

    let protocol_options = GossipProtocolOption {
        fanout: options.fanout,
        mode: match options.protocol_mode.as_str() {
            "pull" => GossipProtocolMode::PullOnly,
            "pushpull" => GossipProtocolMode::PushPull,
            _ => panic!("Unknown protocol mode"),
        },
    };

    let simulator = GossipSimulator::new(
        nodes
            .into_iter()
            .map(|n| GossipProtocolClient::new(n, protocol_options.clone()))
            .collect(),
        GossipSimulatorOptions {
            message_delay: options.message_delay,
            client_timer: options.client_timer,
            client_timer_random: options.client_timer_random,
        },
    );

    ExportedSimulator { inner: simulator }
}
