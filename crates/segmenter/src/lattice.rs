pub type NodeId = usize;

///
pub(crate) struct NodePath<'a> {
    nodes: &'a [LatticeNode],
    node_path: Vec<usize>,
}

///
impl<'a> NodePath<'a> {
    pub fn path(&self) -> impl Iterator<Item = &LatticeNode> {
        self.node_path.iter().rev().map(|i| &self.nodes[*i])
    }
}

///
#[derive(Debug, Clone, Copy)]
pub struct LatticeNode {
    pub term_id: Option<u32>,
    pub start: usize,
    pub end: usize,
    pub score: f32,
}

///
#[derive(Clone)]
pub struct Lattice {
    length: usize,
    nodes: Vec<LatticeNode>,
    start: Vec<Vec<NodeId>>,
    end: Vec<Vec<NodeId>>,
}

///
impl Lattice {
    const NODE_ID_NONE: usize = !0usize;
    const NODE_ID_BEGIN: usize = Self::NODE_ID_NONE - 1;

    ///
    pub fn new(node_count: usize, length: usize) -> Self {
        let start = vec![Vec::<NodeId>::new(); length];
        let mut end = vec![Vec::<NodeId>::new(); length];
        end[0].push(Self::NODE_ID_BEGIN);

        Self {
            length,
            nodes: Vec::with_capacity(node_count),
            start,
            end,
        }
    }

    ///
    pub fn add_node(&mut self, node: LatticeNode) {
        let node_id = self.nodes.len();
        self.start[node.start].push(node_id);
        self.end[node.end].push(node_id);
        self.nodes.push(node);
    }

    ///
    pub(crate) fn find_path(&self) -> Vec<&LatticeNode> {
        assert!(self.nodes.len() < Self::NODE_ID_BEGIN);
        if (self.length == 0) || self.nodes.is_empty() {
            return Vec::new();
        }

        let mut total_scores: Vec<f32> =
            self.nodes.iter().map(|node| node.score).collect();
        let mut previous_nodes = vec![Self::NODE_ID_NONE; self.nodes.len()];

        for i in self.start[0].iter() {
            previous_nodes[*i] = Self::NODE_ID_BEGIN;
        }

        for i in 1..self.length {
            for right_node_id in self.start[i].iter() {
                // let right_node = &self.nodes[*right_node_id];
                let mut max_previous_node = None;
                let mut max_previous_score = 0.0;

                for left_node_id in self.end[i].iter() {
                    // let left_node = &self.nodes[*left_node_id];

                    if previous_nodes[*left_node_id] != Self::NODE_ID_NONE {
                        let prev_total_score = total_scores[*left_node_id];

                        if prev_total_score > max_previous_score {
                            max_previous_score = prev_total_score;
                            max_previous_node = Some(*left_node_id);
                        }
                    }
                }

                if let Some(max_previous_node) = max_previous_node {
                    previous_nodes[*right_node_id] = max_previous_node;
                    total_scores[*right_node_id] += max_previous_score;
                }
            }
        }

        let mut max_ending_node = None;
        let mut max_ending_score = 0.0;

        for node_id in self.end[self.length - 1].iter() {
            if previous_nodes[*node_id] != Self::NODE_ID_NONE {
                let prev_total_score = total_scores[*node_id];
                if prev_total_score > max_ending_score {
                    max_ending_score = prev_total_score;
                    max_ending_node = Some(*node_id);
                }
            }
        }

        if max_ending_node.is_none() {
            return Vec::new();
        }
        let mut node_path = Vec::with_capacity(self.length);
        let mut current_node_id = max_ending_node.unwrap();
        while previous_nodes[current_node_id] != Self::NODE_ID_BEGIN {
            node_path.push(current_node_id);
            current_node_id = previous_nodes[current_node_id];
        }
        node_path.push(current_node_id);

        node_path.iter().rev().map(|i| &self.nodes[*i]).collect()
    }
}
