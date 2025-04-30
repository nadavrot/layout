//! This is a pass that moves labels up and down to rows that are less busy.

use crate::adt::dag::NodeHandle;
use crate::core::format::Visible;
use crate::std_shapes::shapes::ShapeKind;
use crate::topo::layout::VisualGraph;

/// Returns the sum of the width of the blocks in a row.
fn get_row_width(vg: &mut VisualGraph, idx: usize) -> f64 {
    let mut sum = 0.;
    let row = vg.dag.row(idx);

    for elem in row {
        sum += vg.pos(*elem).size(true).x;
    }

    sum
}

/// Move the node label from curr to pred. Pred must have an empty label.
/// Return True if it was possible to move the labels.
fn move_label(
    vg: &mut VisualGraph,
    curr: NodeHandle,
    pred: NodeHandle,
) -> bool {
    let curr_shape = vg.element(curr).shape.clone();
    let pred_shape = vg.element(pred).shape.clone();

    // Make sure that pred is empty.
    if let ShapeKind::Connector(txt) = pred_shape {
        if txt.is_some() {
            return false;
        }
    } else {
        return false;
    }

    // Check the curr node and make the move.
    if let ShapeKind::Connector(txt) = curr_shape {
        if txt.is_none() {
            return false;
        }
        vg.element_mut(pred).shape = ShapeKind::Connector(txt);
        vg.element_mut(curr).shape = ShapeKind::Connector(None);
        vg.element_mut(pred).resize();
        vg.element_mut(curr).resize();
        return true;
    }
    false
}

fn move_text_up(vg: &mut VisualGraph) -> usize {
    // Holds the size of the row above.
    let mut prev_row_size = get_row_width(vg, 0);
    let mut cnt = 0;
    // For each row, starting from the second row:
    for i in 1..vg.dag.num_levels() {
        let row = vg.dag.row(i).clone();
        let mut curr_row_size = get_row_width(vg, i);

        for elem in row.iter() {
            // Consider connectors.
            if !vg.is_connector(*elem) {
                continue;
            }

            // Only consider connectors with a single predecessor.
            if vg.dag.predecessors(*elem).len() != 1 {
                continue;
            }

            // The predecessor must also be a label.
            let pred = vg.dag.predecessors(*elem)[0];
            if !vg.is_connector(pred) {
                continue;
            }

            // Check that the previous element is smaller.
            let pred_node_size = vg.pos(pred).size(true).x;
            let curr_node_size = vg.pos(*elem).size(true).x;

            // Compare the previous row size to the current row size and decide
            // where the label would fit better.
            if prev_row_size + curr_node_size < curr_row_size {
                // Try to move up.
                if move_label(vg, *elem, pred) {
                    curr_row_size -= curr_node_size;
                    prev_row_size += curr_node_size;
                    cnt += 1;
                    continue;
                }
            }

            if prev_row_size > curr_row_size + pred_node_size {
                // Try to move down
                if move_label(vg, pred, *elem) {
                    curr_row_size += pred_node_size;
                    prev_row_size -= pred_node_size;
                    cnt += 1;
                    continue;
                }
            }
        }
    }
    cnt
}

#[cfg_attr(not(feature = "log"), allow(unused_assignments, unused_variables))]
pub(crate) fn do_it(vg: &mut VisualGraph) {
    let mut cnt = 0;
    for _ in 0..3 {
        cnt += move_text_up(vg);
    }
    #[cfg(feature = "log")]
    log::info!("Moved {} labels between rows", cnt);
}
