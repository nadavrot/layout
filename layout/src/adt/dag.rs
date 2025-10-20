//! This module implements the Ranked-DAG data structure. I's a data structure
//! that represents the edges between nodes in the dag as well as the leveling
//! of the nodes. A rank is the ordering of some nodes along the x-axis. Users
//! of this data structure may change the leveling of nodes, and the only
//! guarantee is that the nodes are assigned to some level.

use std::cmp;

/// The Ranked-DAG data structure.
#[derive(Debug)]
pub struct DAG {
    /// Nesting Tree for Subgraph Hierarchy
    nesting_tree: NTree,

    /// A list of nodes in the dag.
    nodes: Vec<Node>,

    /// Places nodes in levels.
    ranks: RankType,

    /// levels info
    levels: Vec<usize>,

    /// Perform validation checks.
    validate: bool,

    /// Ranks for top border of subgraphs
    top_border_ranks: Vec<Vec<Vec<SubgraphHandle>>>,

    /// Ranks for bottom border of subgraphs
    bottom_border_ranks: Vec<Vec<Vec<SubgraphHandle>>>,
}

#[derive(Debug)]
struct NTree {
    subgraphs: Vec<Subgraph>,
}

#[derive(Debug)]
struct Subgraph {
    nodes: Vec<NodeHandle>,
    parent_subgraph_idx: SubgraphHandle,
    subgraphs: Vec<SubgraphHandle>,
    left_borders: Vec<NodeHandle>,
    right_borders: Vec<NodeHandle>,
}

#[derive(Debug)]
struct Node {
    // Points to other edges.
    successors: Vec<NodeHandle>,
    predecessors: Vec<NodeHandle>,
    parent_subgraph_idx: SubgraphHandle,
    node_type: NodeType,
}

#[derive(Debug, PartialEq)]
enum NodeType {
    Regular,
    Connector,
    VerticalBorder,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub(crate) enum ContainerHandle {
    Node(NodeHandle),
    Subgraph(SubgraphHandle),
}

#[derive(Copy, Clone, Default, PartialEq, PartialOrd, Eq, Ord, Hash, Debug)]
pub struct SubgraphHandle {
    pub idx: usize,
}

/// Used by users to keep track of nodes that are saved in the DAG.
#[derive(Copy, Clone, Default, PartialEq, PartialOrd, Eq, Ord, Hash, Debug)]
pub struct NodeHandle {
    idx: usize,
}

pub type RankType = Vec<Vec<NodeHandle>>;

impl NTree {
    pub fn new() -> Self {
        NTree {
            subgraphs: vec![Subgraph::new(SubgraphHandle::new(0))],
        }
    }

    pub(crate) fn new_subgraph(
        &mut self,
        subgraph: Subgraph,
        parent_subgraph_idx: SubgraphHandle,
    ) -> SubgraphHandle {
        let idx = self.subgraphs.len();
        self.subgraphs.push(subgraph);
        if idx == parent_subgraph_idx.get_index() {
            return SubgraphHandle { idx };
        }
        self.subgraphs[parent_subgraph_idx.get_index()]
            .subgraphs
            .push(SubgraphHandle::new(idx));
        SubgraphHandle { idx }
    }
}

impl Subgraph {
    pub(crate) fn new(parent_subgraph_idx: SubgraphHandle) -> Self {
        Subgraph {
            nodes: Vec::new(),
            parent_subgraph_idx,
            subgraphs: Vec::new(),
            left_borders: Vec::new(),
            right_borders: Vec::new(),
        }
    }
}

impl SubgraphHandle {
    pub fn new(idx: usize) -> Self {
        SubgraphHandle { idx }
    }

    pub fn get_index(&self) -> usize {
        self.idx
    }
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

impl Node {
    pub fn new(
        parent_subgraph_idx: SubgraphHandle,
        node_type: NodeType,
    ) -> Self {
        Node {
            successors: Vec::new(),
            predecessors: Vec::new(),
            parent_subgraph_idx,
            node_type,
        }
    }

    pub fn get_parent_subgraph_index(&self) -> SubgraphHandle {
        self.parent_subgraph_idx
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

enum VerticalBorder {
    Left,
    Right,
}

impl DAG {
    pub fn new() -> Self {
        DAG {
            nesting_tree: NTree::new(),
            nodes: Vec::new(),
            ranks: Vec::new(),
            levels: Vec::new(),
            top_border_ranks: Vec::new(),
            bottom_border_ranks: Vec::new(),
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

    /// Create a new subgraph.
    pub(crate) fn new_subgraph(
        &mut self,
        parent_subgraph_idx: SubgraphHandle,
    ) -> SubgraphHandle {
        let subgraph = Subgraph::new(parent_subgraph_idx);
        let subgraph_idx =
            SubgraphHandle::new(self.nesting_tree.subgraphs.len());
        self.nesting_tree
            .new_subgraph(subgraph, parent_subgraph_idx);
        subgraph_idx
    }

    pub(crate) fn subgraph_left_borders(
        &self,
        sg: SubgraphHandle,
    ) -> &Vec<NodeHandle> {
        &self.nesting_tree.subgraphs[sg.idx].left_borders
    }

    pub(crate) fn subgraph_right_borders(
        &self,
        sg: SubgraphHandle,
    ) -> &Vec<NodeHandle> {
        &self.nesting_tree.subgraphs[sg.idx].right_borders
    }

    /// Create a new node.
    pub fn new_node(
        &mut self,
        parent_subgraph_idx: SubgraphHandle,
    ) -> NodeHandle {
        self.nodes
            .push(Node::new(parent_subgraph_idx, NodeType::Regular));
        self.levels.push(0);
        let node = NodeHandle::new(self.nodes.len() - 1);
        self.nesting_tree.subgraphs[parent_subgraph_idx.idx]
            .nodes
            .push(node);
        self.add_element_to_rank(node, 0, false);
        node
    }

    pub(crate) fn new_connector_node(
        &mut self,
        from: NodeHandle,
        to: NodeHandle,
    ) -> NodeHandle {
        let from_c = self.nesting_edge_pair(from, to).0;
        let connector_sg_idx = match from_c {
            ContainerHandle::Node(n) => self.get_parent_subgraph_index_n(n),
            ContainerHandle::Subgraph(sg) => {
                self.get_parent_subgraph_index_sg(sg)
            }
        };
        self.nodes
            .push(Node::new(connector_sg_idx, NodeType::Connector));
        self.levels.push(0);
        let node = NodeHandle::new(self.nodes.len() - 1);
        self.nesting_tree.subgraphs[connector_sg_idx.idx]
            .nodes
            .push(node);
        self.add_element_to_rank(node, 0, false);
        node
    }

    /// Create \p n new nodes.
    pub fn new_nodes(&mut self, n: usize) {
        for _ in 0..n {
            self.nodes
                .push(Node::new(SubgraphHandle::new(0), NodeType::Regular));
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

    pub fn get_parent_subgraph_index_n(
        &self,
        from: NodeHandle,
    ) -> SubgraphHandle {
        self.nodes[from.idx].get_parent_subgraph_index()
    }

    pub fn get_parent_subgraph_index_sg(
        &self,
        from: SubgraphHandle,
    ) -> SubgraphHandle {
        self.nesting_tree.subgraphs[from.idx].parent_subgraph_idx
    }

    pub fn get_children_subgraphs(
        &self,
        from: SubgraphHandle,
    ) -> &Vec<SubgraphHandle> {
        &self.nesting_tree.subgraphs[from.idx].subgraphs
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

    pub fn is_vertical_border(&self, node: NodeHandle) -> bool {
        match self.nodes[node.idx].node_type {
            NodeType::VerticalBorder => true,
            _ => false,
        }
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

    pub(crate) fn get_subgraph_levels(&self) -> Vec<(usize, usize)> {
        let mut levels = vec![(usize::MAX, usize::MIN); self.num_subgraphs()];
        for (i, node) in self.nodes.iter().enumerate() {
            let subgraph_idx = node.get_parent_subgraph_index();
            let level = self.level(NodeHandle::from(i));
            levels[subgraph_idx.idx].0 =
                cmp::min(levels[subgraph_idx.idx].0, level);
            levels[subgraph_idx.idx].1 =
                cmp::max(levels[subgraph_idx.idx].1, level);
        }

        for (i, s) in self.nesting_tree.subgraphs.iter().enumerate().rev() {
            for child in s.subgraphs.iter() {
                levels[i].0 = cmp::min(levels[i].0, levels[child.idx].0);
                levels[i].1 = cmp::max(levels[i].1, levels[child.idx].1);
            }
        }
        for (i, s) in self.nesting_tree.subgraphs.iter().enumerate() {
            let parent_idx = s.parent_subgraph_idx;
            if levels[i].0 > levels[i].1 {
                // No nodes in this subgraph.
                levels[i].0 = levels[parent_idx.idx].0;
                levels[i].1 = levels[parent_idx.idx].0;
            }
        }
        levels
    }

    fn is_reachable_inner_c(
        &self,
        from: ContainerHandle,
        to: ContainerHandle,
        visited_nodes: &mut Vec<bool>,
        visited_subgraphs: &mut Vec<bool>,
        node_successors_c: &Vec<Vec<ContainerHandle>>,
        subgraph_successors_c: &Vec<Vec<ContainerHandle>>,
    ) -> bool {
        if from == to {
            return true;
        }

        match from {
            ContainerHandle::Node(n) => {
                // Don't step into a cycle.
                if visited_nodes[n.idx] {
                    return false;
                }

                // Push to the dfs stack.
                visited_nodes[n.idx] = true;

                for edge in &node_successors_c[n.idx] {
                    if self.is_reachable_inner_c(
                        edge.clone(),
                        to.clone(),
                        visited_nodes,
                        visited_subgraphs,
                        node_successors_c,
                        subgraph_successors_c,
                    ) {
                        return true;
                    }
                }

                // Pop from the dfs stack.
                visited_nodes[n.idx] = false;
            }
            ContainerHandle::Subgraph(s) => {
                // Don't step into a cycle.
                if visited_subgraphs[s.idx] {
                    return false;
                }

                // Push to the dfs stack.
                visited_subgraphs[s.idx] = true;

                for edge in &subgraph_successors_c[s.idx] {
                    if self.is_reachable_inner_c(
                        edge.clone(),
                        to.clone(),
                        visited_nodes,
                        visited_subgraphs,
                        node_successors_c,
                        subgraph_successors_c,
                    ) {
                        return true;
                    }
                }

                // Pop from the dfs stack.
                visited_subgraphs[s.idx] = false;
            }
        }

        false
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

    fn is_reachable_c(
        &self,
        from: ContainerHandle,
        to: ContainerHandle,
        node_successors_c: &Vec<Vec<ContainerHandle>>,
        subgraph_successors_c: &Vec<Vec<ContainerHandle>>,
    ) -> bool {
        if from == to {
            return true;
        }
        let mut visited_nodes = vec![false; self.nodes.len()];
        let mut visited_subgraphs =
            vec![false; self.nesting_tree.subgraphs.len()];

        self.is_reachable_inner_c(
            from,
            to,
            &mut visited_nodes,
            &mut visited_subgraphs,
            node_successors_c,
            subgraph_successors_c,
        )
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

    fn topological_sort_container(
        &self,
        subgraph_idx: SubgraphHandle,
        node_successors_c: &Vec<Vec<ContainerHandle>>,
        subgraph_successors_c: &Vec<Vec<ContainerHandle>>,
    ) -> Vec<ContainerHandle> {
        // A list of vectors in post-order.
        let mut order: Vec<ContainerHandle> = Vec::new();

        // Marks that a node is in the worklist.
        let mut visited_nodes = vec![false; self.nodes.len()];
        let mut visited_subgraphs =
            vec![false; self.nesting_tree.subgraphs.len()];

        // A tuple of handle, and command:
        // true- force push.
        // false- this is a child to visit.
        let mut worklist: Vec<(ContainerHandle, bool)> = Vec::new();

        // Add all of the values that we want to compute into the worklist.
        for n in self.nesting_tree.subgraphs[subgraph_idx.idx].nodes.iter() {
            worklist.push((ContainerHandle::Node(*n), false));
        }
        for s in self.nesting_tree.subgraphs[subgraph_idx.idx]
            .subgraphs
            .iter()
        {
            worklist.push((ContainerHandle::Subgraph(*s), false));
        }
        while let Some((current, cmd)) = worklist.pop() {
            // Handle 'push' commands.
            if cmd {
                order.push(current);
                continue;
            }

            match current {
                ContainerHandle::Node(n) => {
                    // Don't visit visited nodes.
                    if visited_nodes[n.idx] {
                        continue;
                    }

                    visited_nodes[n.idx] = true;

                    // Save this node after all of the children are handles.
                    worklist.push((current, true));

                    // Add the children to the worklist.
                    let succs = node_successors_c[n.idx].clone();

                    for edge in succs.iter() {
                        worklist.push((edge.clone(), false));
                    }
                }
                ContainerHandle::Subgraph(s) => {
                    if visited_subgraphs[s.idx] {
                        continue;
                    }
                    visited_subgraphs[s.idx] = true;
                    worklist.push((current, true));
                    let succs = subgraph_successors_c[s.idx].clone();
                    for edge in succs.iter() {
                        worklist.push((edge.clone(), false));
                    }
                }
            }
        }

        order.reverse();
        order
    }

    fn compute_successors_c(
        &self,
    ) -> (
        Vec<Vec<ContainerHandle>>,
        Vec<Vec<ContainerHandle>>,
        Vec<Vec<(NodeHandle, NodeHandle)>>,
        Vec<Vec<(NodeHandle, NodeHandle)>>,
    ) {
        let mut node_successors_c = vec![Vec::new(); self.nodes.len()];
        let mut subgraph_successors_c =
            vec![Vec::new(); self.nesting_tree.subgraphs.len()];
        let mut node_successors_pairs = vec![Vec::new(); self.nodes.len()];
        let mut subgraph_successors_pairs =
            vec![Vec::new(); self.nesting_tree.subgraphs.len()];

        for (i, node) in self.nodes.iter().enumerate() {
            for succ in node.successors.iter() {
                let (from_c, to_c) =
                    self.nesting_edge_pair(NodeHandle::new(i), *succ);

                if self.is_reachable_c(
                    to_c.clone(),
                    from_c.clone(),
                    &node_successors_c,
                    &subgraph_successors_c,
                ) {
                    continue;
                }
                match from_c {
                    ContainerHandle::Node(n) => {
                        node_successors_c[n.idx].push(to_c);
                        node_successors_pairs[n.idx]
                            .push((NodeHandle::new(i), *succ));
                    }
                    ContainerHandle::Subgraph(s) => {
                        subgraph_successors_c[s.idx].push(to_c);
                        subgraph_successors_pairs[s.idx]
                            .push((NodeHandle::new(i), *succ));
                    }
                }
            }
        }
        (
            node_successors_c,
            subgraph_successors_c,
            node_successors_pairs,
            subgraph_successors_pairs,
        )
    }

    pub fn compute_level_container(&self) -> Vec<usize> {
        let mut levels: Vec<usize> = Vec::new();

        // Levels has the same layout as the DAG node list.
        levels.resize(self.nodes.len(), 0);

        let (node_successors_c, subgraph_successors_c, node_s_pair, sg_s_pair) =
            self.compute_successors_c();
        let subgraphs_num = self.num_subgraphs();
        for s_idx in (0..subgraphs_num).rev() {
            let order_c = self.topological_sort_container(
                SubgraphHandle::new(s_idx),
                &node_successors_c,
                &subgraph_successors_c,
            );
            for t in order_c {
                let (edge_pairs, successors_c) = match t {
                    ContainerHandle::Node(from) => {
                        (&node_s_pair[from.idx], &node_successors_c[from.idx])
                    }
                    ContainerHandle::Subgraph(from_s) => (
                        &sg_s_pair[from_s.idx],
                        &subgraph_successors_c[from_s.idx],
                    ),
                };
                for (j, to_t) in successors_c.iter().enumerate() {
                    let (from, to) = edge_pairs[j];
                    self.level_offset_container(from, to, *to_t, &mut levels);
                }
            }
        }
        // assign connectors to be average of their end nodes
        for (i, node) in self.nodes.iter().enumerate() {
            if node.node_type == NodeType::Connector {
                let pred = self.single_pred(NodeHandle::from(i)).unwrap();
                let succ = self.single_succ(NodeHandle::from(i)).unwrap();
                // round to nearest integer
                let level = (levels[pred.idx] + levels[succ.idx]) / 2;
                levels[i] = level;
            }
        }
        levels
    }

    // The methods below are related to the rank (placing nodes in levels). //

    /// \returns the number of ranks in the dag.
    pub fn num_levels(&self) -> usize {
        self.ranks.len()
    }

    pub fn num_subgraphs(&self) -> usize {
        self.nesting_tree.subgraphs.len()
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

    /// \returns a reference to the whole border ranks data structure.
    pub(crate) fn top_border_ranks(&self) -> &Vec<Vec<Vec<SubgraphHandle>>> {
        &self.top_border_ranks
    }

    /// \returns a reference to the whole border ranks data structure.
    pub(crate) fn bottom_border_ranks(&self) -> &Vec<Vec<Vec<SubgraphHandle>>> {
        &self.bottom_border_ranks
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
        let levels = self.compute_level_container();
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
    }

    fn level_offset_container(
        &self,
        from: NodeHandle,
        to: NodeHandle,
        to_t: ContainerHandle,
        levels: &mut Vec<usize>,
    ) {
        match to_t {
            ContainerHandle::Node(n) => {
                levels[n.idx] = cmp::max(levels[n.idx], levels[from.idx] + 1);
            }
            ContainerHandle::Subgraph(sg) => {
                if levels[from.idx] + 1 <= levels[to.idx] {
                    return;
                }
                let offset = (levels[from.idx] + 1) - levels[to.idx];
                for n in self.nesting_tree.subgraphs[sg.idx].nodes.iter() {
                    levels[n.idx] += offset;
                }
                let mut worklist = Vec::new();
                for s in self.nesting_tree.subgraphs[sg.idx].subgraphs.iter() {
                    worklist.push(*s);
                }
                while let Some(current) = worklist.pop() {
                    for n in
                        self.nesting_tree.subgraphs[current.idx].nodes.iter()
                    {
                        levels[n.idx] += offset;
                    }
                    for s in self.nesting_tree.subgraphs[current.idx]
                        .subgraphs
                        .iter()
                    {
                        worklist.push(*s);
                    }
                }
            }
        }
    }

    pub(crate) fn place_horizontal_borders(
        &mut self,
        subgraph_levels: &Vec<(usize, usize)>,
    ) {
        let num_levels = self.num_levels();
        self.top_border_ranks = vec![vec![]; num_levels];
        self.bottom_border_ranks = vec![vec![]; num_levels];

        self.top_border_ranks[0] = vec![vec![SubgraphHandle::new(0)]];
        self.bottom_border_ranks[num_levels - 1] =
            vec![vec![SubgraphHandle::new(0)]];

        let mut inner_levels = vec![None; self.num_subgraphs()];

        inner_levels[0] = Some((0, 0));
        for s in 0..self.num_subgraphs() {
            self.new_horizontal_border(
                SubgraphHandle::new(s),
                subgraph_levels,
                &mut inner_levels,
            );
        }
    }

    fn new_horizontal_border(
        &mut self,
        subgraph_idx: SubgraphHandle,
        levels_map: &Vec<(usize, usize)>,
        inner_levels: &mut Vec<Option<(usize, usize)>>,
    ) {
        let (p_lvl_s, p_lvl_e) = levels_map[subgraph_idx.idx];

        let (p_in_lvl_s, p_in_lvl_e) = inner_levels[subgraph_idx.idx].expect(
            "Ordering subgraph horizontal borders is wrong, this is a bug.",
        );

        let mut c_in_lvl_s;
        let mut c_in_lvl_e;

        for child_s in self.nesting_tree.subgraphs[subgraph_idx.idx]
            .subgraphs
            .iter()
        {
            let (lvl_s, lvl_e) = levels_map[child_s.idx];
            if lvl_s == p_lvl_s {
                if p_in_lvl_s + 1 < self.top_border_ranks[lvl_s].len() {
                    self.top_border_ranks[lvl_s][p_in_lvl_s + 1].push(*child_s);
                } else {
                    self.top_border_ranks[lvl_s].push(vec![*child_s]);
                }

                c_in_lvl_s = p_in_lvl_s + 1;
            } else {
                if self.top_border_ranks[lvl_s].is_empty() {
                    self.top_border_ranks[lvl_s].push(vec![*child_s]);
                    c_in_lvl_s = 0;
                } else {
                    self.top_border_ranks[lvl_s]
                        .last_mut()
                        .unwrap()
                        .push(*child_s);
                    c_in_lvl_s = self.top_border_ranks[lvl_s].len() - 1;
                }
            }

            if lvl_e == p_lvl_e {
                if p_in_lvl_e + 1 < self.top_border_ranks[lvl_e].len() {
                    self.top_border_ranks[lvl_e][p_in_lvl_e + 1].push(*child_s);
                } else {
                    self.bottom_border_ranks[lvl_e].push(vec![*child_s]);
                }
                c_in_lvl_e = p_in_lvl_e + 1;
            } else {
                if self.bottom_border_ranks[lvl_e].is_empty() {
                    self.bottom_border_ranks[lvl_e].push(vec![*child_s]);
                    c_in_lvl_e = 0;
                } else {
                    self.bottom_border_ranks[lvl_e]
                        .last_mut()
                        .unwrap()
                        .push(*child_s);
                    c_in_lvl_e = self.bottom_border_ranks[lvl_e].len() - 1;
                }
            }

            inner_levels[child_s.idx] = Some((c_in_lvl_s, c_in_lvl_e));
        }
    }

    fn get_tree_path(
        &self,
        start_sg: SubgraphHandle,
        end_sg: SubgraphHandle,
    ) -> (Vec<SubgraphHandle>, SubgraphHandle, Vec<SubgraphHandle>) {
        let from_path = self.subgraph_path(start_sg);
        let to_path = self.subgraph_path(end_sg);
        let max_len = from_path.len().min(to_path.len());
        let mut i = 0;
        while i < max_len {
            if from_path[i] == to_path[i] {
                i += 1;
            } else {
                break;
            }
        }
        let from_path = from_path.iter().skip(i).rev().cloned().collect();
        (from_path, to_path[i - 1], to_path[i..].to_vec())
    }

    fn new_vertical_border_node(
        &mut self,
        sg: SubgraphHandle,
        level: usize,
        border_type: VerticalBorder,
    ) -> NodeHandle {
        self.nodes.push(Node::new(sg, NodeType::VerticalBorder));
        let node_handle = NodeHandle::new(self.nodes.len() - 1);
        self.nesting_tree.subgraphs[sg.idx].nodes.push(node_handle);
        self.levels.push(level);
        self.ranks[level].push(node_handle);
        match border_type {
            VerticalBorder::Left => self.nesting_tree.subgraphs[sg.idx]
                .left_borders
                .push(node_handle),
            VerticalBorder::Right => self.nesting_tree.subgraphs[sg.idx]
                .right_borders
                .push(node_handle),
        }
        node_handle
    }

    pub(crate) fn place_vertical_borders(&mut self) {
        let max_level = self.num_levels();
        for ri in 0..max_level {
            let row_len = self.ranks[ri].len();
            let old_row = self.ranks[ri].clone();
            self.ranks[ri].clear();
            let sg_path = self
                .subgraph_path(self.get_parent_subgraph_index_n(old_row[0]));
            for s in sg_path.iter() {
                self.new_vertical_border_node(*s, ri, VerticalBorder::Left);
            }
            for i in 0..(row_len - 1) {
                self.ranks[ri].push(old_row[i]);
                let curr_sg = self.get_parent_subgraph_index_n(old_row[i]);
                let next_sg = self.get_parent_subgraph_index_n(old_row[i + 1]);
                if curr_sg == next_sg {
                    continue;
                }
                let (from_path, _, to_path) =
                    self.get_tree_path(curr_sg, next_sg);
                for s in from_path.iter() {
                    self.new_vertical_border_node(
                        *s,
                        ri,
                        VerticalBorder::Right,
                    );
                }

                for s in to_path.iter() {
                    self.new_vertical_border_node(*s, ri, VerticalBorder::Left);
                }
            }
            self.ranks[ri].push(old_row[row_len - 1]);
            let sg_path = self.subgraph_path(
                self.get_parent_subgraph_index_n(old_row[row_len - 1]),
            );
            for s in sg_path.iter().rev() {
                self.new_vertical_border_node(*s, ri, VerticalBorder::Right);
            }

            // go trhough left and right borders and connect them
            for s in 0..self.num_subgraphs() {
                let border_len =
                    self.nesting_tree.subgraphs[s].right_borders.len();
                for i in 1..border_len {
                    self.add_edge(
                        self.nesting_tree.subgraphs[s].left_borders[i - 1],
                        self.nesting_tree.subgraphs[s].left_borders[i],
                    );
                    self.add_edge(
                        self.nesting_tree.subgraphs[s].right_borders[i - 1],
                        self.nesting_tree.subgraphs[s].right_borders[i],
                    );
                }
            }
        }
        // assert right and left border length are equal for each subgraph
        for s in 0..self.num_subgraphs() {
            assert_eq!(
                self.nesting_tree.subgraphs[s].left_borders.len(),
                self.nesting_tree.subgraphs[s].right_borders.len(),
                "Subgraph {:?} has unbalanced borders",
                SubgraphHandle::new(s)
            );
        }
    }

    fn barycenter_weight_pred(
        &self,
        node: NodeHandle,
        col_map: &Vec<usize>,
    ) -> f64 {
        let mut total_weight = 0;
        let total_count = self.predecessors(node).len();
        for p in self.predecessors(node).iter() {
            total_weight += col_map[p.get_index()];
        }

        if total_count == 0 {
            return col_map[node.get_index()] as f64;
        }
        total_weight as f64 / total_count as f64
    }

    fn barycenter_weight_succ(
        &self,
        node: NodeHandle,
        col_map: &Vec<usize>,
    ) -> f64 {
        let mut total_weight = 0;
        let total_count = self.successors(node).len();
        for p in self.successors(node).iter() {
            total_weight += col_map[p.get_index()];
        }

        if total_count == 0 {
            return col_map[node.get_index()] as f64;
        }
        total_weight as f64 / total_count as f64
    }

    fn find_endpoints_of_connectors(
        &self,
        node: NodeHandle,
    ) -> (NodeHandle, NodeHandle) {
        let mut pred = node;
        let mut succ = node;
        assert!(
            self.nodes[node.idx].node_type == NodeType::Connector,
            "Node {:?} is not a connector node",
            node
        );

        while self.nodes[pred.idx].node_type == NodeType::Connector {
            let p = self
                .single_pred(pred)
                .expect("Connector node has no predecessor, this is a bug.");
            pred = p;
        }

        while self.nodes[succ.idx].node_type == NodeType::Connector {
            let s = self
                .single_succ(succ)
                .expect("Connector node has no successor, this is a bug.");
            succ = s;
        }

        (pred, succ)
    }

    fn aggregate_position(&self) -> (Vec<f64>, Vec<f64>) {
        let subgraphs_num = self.num_subgraphs();
        let mut total_position = vec![(0, 0); subgraphs_num];
        let mut position_nodes = vec![0.0; self.len()];

        for row in self.ranks.iter() {
            for (j, node) in row.iter().enumerate() {
                let subgraph_idx =
                    self.nodes[node.idx].get_parent_subgraph_index();
                total_position[subgraph_idx.idx].0 += j;
                total_position[subgraph_idx.idx].1 += 1;
                position_nodes[node.idx] = j as f64;
            }
        }

        for i in (0..subgraphs_num).rev() {
            for j in self.nesting_tree.subgraphs[i].subgraphs.iter() {
                total_position[i].0 += total_position[j.idx].0;
                total_position[i].1 += total_position[j.idx].1;
            }
        }

        let average_position: Vec<f64> = total_position
            .into_iter()
            .map(|(total, count)| {
                if count == 0 {
                    0.0
                } else {
                    total as f64 / count as f64
                }
            })
            .collect();

        for (i, node) in self.nodes.iter().enumerate() {
            if node.node_type == NodeType::Connector {
                let (pred_n, succ_n) =
                    self.find_endpoints_of_connectors(NodeHandle::new(i));
                let (pred, succ) = self.nesting_edge_pair(pred_n, succ_n);
                let p_pos = match pred {
                    ContainerHandle::Node(n) => position_nodes[n.idx],
                    ContainerHandle::Subgraph(s) => average_position[s.idx],
                };
                let s_pos = match succ {
                    ContainerHandle::Node(n) => position_nodes[n.idx],
                    ContainerHandle::Subgraph(s) => average_position[s.idx],
                };
                position_nodes[i] = (p_pos + s_pos) / 2.0;
            }
        }

        (average_position, position_nodes)
    }

    fn aggregate_position_layerwise(
        &self,
        layer_idx: usize,
    ) -> (Vec<Option<f64>>, Vec<Option<f64>>) {
        let subgraphs_num = self.num_subgraphs();

        let mut total_position = vec![(0, 0); subgraphs_num];
        let mut position_nodes = vec![None; self.len()];
        for (j, node) in self.ranks[layer_idx].iter().enumerate() {
            let subgraph_idx = self.nodes[node.idx].get_parent_subgraph_index();
            total_position[subgraph_idx.idx].0 += j;
            total_position[subgraph_idx.idx].1 += 1;
            position_nodes[node.idx] = Some(j as f64);
        }

        for i in 0..subgraphs_num {
            for j in self.nesting_tree.subgraphs[subgraphs_num - i - 1]
                .subgraphs
                .iter()
            {
                total_position[subgraphs_num - i - 1].0 +=
                    total_position[j.idx].0;
                total_position[subgraphs_num - i - 1].1 +=
                    total_position[j.idx].1;
            }
        }

        let average_position: Vec<Option<f64>> = total_position
            .into_iter()
            .map(|(total, count)| {
                if count == 0 {
                    None
                } else {
                    Some(total as f64 / count as f64)
                }
            })
            .collect();

        (average_position, position_nodes)
    }

    pub(crate) fn subgraph_order_by_p_layerwise(&mut self, layer_idx: usize) {
        let (subgraph_pos_map, node_pos) =
            self.aggregate_position_layerwise(layer_idx);

        let mut working_list =
            vec![ContainerHandle::Subgraph(SubgraphHandle::new(0))];
        let mut output = vec![];

        while !working_list.is_empty() {
            let current = working_list.pop().unwrap();
            match current {
                ContainerHandle::Node(n) => {
                    output.push(n);
                }
                ContainerHandle::Subgraph(s) => {
                    let mut cur = vec![];
                    for s_child in
                        self.nesting_tree.subgraphs[s.idx].subgraphs.iter()
                    {
                        if let Some(_) = subgraph_pos_map[s_child.idx] {
                            cur.push(ContainerHandle::Subgraph(*s_child));
                        }
                    }

                    for n_child in
                        self.nesting_tree.subgraphs[s.idx].nodes.iter()
                    {
                        if let Some(_) = node_pos[n_child.idx] {
                            cur.push(ContainerHandle::Node(*n_child));
                        }
                    }

                    cur.sort_by(|a, b| {
                        let p_a = match a {
                            ContainerHandle::Node(n) => {
                                node_pos[n.idx].unwrap()
                            }
                            ContainerHandle::Subgraph(sg) => {
                                subgraph_pos_map[sg.idx].unwrap()
                            }
                        };
                        let p_b = match b {
                            ContainerHandle::Node(n) => {
                                node_pos[n.idx].unwrap()
                            }
                            ContainerHandle::Subgraph(sg) => {
                                subgraph_pos_map[sg.idx].unwrap()
                            }
                        };
                        p_b.partial_cmp(&p_a).unwrap()
                    });

                    for c in cur.into_iter() {
                        working_list.push(c);
                    }
                }
            }
        }

        self.ranks[layer_idx] = output;
    }

    pub(crate) fn subgraph_order_by_p(&mut self) -> Vec<NodeHandle> {
        let (subgraph_pos_map, node_pos) = self.aggregate_position();
        let mut working_list =
            vec![ContainerHandle::Subgraph(SubgraphHandle::new(0))];
        let mut output = vec![];

        while !working_list.is_empty() {
            let current = working_list.pop().unwrap();
            match current {
                ContainerHandle::Node(n) => {
                    output.push(n);
                }
                ContainerHandle::Subgraph(s) => {
                    let mut cur = vec![];
                    for s_child in
                        self.nesting_tree.subgraphs[s.idx].subgraphs.iter()
                    {
                        cur.push(ContainerHandle::Subgraph(*s_child));
                    }
                    for n_child in
                        self.nesting_tree.subgraphs[s.idx].nodes.iter()
                    {
                        cur.push(ContainerHandle::Node(*n_child));
                    }
                    cur.sort_by(|a, b| {
                        let p_a = match a {
                            ContainerHandle::Node(n) => node_pos[n.idx],
                            ContainerHandle::Subgraph(sg) => {
                                subgraph_pos_map[sg.idx]
                            }
                        };
                        let p_b = match b {
                            ContainerHandle::Node(n) => node_pos[n.idx],
                            ContainerHandle::Subgraph(sg) => {
                                subgraph_pos_map[sg.idx]
                            }
                        };
                        p_b.partial_cmp(&p_a).unwrap()
                    });

                    for c in cur.into_iter() {
                        working_list.push(c);
                    }
                }
            }
        }
        output
    }

    pub(crate) fn node_order_ws_layerwise(
        &mut self,
        layer_idx: usize,
        col_map: &Vec<usize>,
    ) {
        // using barycenter weights to order nodes in the layer
        let mut result = Vec::new();
        let mut node_weights = Vec::new();
        for node in self.row(layer_idx).iter() {
            let weight = self.barycenter_weight_succ(*node, col_map);
            node_weights.push((weight, *node));
        }
        // sort by weight
        node_weights.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        for (_, node) in node_weights.iter() {
            result.push(*node);
        }
        self.ranks[layer_idx] = result;
    }

    pub(crate) fn node_order_wp_layerwise(
        &mut self,
        layer_idx: usize,
        col_map: &Vec<usize>,
    ) {
        // using barycenter weights to order nodes in the layer
        let mut result = Vec::new();
        let mut node_weights = Vec::new();
        for node in self.row(layer_idx).iter() {
            let weight = self.barycenter_weight_pred(*node, col_map);
            node_weights.push((weight, *node));
        }
        // sort by weight
        node_weights.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        for (_, node) in node_weights.iter() {
            result.push(*node);
        }
        self.ranks[layer_idx] = result;
    }

    fn subgraph_path(&self, sg: SubgraphHandle) -> Vec<SubgraphHandle> {
        let mut path = vec![sg];
        let mut current = sg;
        while current.idx != 0 {
            current =
                self.nesting_tree.subgraphs[current.idx].parent_subgraph_idx;
            path.push(current);
        }
        path.reverse();
        path
    }

    fn nesting_edge_pair(
        &self,
        from: NodeHandle,
        to: NodeHandle,
    ) -> (ContainerHandle, ContainerHandle) {
        let from_path =
            self.subgraph_path(self.get_parent_subgraph_index_n(from));
        let to_path = self.subgraph_path(self.get_parent_subgraph_index_n(to));
        let max_len = from_path.len().min(to_path.len());
        let mut i = 0;
        while i < max_len {
            if from_path[i] != to_path[i] {
                break;
            }
            i += 1;
        }
        let from_t = if i == from_path.len() {
            ContainerHandle::Node(from)
        } else {
            ContainerHandle::Subgraph(from_path[i])
        };

        let to_t = if i == to_path.len() {
            ContainerHandle::Node(to)
        } else {
            ContainerHandle::Subgraph(to_path[i])
        };
        (from_t, to_t)
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
    let subgraph_idx = SubgraphHandle::new(0);
    let h0 = g.new_node(subgraph_idx);
    g.verify();

    let h1 = g.new_node(subgraph_idx);
    let h2 = g.new_node(subgraph_idx);
    let h3 = g.new_node(subgraph_idx);
    let h4 = g.new_node(subgraph_idx);

    assert_ne!(h0, h1);
    assert_ne!(h1, h2);

    g.add_edge(h0, h1);
    g.add_edge(h1, h2);
    g.add_edge(h0, h2);
    g.add_edge(h2, h3);
    g.add_edge(h3, h4);

    g.verify();

    let levels = g.compute_level_container();
    assert_eq!(levels.len(), g.len());

    for i in 0..g.len() {
        println!("{}),  level {}", i, levels[i]);
    }
}

#[test]
fn test_rank_api() {
    let mut g = DAG::new();
    let subgraph_idx = SubgraphHandle::new(0);
    let h0 = g.new_node(subgraph_idx);
    let h1 = g.new_node(subgraph_idx);
    let h2 = g.new_node(subgraph_idx);

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
