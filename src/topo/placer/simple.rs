//! This is a simple pass that just places the boxes in a row, one after the
//! other.

use super::EPSILON;
use crate::core::geometry::Point;
use crate::topo::layout::VisualGraph;

/// Move the whole graph all the way to the left.
pub fn align_to_left(vg: &mut VisualGraph) {
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
fn assign_y_coordinates(vg: &mut VisualGraph) {
    let mut lowest_point = 0.;
    for i in 0..vg.dag.num_levels() {
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

pub fn do_it(vg: &mut VisualGraph) {
    // Adjust the boxes within the line (along y).
    assign_y_coordinates(vg);

    // Assign X coordinates. Using the rank order from the topological sort
    // is a good starting point.
    assign_x_coordinates(vg);
}
