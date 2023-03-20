use std::{fmt::Debug, ops::Deref};

use rand::seq::{IteratorRandom, SliceRandom};

pub trait GossipNode {
    type Id: Eq + Clone + Debug;
    type PushMsg;
    type PullMsg;
    fn id(&self) -> Self::Id;
    fn prepare(&self) -> Self::PushMsg;
    fn push(&self, msg: Self::PushMsg) -> Self::PullMsg;
    fn pull(&self, msg: Self::PullMsg);
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

pub struct GossipSimulator<Node: GossipNode> {
    pub clients: Vec<GossipProtocolClient<Node>>,
    node_ids: Vec<Node::Id>,
    messages: Vec<GossipSimulatorMessage<Node>>,
}

impl<Node: GossipNode> GossipRuntime<Node> for GossipSimulator<Node> {
    fn reachable_node_ids(&self) -> &[Node::Id] {
        &self.node_ids
    }

    fn send(&self, from_id: Node::Id, to_id: Node::Id, msg: GossipMsg<Node>) {
        for client in self.clients.iter() {
            let client_id = client.id();
            if client_id == to_id {
                client.on_receive(from_id, msg, self);
                break;
            }
        }
    }
}

impl<Node: GossipNode> GossipSimulator<Node> {
    pub fn new(clients: Vec<GossipProtocolClient<Node>>) -> Self {
        Self {
            node_ids: clients.iter().map(|c| c.id()).collect(),
            clients,
            messages: Default::default(),
        }
    }
    pub fn round(&self) {
        let mut rng = rand::thread_rng();
        self.clients.choose(&mut rng).unwrap().on_send(self);
    }
}
