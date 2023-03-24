use std::{
    collections::hash_map::DefaultHasher, fmt::Debug, hash::Hasher, ops::Deref, sync::Mutex,
};

use rand::{seq::IteratorRandom, Rng};

pub trait GossipNode {
    type Id: Eq + Clone + Debug + ToString;
    type PushMsg;
    type PullMsg;
    fn id(&self) -> Self::Id;
    fn prepare(&self) -> Self::PushMsg;
    fn push(&self, msg: Self::PushMsg) -> Self::PullMsg;
    fn pull(&self, msg: Self::PullMsg);
    fn debug_hash(&self, _hasher: &mut impl Hasher) {}
    fn debug_state(&self) -> Option<serde_json::Value> {
        None
    }
}

pub enum GossipMsg<Node: GossipNode> {
    Push(Node::PushMsg),
    Pull(Node::PullMsg),
    PushPull(Node::PushMsg, Node::PullMsg),
}

pub trait GossipRuntime<Node: GossipNode> {
    fn reachable_node_ids(&self) -> &[Node::Id];

    fn send(&self, from_id: Node::Id, to_id: Node::Id, msg: GossipMsg<Node>);
}

#[derive(Debug, Clone)]
pub struct GossipProtocolOption {
    pub fanout: u32,
}

pub struct GossipProtocolClient<Node: GossipNode> {
    pub node: Node,
    options: GossipProtocolOption,
}

impl<Node: GossipNode> GossipProtocolClient<Node> {
    pub fn new(node: Node, options: GossipProtocolOption) -> Self {
        Self { node, options }
    }
    fn on_send(&self, runtime: &impl GossipRuntime<Node>) {
        let self_id = self.id();
        let mut rng = rand::thread_rng();
        let target_node_ids = runtime
            .reachable_node_ids()
            .iter()
            .filter(|id| id != &&self_id)
            .choose_multiple(&mut rng, self.options.fanout as usize);

        for node_id in target_node_ids {
            runtime.send(
                self_id.clone(),
                node_id.clone(),
                GossipMsg::Push(self.node.prepare()),
            )
        }
    }

    fn on_receive(&self, id: Node::Id, msg: GossipMsg<Node>, runtime: &impl GossipRuntime<Node>) {
        match msg {
            GossipMsg::Push(push) => runtime.send(
                self.id(),
                id,
                GossipMsg::PushPull(self.prepare(), self.push(push)),
            ),
            GossipMsg::Pull(pull) => self.pull(pull),
            GossipMsg::PushPull(push, pull) => {
                self.pull(pull);
                runtime.send(self.id(), id, GossipMsg::Pull(self.push(push)))
            }
        }
    }
}

impl<Node: GossipNode> Deref for GossipProtocolClient<Node> {
    type Target = Node;

    fn deref(&self) -> &Self::Target {
        &self.node
    }
}

struct GossipSimulatorMessage<Node: GossipNode> {
    from_id: Node::Id,
    to_id: Node::Id,
    msg: GossipMsg<Node>,
    start: f64,
    end: f64,
}

#[derive(Debug, Clone)]
pub struct GossipSimulatorOptions {
    pub message_delay: f64,
    pub client_timer: f64,
    pub client_timer_random: f64,
}

pub struct GossipSimulator<Node: GossipNode> {
    pub clients: Vec<GossipProtocolClient<Node>>,
    node_ids: Vec<Node::Id>,
    client_timers: Vec<(f64, f64)>,
    time: f64,
    messages: Mutex<Vec<GossipSimulatorMessage<Node>>>,
    options: GossipSimulatorOptions,
}

impl<Node: GossipNode> GossipRuntime<Node> for GossipSimulator<Node> {
    fn reachable_node_ids(&self) -> &[Node::Id] {
        &self.node_ids
    }

    fn send(&self, from_id: Node::Id, to_id: Node::Id, msg: GossipMsg<Node>) {
        self.messages.lock().unwrap().push(GossipSimulatorMessage {
            from_id,
            to_id,
            msg,
            start: self.time,
            end: self.time + self.options.message_delay,
        });
    }
}

impl<Node: GossipNode> GossipSimulator<Node> {
    pub fn new(clients: Vec<GossipProtocolClient<Node>>, options: GossipSimulatorOptions) -> Self {
        Self {
            node_ids: clients.iter().map(|c| c.id()).collect(),
            client_timers: clients
                .iter()
                .map(|_| Self::generate_client_timer(&options, 0.0))
                .collect(),
            clients,
            time: 0.0,
            messages: Default::default(),
            options,
        }
    }
    pub fn tick(&mut self, time: f64) {
        self.time += time;
        let mut timeout_clients = self
            .client_timers
            .iter()
            .enumerate()
            .filter(|(_, (_, end))| self.time >= *end)
            .map(|(i, a)| (i, a.clone()))
            .collect::<Vec<_>>();
        timeout_clients
            .sort_by(|(_, (_, a_end)), (_, (_, b_end))| a_end.partial_cmp(&b_end).unwrap());
        for (i, (_, _)) in timeout_clients {
            self.clients[i].on_send(self);
            self.client_timers[i] = Self::generate_client_timer(&self.options, self.time);
        }

        let mut arrived_messages = Vec::new();
        let mut i = 0;
        let mut messages_lock = self.messages.lock().unwrap();
        while i < messages_lock.len() {
            if self.time >= messages_lock[i].end {
                arrived_messages.push(messages_lock.remove(i));
            } else {
                i += 1;
            }
        }
        std::mem::drop(messages_lock);
        arrived_messages.sort_by(|a, b| a.end.partial_cmp(&b.end).unwrap());
        for msg in arrived_messages {
            self.clients
                .iter()
                .find(|c| c.id() == msg.to_id)
                .unwrap()
                .on_receive(msg.from_id, msg.msg, self)
        }
    }
    pub fn generate_client_timer(
        options: &GossipSimulatorOptions,
        current_time: f64,
    ) -> (f64, f64) {
        let mut rng = rand::thread_rng();
        let time = rng.gen_range(
            options.client_timer - options.client_timer_random
                ..options.client_timer + options.client_timer_random,
        );
        return (current_time, current_time + time);
    }
    pub fn debug(&self) -> GossipSimulatorDebug {
        GossipSimulatorDebug {
            time: self.time,
            messages: self
                .messages
                .lock()
                .unwrap()
                .iter()
                .map(|msg| GossipSimulatorMessageDebug {
                    from: msg.from_id.to_string(),
                    to: msg.to_id.to_string(),
                    progress: (self.time - msg.start) / (msg.end - msg.start),
                    kind: match msg.msg {
                        GossipMsg::Push(_) => "Push".to_string(),
                        GossipMsg::Pull(_) => "Pull".to_string(),
                        GossipMsg::PushPull(_, _) => "PushPull".to_string(),
                    },
                })
                .collect(),
            clients: self
                .clients
                .iter()
                .enumerate()
                .map(|(i, client)| {
                    let timer = self.client_timers[i];
                    let hash = {
                        let mut hasher = DefaultHasher::new();
                        client.debug_hash(&mut hasher);
                        format!("{:x}", hasher.finish())
                    };
                    GossipSimulatorClientDebug {
                        id: client.id().to_string(),
                        hash: Some(hash),
                        progress: self.time - timer.0 / timer.1 - timer.0,
                    }
                })
                .collect(),
        }
    }

    pub fn debug_client(&self, id: Node::Id) -> Option<serde_json::Value> {
        self.clients
            .iter()
            .find(|c| c.id() == id)
            .and_then(|c| c.debug_state())
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct GossipSimulatorMessageDebug {
    pub from: String,
    pub to: String,
    pub progress: f64,
    pub kind: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct GossipSimulatorClientDebug {
    pub id: String,
    pub hash: Option<String>,
    pub progress: f64,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct GossipSimulatorDebug {
    pub time: f64,
    pub messages: Vec<GossipSimulatorMessageDebug>,
    pub clients: Vec<GossipSimulatorClientDebug>,
}
