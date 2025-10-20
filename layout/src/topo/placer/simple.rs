//! This is a simple pass that just places the boxes in a row, one after the
//! other.

use super::EPSILON;
use crate::adt::dag::{NodeHandle, SubgraphHandle};
use crate::core::geometry::Point;
use crate::std_shapes::shapes::BORDER_PADDING;
use crate::topo::layout::VisualGraph;

/// Move the whole graph all the way to the left.
pub(crate) fn align_to_left(vg: &mut VisualGraph) {
    // Find the element with the lowest X value.
    let mut first_x: f64 = 10000.;

    for elem in vg.iter_nodes() {
        let loc = vg.pos(elem).bbox(true).0.x;
        first_x = first_x.min(loc);
    }

    // Subtract the lowest X value from everything.
    for elem in vg.iter_nodes() {
        vg.pos_mut(elem).translate(Point::new(-first_x, 0.));
    }
}

/// Assign the initial Y coordinates.
pub fn assign_y_coordinates(vg: &mut VisualGraph) {
    let mut lowest_point = 0.;
    for i in 0..vg.dag.num_levels() {
        // set the x of subgraphs in start_rank
        for b_row in vg.dag.top_border_ranks()[i].clone().iter() {
            let mut max_height_border: f64 = 0.;
            for sg in b_row.iter() {
                vg.pos_sg_mut(*sg).set_min_y(lowest_point);
                max_height_border =
                    max_height_border.max(vg.size_sg_label(*sg).y);
            }
            lowest_point += max_height_border + BORDER_PADDING;
        }
        let current_row = vg.dag.row(i);
        // Find the tallest box in the row.
        let mut max_height: f64 = 0.;
        for idx in current_row.iter() {
            let height = vg.pos(*idx).size(true).y;
            max_height = max_height.max(height);
        }

        // Align all of the boxes.
        let new_center = lowest_point + max_height / 2.;
        for idx in current_row.clone().iter() {
            let height = vg.pos(*idx).size(true).y;
            vg.pos_mut(*idx).align_to_top(new_center - height / 2.);
        }

        lowest_point += max_height;

        for b_row in vg.dag.bottom_border_ranks()[i].clone().iter().rev() {
            lowest_point += BORDER_PADDING;
            for sg in b_row.iter() {
                vg.pos_sg_mut(*sg).set_max_y(lowest_point);
            }
            lowest_point += BORDER_PADDING;
        }
    }
}

pub fn adjust_subgraph_borders(vg: &mut VisualGraph) {
    // TODO: we need to go through every non border node and adjust the borders
    // since sometimes borders puts larger limit than needed to contain child nodes.
    // Is there a more efficient way to do this?

    let subgraphs_num = vg.dag.num_subgraphs();
    let mut min_subgraph_x: Vec<f64> = vec![f64::MAX; subgraphs_num];
    let mut max_subgraph_x: Vec<f64> = vec![f64::MIN; subgraphs_num];
    for i in 0..vg.dag.len() {
        let node = NodeHandle::new(i);
        if vg.dag.is_vertical_border(node) {
            break;
        }
        let sg = vg.dag.get_parent_subgraph_index_n(node);
        let bbox = vg.pos(node).bbox(false);
        min_subgraph_x[sg.get_index()] =
            min_subgraph_x[sg.get_index()].min(bbox.0.x - BORDER_PADDING);
        max_subgraph_x[sg.get_index()] =
            max_subgraph_x[sg.get_index()].max(bbox.1.x + BORDER_PADDING);
    }

    for i in (0..subgraphs_num).rev() {
        for j in vg.dag.get_children_subgraphs(SubgraphHandle::new(i)).iter() {
            let child_index = j.get_index();
            min_subgraph_x[i] = min_subgraph_x[i]
                .min(min_subgraph_x[child_index] - BORDER_PADDING);
            max_subgraph_x[i] = max_subgraph_x[i]
                .max(max_subgraph_x[child_index] + BORDER_PADDING);
        }
    }

    for i in 0..subgraphs_num {
        vg.pos_sg_mut(SubgraphHandle::new(i))
            .set_min_x(min_subgraph_x[i]);
        vg.pos_sg_mut(SubgraphHandle::new(i))
            .set_max_x(max_subgraph_x[i]);
    }
}

/// Assign the initial x coordinates based on the natural ordering in the
/// rank.
fn assign_x_coordinates(vg: &mut VisualGraph) {
    for i in 0..vg.dag.num_levels() {
        let current_row = vg.dag.row(i);
        let mut rightmost_point = 0.;
        for idx in current_row.clone().iter() {
            let pos = vg.pos_mut(*idx);
            pos.align_to_left(rightmost_point + EPSILON);
            rightmost_point = pos.bbox(true).1.x + EPSILON;
        }
    }
}

pub(crate) fn do_it(vg: &mut VisualGraph) {
    // Adjust the boxes within the line (along y).
    assign_y_coordinates(vg);

    // Assign X coordinates. Using the rank order from the topological sort
    // is a good starting point.
    assign_x_coordinates(vg);
}
