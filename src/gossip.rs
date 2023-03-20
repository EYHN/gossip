use rand::seq::SliceRandom;

pub trait GossipNode {
    type Id;
    type PushMsg;
    type PullMsg;
    fn id(&self) -> Self::Id;
    fn prepare(&self) -> Self::PushMsg;
    fn push(&self, msg: Self::PushMsg) -> Self::PullMsg;
    fn pull(&self, msg: Self::PullMsg);
}

pub struct GossipSimulator<'a, Node> {
    pub nodes: &'a [Node],
}

impl<'a, Node> GossipSimulator<'a, Node>
where
    Node: GossipNode,
    Node::Id: Eq,
{
    pub fn round(&self) {
        for node in self.nodes.iter() {
            let peer = Self::select_random_peer(&self.nodes);
            if node.id() == peer.id() {
                continue;
            }
            let push_msg = node.prepare();
            let pull_msg = peer.push(push_msg);
            node.pull(pull_msg);
        }
    }

    fn select_random_peer(nodes: &[Node]) -> &Node {
        let mut rng = rand::thread_rng();
        nodes.choose(&mut rng).unwrap()
    }
}
