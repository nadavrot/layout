//! This module contains optimization passes that transform the graphs in different
//! phases of the program. Here you can find things like optimizations for
//! sinking or hoisting nodes to reduce the number of live edges, and
//! optimizations that move nodes within a row to reduce edge crossing.

use crate::adt::dag::NodeHandle;
use crate::adt::dag::DAG;
use crate::core::base::Direction;

/// This optimizations changes the order of nodes within a rank (ordering along
/// the x-axis). The transformation tries to reduce the number of edges that
/// cross each other.
pub struct EdgeCrossOptimizer<'a> {
    dag: &'a mut DAG,
}
impl<'a> EdgeCrossOptimizer<'a> {
    pub fn new(dag: &'a mut DAG) -> Self {
        Self { dag }
    }

    /// Given two nodes that may have connections in \p row, check how many of
    /// these edges intersect. Check both successors and predecessors.
    ///               A   B
    ///             /   \/ \
    ///            /    /\  \
    ///  Row: [][][][][][][][][][]
    fn num_crossing(
        &self,
        a: NodeHandle,
        b: NodeHandle,
        row: &[NodeHandle],
    ) -> usize {
        let mut sum = 0;
        // Record the number of edges that previously connected with node B.
        let mut num_b = 0;

        let a_edges1 = self.dag.successors(a);
        let a_edges2 = self.dag.predecessors(a);
        let b_edges1 = self.dag.successors(b);
        let b_edges2 = self.dag.predecessors(b);

        for node in row {
            let is_a1 = a_edges1.iter().any(|x| x == node);
            let is_a2 = a_edges2.iter().any(|x| x == node);
            let is_b1 = b_edges1.iter().any(|x| x == node);
            let is_b2 = b_edges2.iter().any(|x| x == node);
            if is_a1 || is_a2 {
                sum += num_b;
            }
            if is_b1 || is_b2 {
                num_b += 1;
            }
        }
        sum
    }

    // Shuffle the nodes in all of the ranks.
    pub fn perturb_rank(&mut self) {
        for i in 0..self.dag.num_levels() {
            let row = self.dag.row_mut(i);
            let len = row.len();
            for j in 0..len {
                row.swap((j * 17) % len, j);
            }
        }
    }

    // Move the elements in the rank to the left, to perturb the graph.
    pub fn rotate_rank(&mut self) {
        for i in 0..self.dag.num_levels() {
            let row = self.dag.row_mut(i);
            row.rotate_left(1);
        }
    }

    pub fn optimize(&mut self) {
        self.dag.verify();
        #[cfg(feature = "log")]
        log::info!("Optimizing edge crossing.");
        let mut best_rank = self.dag.ranks().clone();
        let mut best_cnt = self.count_crossed_edges();
        #[cfg(feature = "log")]
        log::info!("Starting with {} crossings.", best_cnt);
        for i in 0..50 {
            let dir = match i % 4 {
                0 => Direction::Both,
                1 => Direction::Up,
                _ => Direction::Down,
            };
            self.swap_crossed_edges(dir);
            let new_cnt = self.count_crossed_edges();
            if new_cnt < best_cnt {
                #[cfg(feature = "log")]
                log::info!("Found a rank with {} crossings.", new_cnt);
                best_rank = self.dag.ranks().clone();
                best_cnt = new_cnt;
            }
            self.rotate_rank();
            if i % 10 == 0 {
                self.perturb_rank();
            }
        }
        *self.dag.ranks_mut() = best_rank;
    }

    fn count_crossed_edges(&self) -> usize {
        let mut sum = 0;
        // Compare each row to the row afterwards.
        for row_idx in 0..self.dag.num_levels() - 1 {
            let first_row = self.dag.row(row_idx);
            let second_row = self.dag.row(row_idx + 1);
            sum += self.count_crossing_in_rows(first_row, second_row);
        }
        sum
    }

    fn count_crossing_in_rows(
        &self,
        first: &[NodeHandle],
        second: &[NodeHandle],
    ) -> usize {
        if first.len() < 2 {
            return 0;
        }
        let mut sum = 0;
        // Check for each pair of nodes a,b where b comes after a.
        for i in 0..first.len() {
            for j in i + 1..first.len() {
                let a = first[i];
                let b = first[j];
                sum += self.num_crossing(a, b, second);
            }
        }
        sum
    }

    /// Scan all of the node pairs in the module and count the number of crossed
    /// edges. If \p allow_swap is set then swap the edges if it reduces the
    /// number of crossing.
    fn swap_crossed_edges(&mut self, dir: Direction) {
        let mut changed = true;
        while changed {
            changed = false;
            if dir.is_down() {
                for i in 0..self.dag.num_levels() {
                    changed |= self.swap_crossed_edges_on_row(i, dir);
                }
            }
            if dir.is_up() {
                for i in (0..self.dag.num_levels()).rev() {
                    changed |= self.swap_crossed_edges_on_row(i, dir);
                }
            }
        }
    }

    /// See swap_crossed_edges.
    fn swap_crossed_edges_on_row(
        &mut self,
        row_idx: usize,
        dir: Direction,
    ) -> bool {
        let mut changed = false;

        let num_rows = self.dag.num_levels();

        let prev_row = if row_idx > 0 && dir.is_up() {
            self.dag.row(row_idx - 1).clone()
        } else {
            Vec::new()
        };
        let next_row = if row_idx + 1 < num_rows && dir.is_down() {
            self.dag.row(row_idx + 1).clone()
        } else {
            Vec::new()
        };

        let mut row = self.dag.row(row_idx).clone();

        if row.len() < 2 {
            return false;
        }

        // For each two consecutive elements in the row:
        for i in 0..row.len() - 1 {
            let a = row[i];
            let b = row[i + 1];

            let mut ab = 0;
            let mut ba = 0;
            // Figure out if A crosses the edges of B, and vice versa, on both
            // the edges pointing up and down.
            ab += self.num_crossing(a, b, &prev_row);
            ba += self.num_crossing(b, a, &prev_row);
            ab += self.num_crossing(a, b, &next_row);
            ba += self.num_crossing(b, a, &next_row);

            // Swap the edges.
            if ab > ba {
                row[i] = b;
                row[i + 1] = a;
                changed = true;
            }
        }

        if changed {
            *self.dag.row_mut(row_idx) = row;
        }
        changed
    }
}

/// This optimization sinks nodes in an attempt to shorten the length of edges
/// that run through the graph.
pub struct RankOptimizer<'a> {
    dag: &'a mut DAG,
}

impl<'a> RankOptimizer<'a> {
    pub fn new(dag: &'a mut DAG) -> Self {
        Self { dag }
    }

    pub fn try_to_sink_node(&mut self, node: NodeHandle) -> bool {
        let backs = self.dag.predecessors(node);
        let fwds = self.dag.successors(node);

        // Don't try to sink if we increase the number of live edges,
        // or if there are no forward edges.
        if backs.len() > fwds.len() || backs.len() + fwds.len() == 0 {
            return false;
        }

        let curr_rank = self.dag.level(node);
        let mut highest_next = self.dag.len();
        for elem in fwds {
            let next_rank = self.dag.level(*elem);
            highest_next = highest_next.min(next_rank);
        }

        // We found an opportunity to sink a node.
        if highest_next > curr_rank + 1 {
            self.dag
                .update_node_rank_level(node, highest_next - 1, None);
            return true;
        }
        false
    }

    // Try to sink nodes to shorten the length of edges.
    pub fn optimize(&mut self) {
        self.dag.verify();

        #[cfg(feature = "log")]
        log::info!("Optimizing the ranks.");
        #[cfg(feature = "log")]
        let mut cnt = 0;
        #[cfg(feature = "log")]
        let mut iter = 0;

        loop {
            let mut c = 0;
            for node in self.dag.iter() {
                if self.try_to_sink_node(node) {
                    c += 1;
                }
            }
            #[cfg(feature = "log")]
            {
                cnt += c;
                iter += 1;
            }
            if c == 0 {
                break;
            }
        }

        #[cfg(feature = "log")]
        log::info!("Sank {} nodes in {} iteration.", cnt, iter);
    }
}
