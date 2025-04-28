//! This module implements the Ranked-DAG data structure. I's a data structure
//! that represents the edges between nodes in the dag as well as the leveling
//! of the nodes. A rank is the ordering of some nodes along the x-axis. Users
//! of this data structure may change the leveling of nodes, and the only
//! guarantee is that the nodes are assigned to some level.

use std::cmp;

/// The Ranked-DAG data structure.
#[derive(Debug)]
pub struct DAG {
    /// A list of nodes in the dag.
    nodes: Vec<Node>,

    /// Places nodes in levels.
    ranks: RankType,

    /// levels info
    levels: Vec<usize>,

    /// Perform validation checks.
    validate: bool,
}

/// Used by users to keep track of nodes that are saved in the DAG.
#[derive(Copy, Clone, Default, PartialEq, PartialOrd, Eq, Ord, Hash, Debug)]
pub struct NodeHandle {
    idx: usize,
}

impl NodeHandle {
    pub fn new(x: usize) -> Self {
        NodeHandle { idx: x }
    }
    pub fn get_index(&self) -> usize {
        self.idx
    }
}

impl From<usize> for NodeHandle {
    fn from(idx: usize) -> Self {
        NodeHandle { idx }
    }
}

#[derive(Debug)]
struct Node {
    // Points to other edges.
    successors: Vec<NodeHandle>,
    predecessors: Vec<NodeHandle>,
}

pub type RankType = Vec<Vec<NodeHandle>>;

impl Node {
    pub fn new() -> Self {
        Node {
            successors: Vec::new(),
            predecessors: Vec::new(),
        }
    }
}

/// Node iterator for iterating over nodes in the graph.
#[derive(Debug)]
pub struct NodeIterator {
    curr: usize,
    last: usize,
}

impl Iterator for NodeIterator {
    type Item = NodeHandle;

    fn next(&mut self) -> Option<Self::Item> {
        if self.curr == self.last {
            return None;
        }

        let item = Some(NodeHandle::from(self.curr));
        self.curr += 1;
        item
    }
}

impl DAG {
    pub fn new() -> Self {
        DAG {
            nodes: Vec::new(),
            ranks: Vec::new(),
            levels: Vec::new(),
            validate: true,
        }
    }

    pub fn set_validate(&mut self, validate: bool) {
        self.validate = validate;
    }

    pub fn clear(&mut self) {
        self.nodes.clear();
        self.ranks.clear();
        self.levels.clear();
    }

    pub fn iter(&self) -> NodeIterator {
        NodeIterator {
            curr: 0,
            last: self.nodes.len(),
        }
    }

    pub fn add_edge(&mut self, from: NodeHandle, to: NodeHandle) {
        self.nodes[from.idx].successors.push(to);
        self.nodes[to.idx].predecessors.push(from);
    }

    /// Remove an edge from \p from to \p to.
    /// \returns True if an edge was removed.
    pub fn remove_edge(&mut self, from: NodeHandle, to: NodeHandle) -> bool {
        let succ = &mut self.nodes[from.idx].successors;
        let mut removed_succ = false;

        if let Some(pos) = succ.iter().position(|x| *x == to) {
            succ.remove(pos);
            removed_succ = true;
        }

        let pred = &mut self.nodes[to.idx].predecessors;
        let mut removed_pred = false;
        if let Some(pos) = pred.iter().position(|x| *x == from) {
            pred.remove(pos);
            removed_pred = true;
        }

        // We must preserve the invariant that the pred-succ list must always
        // be up to date.
        assert_eq!(removed_pred, removed_succ);
        removed_pred
    }

    /// Create a new node.
    pub fn new_node(&mut self) -> NodeHandle {
        self.nodes.push(Node::new());
        self.levels.push(0);
        let node = NodeHandle::new(self.nodes.len() - 1);
        self.add_element_to_rank(node, 0, false);
        node
    }

    /// Create \p n new nodes.
    pub fn new_nodes(&mut self, n: usize) {
        for _ in 0..n {
            self.nodes.push(Node::new());
            self.levels.push(0);
            let node = NodeHandle::new(self.nodes.len() - 1);
            self.add_element_to_rank(node, 0, false);
        }
        self.verify();
    }

    pub fn successors(&self, from: NodeHandle) -> &Vec<NodeHandle> {
        &self.nodes[from.idx].successors
    }

    pub fn predecessors(&self, from: NodeHandle) -> &Vec<NodeHandle> {
        &self.nodes[from.idx].predecessors
    }

    pub fn single_pred(&self, from: NodeHandle) -> Option<NodeHandle> {
        if self.nodes[from.idx].predecessors.len() == 1 {
            return Some(self.nodes[from.idx].predecessors[0]);
        }
        None
    }

    pub fn single_succ(&self, from: NodeHandle) -> Option<NodeHandle> {
        if self.nodes[from.idx].successors.len() == 1 {
            return Some(self.nodes[from.idx].successors[0]);
        }
        None
    }

    pub fn verify(&self) {
        if self.validate {
            // Check that the node indices are valid.
            for node in &self.nodes {
                for edge in &node.successors {
                    assert!(edge.idx < self.nodes.len());
                }
            }

            // Check that the graph is a DAG.
            for (i, node) in self.nodes.iter().enumerate() {
                let from = NodeHandle::from(i);
                for dest in node.successors.iter() {
                    let reachable =
                        self.is_reachable(*dest, from) && from != *dest;
                    assert!(!reachable, "We found a cycle!");
                }
            }

            // Make sure that all of the nodes are in ranks.
            assert_eq!(self.count_nodes_in_ranks(), self.len());
        }
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// \returns True if the node \to is reachable from the node \p from.
    /// This internal method is used for the verification of the graph.
    fn is_reachable_inner(
        &self,
        from: NodeHandle,
        to: NodeHandle,
        visited: &mut Vec<bool>,
    ) -> bool {
        if from == to {
            return true;
        }

        // Don't step into a cycle.
        if visited[from.idx] {
            return false;
        }

        // Push to the dfs stack.
        visited[from.idx] = true;

        let from_node = &self.nodes[from.idx];
        for edge in &from_node.successors {
            if self.is_reachable_inner(*edge, to, visited) {
                return true;
            }
        }

        // Pop from the dfs stack.
        visited[from.idx] = false;
        false
    }

    /// \returns True if there is a path from \p 'from' to \p 'to'.
    pub fn is_reachable(&self, from: NodeHandle, to: NodeHandle) -> bool {
        if from == to {
            return true;
        }

        let mut visited = Vec::new();
        visited.resize(self.nodes.len(), false);
        self.is_reachable_inner(from, to, &mut visited)
    }

    /// Return the topological sort order of the nodes in the dag.
    /// This is implemented as the reverse post order scan.
    fn topological_sort(&self) -> Vec<NodeHandle> {
        // A list of vectors in post-order.
        let mut order: Vec<NodeHandle> = Vec::new();

        // Marks that a node is in the worklist.
        let mut visited = Vec::new();
        visited.resize(self.nodes.len(), false);

        // A tuple of handle, and command:
        // true- force push.
        // false- this is a child to visit.
        let mut worklist: Vec<(NodeHandle, bool)> = Vec::new();

        // Add all of the values that we want to compute into the worklist.
        for n in self.iter() {
            worklist.push((n, false));
        }

        while let Some((current, cmd)) = worklist.pop() {
            // Handle 'push' commands.
            if cmd {
                order.push(current);
                continue;
            }

            // Don't visit visited nodes.
            if visited[current.idx] {
                continue;
            }

            visited[current.idx] = true;

            // Save this node after all of the children are handles.
            worklist.push((current, true));

            // Add the children to the worklist.
            let node = &self.nodes[current.idx];
            for edge in &node.successors {
                worklist.push((*edge, false));
            }
        }

        // Turn the post-order to a reverse post order.
        order.reverse();
        order
    }

    // The methods below are related to the rank (placing nodes in levels). //

    /// \returns the number of ranks in the dag.
    pub fn num_levels(&self) -> usize {
        self.ranks.len()
    }

    /// \return a mutable reference to a row at level \p level.
    pub fn row_mut(&mut self, level: usize) -> &mut Vec<NodeHandle> {
        assert!(level < self.ranks.len(), "Invalid rank");
        &mut self.ranks[level]
    }

    /// \return a reference to a row at level \p level.
    pub fn row(&self, level: usize) -> &Vec<NodeHandle> {
        assert!(level < self.ranks.len(), "Invalid rank");
        &self.ranks[level]
    }

    /// \return a reference to the whole rank data structure.
    pub fn ranks(&self) -> &RankType {
        &self.ranks
    }

    /// \return a mutable reference to the whole rank data structure.
    pub fn ranks_mut(&mut self) -> &mut RankType {
        &mut self.ranks
    }

    /// \returns True if \p elem is the first node in the row \p level.
    pub fn is_first_in_row(&self, elem: NodeHandle, level: usize) -> bool {
        if level >= self.ranks.len() || self.ranks[level].is_empty() {
            return false;
        }
        self.ranks[level][0] == elem
    }

    /// \returns True if \p elem is the last node in the row \p level.
    pub fn is_last_in_row(&self, elem: NodeHandle, level: usize) -> bool {
        if level >= self.ranks.len() || self.ranks[level].is_empty() {
            return false;
        }
        let last_idx = self.ranks[level].len() - 1;
        self.ranks[level][last_idx] == elem
    }

    /// Place the element \p elem at the nth level \p level. If the level does
    /// not exist then create it. If \p prepend is set then the node is inserted
    /// at the beginning of the rank. The node must not be in the rank when this
    /// method is called.
    fn add_element_to_rank(
        &mut self,
        elem: NodeHandle,
        level: usize,
        prepend: bool,
    ) {
        while self.ranks.len() < level + 1 {
            self.ranks.push(Vec::new());
        }

        if prepend {
            self.ranks[level].insert(0, elem);
        } else {
            self.ranks[level].push(elem);
        }
        self.levels[elem.get_index()] = level;
    }

    /// Places all of the nodes in ranks (levels).
    pub fn recompute_node_ranks(&mut self) {
        assert!(!self.is_empty(), "Sorting an empty graph");
        let order = self.topological_sort();
        let levels = self.compute_levels(&order);
        self.ranks.clear();
        for (i, level) in levels.iter().enumerate() {
            self.add_element_to_rank(NodeHandle::from(i), *level, false);
        }
    }

    /// \returns the number of nodes that are in ranks.
    /// This is used for verification of the dag.
    fn count_nodes_in_ranks(&self) -> usize {
        let mut cnt = 0;
        for row in self.ranks.iter() {
            cnt += row.len();
        }
        cnt
    }

    /// Move the node \p node to a new level \p new_level.
    /// Place the node before \p node, or at the end.
    pub fn update_node_rank_level(
        &mut self,
        node: NodeHandle,
        new_level: usize,
        insert_before: Option<NodeHandle>,
    ) {
        let curr_level = self.level(node);
        let level = &mut self.ranks[curr_level];
        let idx = level
            .iter()
            .position(|x| *x == node)
            .expect("node not found");
        level.remove(idx);

        // Make sure that the row exists.
        while self.ranks.len() < new_level + 1 {
            self.ranks.push(Vec::new());
        }

        if let Option::Some(marker) = insert_before {
            let row = &mut self.ranks[new_level];
            for i in 0..row.len() {
                if row[i] == marker {
                    row.insert(i, node);
                    self.levels[node.get_index()] = new_level;
                    return;
                }
            }
            panic!("Can't find the marker node in the array");
        }

        self.ranks[new_level].push(node);
        self.levels[node.get_index()] = new_level;
        assert_eq!(self.level(node), new_level);
    }

    /// \returns the level of the node \p node in the rank.
    pub fn level(&self, node: NodeHandle) -> usize {
        assert!(node.get_index() < self.len(), "Node not in the dag");
        self.levels[node.get_index()]
        // for (i, row) in self.ranks.iter().enumerate() {
        //     if row.contains(&node) {
        //         return i;
        //     }
        // }
        // panic!("Unexpected node. Is the graph ranked?");
    }

    /// Computes and returns the level of each node in the graph based
    /// on the traversal order \p order.
    fn compute_levels(&self, order: &[NodeHandle]) -> Vec<usize> {
        let mut levels: Vec<usize> = Vec::new();
        assert_eq!(order.len(), self.nodes.len());

        // Levels has the same layout as the DAG node list.
        levels.resize(self.nodes.len(), 0);

        // For each node in the order (starting with a node of level zero).
        for src in order {
            // Update the level of all successors.
            for dest in self.nodes[src.idx].successors.iter() {
                // Ignore self edges.
                if src.idx == dest.idx {
                    continue;
                }
                levels[dest.idx] =
                    cmp::max(levels[dest.idx], levels[src.idx] + 1);
            }
        }

        // For each node in the order.
        for src in order {
            for dest in self.nodes[src.idx].successors.iter() {
                assert!(levels[dest.idx] >= levels[src.idx]);
            }
        }

        levels
    }
}

impl Default for DAG {
    fn default() -> Self {
        Self::new()
    }
}

#[test]
fn test_simple_construction() {
    let mut g = DAG::new();
    let h0 = g.new_node();
    g.verify();

    let h1 = g.new_node();
    let h2 = g.new_node();
    let h3 = g.new_node();
    let h4 = g.new_node();

    assert_ne!(h0, h1);
    assert_ne!(h1, h2);

    g.add_edge(h0, h1);
    g.add_edge(h1, h2);
    g.add_edge(h0, h2);
    g.add_edge(h2, h3);
    g.add_edge(h3, h4);

    g.verify();

    let order = g.topological_sort();
    let levels = g.compute_levels(&order);
    assert_eq!(order.len(), g.len());
    assert_eq!(levels.len(), g.len());

    for i in 0..g.len() {
        println!("{}) node {},  level {}", i, order[i].idx, levels[i]);
    }
}

#[test]
fn test_rank_api() {
    let mut g = DAG::new();
    let h0 = g.new_node();
    let h1 = g.new_node();
    let h2 = g.new_node();

    g.add_edge(h0, h1);
    g.add_edge(h1, h2);

    g.recompute_node_ranks();
    g.verify();

    assert_eq!(g.level(h0), 0);
    assert_eq!(g.level(h1), 1);
    assert_eq!(g.level(h2), 2);

    let r1 = g.remove_edge(h0, h1);
    let r2 = g.remove_edge(h0, h1);
    // Should be able to remove the edge that we inserted.
    assert!(r1);
    // The edge should no longer be there!
    assert!(!r2);
}
