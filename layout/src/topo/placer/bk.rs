//! This module implements block placement that's based on the Brandes and Kopf
//! paper "Fast and Simple Horizontal Coordinate Assignment."

use crate::adt::dag::NodeHandle;
use crate::core::geometry::weighted_median;
use crate::topo::layout::VisualGraph;
use std::collections::HashSet;

use super::simple;

#[derive(Debug, Clone, Copy)]
enum OrderLR {
    LeftToRight,
    RightToLeft,
}

impl OrderLR {
    pub fn is_left_to_right(&self) -> bool {
        match self {
            OrderLR::LeftToRight => true,
            OrderLR::RightToLeft => false,
        }
    }
}

/// Maps the block alignment information.
struct NodeAttachInfo {
    /// For each node, marks which node in the row above it aligns to.
    above: Vec<Option<NodeHandle>>,
    /// For each node, marks which node in the row below aligns to it.
    below: Vec<Option<NodeHandle>>,
}

impl NodeAttachInfo {
    pub fn new(size: usize) -> Self {
        let above = vec![None; size];
        let below = vec![None; size];
        Self { above, below }
    }

    /// Align the node \p from to \p to.
    pub fn add(&mut self, from: NodeHandle, to: NodeHandle) {
        assert!(self.below(to).is_none(), "Node is already taken");
        assert!(self.above(from).is_none(), "Node is already set");
        self.above[from.get_index()] = Some(to);
        self.below[to.get_index()] = Some(from);
    }

    /// \returns the node that this node attaches to.
    pub fn above(&self, node: NodeHandle) -> Option<NodeHandle> {
        self.above[node.get_index()]
    }
    /// \returns the node that attaches to this node.
    pub fn below(&self, node: NodeHandle) -> Option<NodeHandle> {
        self.below[node.get_index()]
    }

    /// Extract a list of vertical nodes. This method will insert all of the
    /// nodes in the graph to some vertical list of nodes based on the
    /// relationship that is expressed in this data-structure.
    pub fn get_verticals(&mut self) -> VerticalList {
        // The list of constructed verticals.
        let mut res = VerticalList::new();
        // The list of used nodes.
        let mut used: Vec<bool> = vec![false; self.above.len()];

        // For each node in the graph:
        for i in 0..self.above.len() {
            let mut vertical: Vec<NodeHandle> = Vec::new();

            // Don't visit visited nodes.
            if used[i] {
                continue;
            }

            // Find the bottom of the vertical:
            let mut idx = i;
            while self.below[idx].is_some() {
                idx = self.below[idx].unwrap().get_index();
            }

            // Go up the vertical and save the nodes into the vector.
            vertical.push(NodeHandle::from(idx));
            assert!(self.below[idx].is_none(), "expected to be at the bottom!");
            while self.above[idx].is_some() && !used[idx] {
                // Wipe out the node so it won't participate in future verticals.
                used[idx] = true;

                // Move up to the next node.
                idx = self.above[idx].unwrap().get_index();

                // Add the node to the vertical.
                vertical.push(NodeHandle::from(idx));
            }
            used[idx] = true;
            res.push(vertical);
        }

        res
    }
}

struct Scheduler<'a> {
    vg: &'a VisualGraph,
    vl: VerticalList,
    // Saves the index of the center of each element in the graph.
    x_coordinates: Vec<f64>,
    // For ech row, saves the index of the first unscheduled item.
    sched_idx: Vec<usize>,
    // For ech row, saves the end point of the last box.
    last_x_for_row: Vec<f64>,
    // The node placement order (left to right, or right to left).
    order: OrderLR,
}

impl<'a> Scheduler<'a> {
    pub fn new(vg: &'a VisualGraph, vl: VerticalList, order: OrderLR) -> Self {
        let xs = vec![0.; vg.num_nodes()];
        let idx = vec![0; vg.dag.num_levels()];
        let v = if order.is_left_to_right() {
            f64::NEG_INFINITY
        } else {
            f64::INFINITY
        };
        let last_x_for_row = vec![v; vg.dag.num_levels()];
        Self {
            vg,
            vl,
            x_coordinates: xs,
            sched_idx: idx,
            last_x_for_row,
            order,
        }
    }

    pub fn get_x_placement(&self) -> &Vec<f64> {
        &self.x_coordinates
    }

    pub fn schedule(&mut self) {
        for v in &self.vl {
            self.verify_vertical(v);
        }

        let mut to_place = self.vl.len();

        while to_place > 0 {
            for i in 0..self.vl.len() {
                if !self.is_vertical_ready(i) {
                    continue;
                }
                let v = self.vl[i].clone();
                // Place the nodes.
                let x = self.first_schedule_x(&v);
                if self.order.is_left_to_right() {
                    self.place_vertical(&v, x);
                } else {
                    self.place_vertical(&v, x);
                }
                // Wipe the vertical.
                self.vl[i].clear();
                to_place -= 1;
            }
        }
    }

    // \returns the first possible schedule point.
    pub fn first_schedule_x(&self, v: &Vertical) -> f64 {
        let mut last_offset_x: f64 = 0.;
        for elem in v {
            let level = self.vg.dag.level(*elem);
            let last = self.last_x_for_row[level];
            let pos = self.vg.pos(*elem);

            let offset = if self.order.is_left_to_right() {
                pos.distance_to_left(true)
            } else {
                pos.distance_to_right(true)
            };

            if self.order.is_left_to_right() {
                last_offset_x = last_offset_x.max(last + offset);
            } else {
                last_offset_x = last_offset_x.min(last - offset);
            }
        }
        last_offset_x
    }

    // Place the nodes in the vertical into the schedule.
    pub fn place_vertical(&mut self, v: &Vertical, center_x: f64) {
        for elem in v {
            // Record the x coordinate for the vertical.
            self.x_coordinates[elem.get_index()] = center_x;
            // Update the last x value for the row.
            let level = self.vg.dag.level(*elem);
            let pos = self.vg.pos(*elem);
            if self.order.is_left_to_right() {
                let side_x = pos.distance_to_right(true);
                self.last_x_for_row[level] = center_x + side_x;
            } else {
                let side_x = pos.distance_to_left(true);
                self.last_x_for_row[level] = center_x - side_x;
            }
            self.sched_idx[level] += 1;
        }
    }

    // Make sure that the vertical is legal.
    pub fn verify_vertical(&self, v: &Vertical) {
        let mut prev_level = 0;
        for (i, elem) in v.iter().enumerate() {
            let level = self.vg.dag.level(*elem);
            if i != 0 {
                assert_eq!(level + 1, prev_level);
            }
            prev_level = level;
        }
    }

    /// \returns True if \p node is the next available in the row \p row_idx.
    fn is_next_avail_in_row(&self, node: NodeHandle, row_idx: usize) -> bool {
        let row = self.vg.dag.row(row_idx);
        let first_free = self.sched_idx[row_idx];
        let len = row.len();

        if first_free < len {
            return if self.order.is_left_to_right() {
                row[first_free] == node
            } else {
                row[len - first_free - 1] == node
            };
        }
        false
    }

    /// \returns True if the vertical \p idx is ready for scheduling (if all of
    /// the dependencies are met).
    fn is_vertical_ready(&self, idx: usize) -> bool {
        let vert = &self.vl[idx];
        if vert.is_empty() {
            return false;
        }
        for node in vert {
            let level = self.vg.dag.level(*node);
            if !self.is_next_avail_in_row(*node, level) {
                return false;
            }
        }
        true
    }
}

pub struct BK<'a> {
    vg: &'a mut VisualGraph,
}

// A set of edges between two nodes in the graph.
type EdgeSet = HashSet<(NodeHandle, NodeHandle)>;
// Represents an edge between two rows (index of the element in the row).
type EdgeIdxs = (usize, usize);
// A list of nodes that are vertically aligned.
type Vertical = Vec<NodeHandle>;
// Represents a list of nodes that needs to be scheduled vertically.
type VerticalList = Vec<Vertical>;

impl<'a> BK<'a> {
    pub fn new(vg: &'a mut VisualGraph) -> Self {
        Self { vg }
    }

    /// A conflict happens when the order of the indices in the row does
    /// not match.
    /// R1:  o  o         o  o
    ///       \\/        /    \
    /// CROSS  \\       /      \
    ///       / \\     /   OK   \
    /// R0:  o    o   o          o
    /// \returns True if the edge \p reg crosses the edge \p strong.
    /// Edges are represented as a pair of indices representing the index of the
    /// src and dest node in the rows.
    fn are_edges_crossing(edge_a: EdgeIdxs, edge_b: EdgeIdxs) -> bool {
        // Check if there is no conflict.
        let before = edge_a.0 < edge_b.0 && edge_a.1 < edge_b.1;
        let after = edge_a.0 > edge_b.0 && edge_a.1 > edge_b.1;
        !before && !after
    }

    /// Compute and return a list of successor edges that don't cross the
    /// internal edges (edges between connection nodes).
    fn get_valid_edges(&self) -> EdgeSet {
        let mut valid_edges: EdgeSet = EdgeSet::new();
        for i in 0..self.vg.dag.num_levels() - 1 {
            let r0 = self.vg.dag.row(i);
            let r1 = self.vg.dag.row(i + 1);
            let edges = self.extract_edges_with_no_type2_conflict(r0, r1);
            for e in edges {
                valid_edges.insert(e);
            }
        }
        valid_edges
    }

    /// Iterates over all of the successor edges between two rows and returns
    /// a vector of edges that don't cross the strong edges (from, to).
    fn extract_edges_with_no_type2_conflict(
        &self,
        r0: &[NodeHandle],
        r1: &[NodeHandle],
    ) -> Vec<(NodeHandle, NodeHandle)> {
        let mut regular_edges: Vec<EdgeIdxs> = Vec::new();
        let mut strong_edges: Vec<EdgeIdxs> = Vec::new();
        // For each node in R0:
        for (idx0, elem) in r0.iter().enumerate() {
            // For each successor:
            for succ in self.vg.succ(*elem) {
                // Check if and where it points to in R1. (we could have
                // same-row self-edges).
                if let Option::Some(idx1) = r1.iter().position(|&r| r == *succ)
                {
                    // Figure out if this is a strong edge or a regular edge.
                    let c0 = self.vg.is_connector(*elem);
                    let c1 = self.vg.is_connector(*succ);
                    if c0 && c1 {
                        strong_edges.push((idx0, idx1));
                    } else {
                        regular_edges.push((idx0, idx1));
                    }
                }
            }
        }
        let mut res: Vec<(NodeHandle, NodeHandle)> = Vec::new();

        'outer: for reg in regular_edges.iter() {
            for strong in strong_edges.iter() {
                // Check if there is no conflict.
                if Self::are_edges_crossing(*reg, *strong) {
                    // Continue to the next strong edges.
                    continue;
                }

                // Found a conflict, we must not register this edge.
                continue 'outer;
            }
            // None of the strong edges conflicted with the regular edge.
            res.push((r0[reg.0], r1[reg.1]));
        }

        // Now also add the strong edges.
        for strong in strong_edges {
            // None of the strong edges conflicted with the regular edge.
            res.push((r0[strong.0], r1[strong.1]));
        }
        res
    }

    /// Computes the median of the predecessors, considering only allowed edges.
    /// Returns a list of x coordinates, for each node in the graph. If the node
    /// has no predecessors then the procedure returns the value zero.
    fn get_pred_medians(&self, valid_edges: EdgeSet) -> Vec<f64> {
        // Builds the median of preds for each node.
        let mut res: Vec<f64> = Vec::new();

        // Collect a list of the pred's x coordinates.
        let mut pos_list: Vec<f64> = Vec::new();

        // For each node.
        for node in self.vg.iter_nodes() {
            pos_list.clear();

            // for each predecessor:
            for pred in self.vg.preds(node) {
                // Make sure that this is a valid edge. We swap the direction of the
                // edges because the list is a collection of successor edges.
                if !valid_edges.contains(&(*pred, node)) {
                    continue;
                }
                let pos = self.vg.pos(*pred).center().x;
                pos_list.push(pos)
            }

            // Merge all of the predecessors into one median value.
            if pos_list.is_empty() {
                res.push(0.);
            } else {
                res.push(weighted_median(&pos_list));
            }
        }
        res
    }

    /// \returns the index of \p elem in \p vec.
    fn index_of(elem: NodeHandle, vec: &[NodeHandle]) -> Option<usize> {
        (0..vec.len()).find(|&i| vec[i] == elem)
    }

    fn compute_alignment(&self, order: OrderLR) -> NodeAttachInfo {
        let num = self.vg.num_nodes();
        let mut align_info = NodeAttachInfo::new(num);

        // Computes important edges (with no type2 conflicts).
        let valid_edges = self.get_valid_edges();

        // The desired medians for each node in the graph.
        let medians: Vec<f64> = self.get_pred_medians(valid_edges);

        for i in 0..self.vg.dag.num_levels() - 1 {
            // The row above.
            let mut r0 = self.vg.dag.row(i).clone();
            // The curent row.
            let mut r1 = self.vg.dag.row(i + 1).clone();
            // Marks which nodes of r0 are available.
            let mut used: Vec<bool> = vec![false; r0.len()];

            // Simulate searching from the right by reversing the order of the
            // edges, and the order of the collisions.
            if !order.is_left_to_right() {
                r1.reverse();
                r0.reverse();
            }

            for node in r1 {
                let node_x = medians[node.get_index()];
                let mut best_idx: Option<usize> = None;
                let mut best_delta = f64::INFINITY;

                // Scan the predecessors:
                for pred in self.vg.preds(node) {
                    let idx;
                    // Search for the index of the predecessor in the row.
                    if let Some(idx_in_row) = Self::index_of(*pred, &r0) {
                        idx = idx_in_row;
                    } else {
                        continue;
                    }

                    // Don't mess with nodes that are taken.
                    if used[idx] {
                        continue;
                    }

                    // Of the remaining edges, select the closest one.
                    let delta = (self.vg.pos(*pred).center().x - node_x).abs();
                    if delta < best_delta {
                        best_idx = Some(idx);
                        best_delta = delta;
                    }
                }

                // Mark the current node as aligned to the 'best' node on the
                // previous line.
                if let Some(idx) = best_idx {
                    for i in 0..(idx + 1) {
                        used[i] = true;
                    }
                    align_info.add(node, r0[idx]);
                }
            }
        }

        align_info
    }

    pub fn do_it(&mut self) {
        let vl = self.compute_alignment(OrderLR::RightToLeft).get_verticals();
        let mut sc0 = Scheduler::new(self.vg, vl, OrderLR::RightToLeft);
        sc0.schedule();
        let vl = self.compute_alignment(OrderLR::RightToLeft).get_verticals();
        let mut sc1 = Scheduler::new(self.vg, vl, OrderLR::LeftToRight);
        sc1.schedule();
        let vl = self.compute_alignment(OrderLR::LeftToRight).get_verticals();
        let mut sc2 = Scheduler::new(self.vg, vl, OrderLR::RightToLeft);
        sc2.schedule();
        let vl = self.compute_alignment(OrderLR::LeftToRight).get_verticals();
        let mut sc3 = Scheduler::new(self.vg, vl, OrderLR::LeftToRight);
        sc3.schedule();

        let xs0 = sc0.get_x_placement().clone();
        let xs1 = sc1.get_x_placement().clone();
        let xs2 = sc2.get_x_placement().clone();
        let xs3 = sc3.get_x_placement().clone();

        for i in 0..xs0.len() {
            let node = NodeHandle::from(i);
            let val = (xs0[i] + xs1[i] + xs2[i] + xs3[i]) / 4.0;
            self.vg.pos_mut(node).set_x(val);
        }

        simple::align_to_left(self.vg);
    }
}

#[test]
fn edge_crossing() {
    /*  X  */
    assert!(BK::are_edges_crossing((0, 10), (10, 0)));
    /* | | */
    assert!(!BK::are_edges_crossing((0, 0), (10, 10)));
    /* \ \ */
    assert!(!BK::are_edges_crossing((10, 0), (13, 3)));
    /*  X  */
    assert!(BK::are_edges_crossing((10, 0), (0, 10)));
    /*  /\  */
    assert!(BK::are_edges_crossing((0, 10), (13, 10)));
    /*  /  \  */
    assert!(!BK::are_edges_crossing((0, 10), (13, 11)));
}

#[test]
fn test_extract_verticals() {
    let mut ai = NodeAttachInfo::new(6);
    ai.add(NodeHandle::new(0), NodeHandle::new(1));
    ai.add(NodeHandle::new(1), NodeHandle::new(2));
    ai.add(NodeHandle::new(2), NodeHandle::new(3));
    ai.add(NodeHandle::new(4), NodeHandle::new(5));

    let verticals = ai.get_verticals();

    assert_eq!(verticals.len(), 2);
    assert_eq!(verticals[0][0].get_index(), 0);
    assert_eq!(verticals[0][1].get_index(), 1);
    assert_eq!(verticals[0][2].get_index(), 2);
    assert_eq!(verticals[0][3].get_index(), 3);
    assert_eq!(verticals[1][0].get_index(), 4);
    assert_eq!(verticals[1][1].get_index(), 5);
}
