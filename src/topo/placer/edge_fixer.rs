//! This is pass attempts to straighten crooked edges.

use super::EPSILON;
use crate::adt::dag::NodeHandle;
use crate::core::geometry::{in_range, segment_rect_intersection, Point};
use crate::topo::layout::VisualGraph;
use crate::topo::placer::simple::align_to_left;

/// Return the leftmost and rightmost x coordinate that are taken by another
/// shape.
fn compute_bounds_for_node(vg: &VisualGraph, node: NodeHandle) -> (f64, f64) {
    let level = vg.dag.level(node);
    let row = vg.dag.row(level);
    assert!(!row.is_empty(), "Empty Row!");

    let pos = vg.pos(node);
    let idx = row.iter().position(|x| *x == node).unwrap();

    // Calculate the leftmost point.
    let mut leftmost = f64::NEG_INFINITY;
    if idx > 0 {
        let prev = row[idx - 1];
        leftmost = vg.pos(prev).right(true);
    }

    // Calculate the rightmost point.
    let mut rightmost = f64::INFINITY;
    if idx < row.len() - 1 {
        let next = row[idx + 1];
        rightmost = vg.pos(next).left(true);
    }

    let loc = pos.center();
    assert!(loc.x >= leftmost);
    assert!(loc.x <= rightmost);
    (leftmost, rightmost)
}

pub fn straighten_edge(vg: &mut VisualGraph) -> usize {
    let mut cnt = 0;

    let mut to_straighten: Vec<NodeHandle> = Vec::new();

    for row_idx in 1..vg.dag.num_levels() - 1 {
        let row = vg.dag.row(row_idx);

        'out: for elem in row.iter() {
            if !vg.is_connector(*elem) {
                continue;
            }

            let pred = vg.dag.single_pred(*elem);
            let succ = vg.dag.single_succ(*elem);
            if let Some(pred) = pred {
                if let Some(succ) = succ {
                    // Only adjust labels between two nodes.
                    if vg.is_connector(pred) || vg.is_connector(succ) {
                        continue;
                    }

                    let p1 = vg.pos(pred).center();
                    let p2 = vg.pos(succ).center();
                    let seg = (p1, p2);
                    // Check if the direct edge between the incoming and
                    // outgoing edges that create a straight line intersect with
                    // any of the boxes in the current row.
                    for elem in row.iter() {
                        let rect = vg.pos(*elem).bbox(false);
                        if segment_rect_intersection(seg, rect) {
                            // The line intersects with some box. Move to the
                            // next candidate.
                            continue 'out;
                        }
                    }

                    // Found an element to straighten.
                    to_straighten.push(*elem);
                }
            }
        }
    }

    // Straighten the edges by moving the center block.
    for elem in to_straighten {
        let pred = vg.dag.single_pred(elem).unwrap();
        let succ = vg.dag.single_succ(elem).unwrap();
        let p1 = vg.pos(pred).center();
        let p2 = vg.pos(succ).center();
        let new_pos = p1.add(p2).scale(0.5);

        let bounds = compute_bounds_for_node(vg, elem);
        if in_range(bounds, new_pos.x) {
            vg.pos_mut(elem).set_x(new_pos.x);
            cnt += 1;
        }
    }
    cnt
}

pub fn handle_disconnected_nodes(vg: &mut VisualGraph) -> usize {
    let mut cnt = 0;

    for row_idx in 0..vg.dag.num_levels() {
        let row = vg.dag.row(row_idx).clone();

        for elem in row.iter() {
            // Only consider edges with no successors and predecessors.
            if !vg.dag.successors(*elem).is_empty()
                || !vg.dag.predecessors(*elem).is_empty()
            {
                continue;
            }

            let range = compute_bounds_for_node(vg, *elem);

            // Try to align to the left.
            if range.0.is_finite() {
                vg.pos_mut(*elem).align_to_left(range.0 + EPSILON);
                cnt += 1;
                continue;
            }

            // Try to align to the right.
            if range.1.is_finite() {
                vg.pos_mut(*elem).align_to_right(range.1 - EPSILON);
                cnt += 1;
                continue;
            }
        }
    }
    cnt
}

pub fn align_self_edges(vg: &mut VisualGraph) -> usize {
    let mut cnt = 0;

    for row_idx in 0..vg.dag.num_levels() {
        let row = vg.dag.row(row_idx).clone();

        for (i, curr) in row.iter().enumerate() {
            // Only consider connectors.
            if !vg.is_connector(*curr) {
                continue;
            }

            let mut found_before = false;
            let mut found_after = false;
            for pred in vg.dag.predecessors(*curr) {
                let idx = row.iter().position(|x| *x == *pred);
                if let Option::Some(idx) = idx {
                    if idx < i {
                        found_before = true;
                    }
                    if idx > i {
                        found_after = true;
                    }
                }
            }

            if found_before {
                let prev = row[i - 1];
                let prev_pos = vg.pos(prev);
                vg.pos_mut(*curr).align_to_left(prev_pos.right(true));
                cnt += 1;
                continue;
            }
            if found_after {
                let next = row[i + 1];
                let next_pos = vg.pos(next);
                vg.pos_mut(*curr).align_to_right(next_pos.left(true));
                cnt += 1;
                continue;
            }
        }
    }
    cnt
}

type Segment = (Point, Point);
type Rect = (Point, Point);

fn is_intersecting_any(segs: &[Segment], rects: &[Rect]) -> bool {
    for seg in segs {
        for rec in rects {
            if segment_rect_intersection(*seg, *rec) {
                return true;
            }
        }
    }
    false
}

pub fn adjust_crossing_edges(vg: &mut VisualGraph) -> usize {
    let mut cnt = 0;
    // A list of nodes to adjust, and the dy.
    let mut to_move: Vec<(NodeHandle, Point)> = Vec::new();
    let len = vg.dag.num_levels();

    let offsets = [
        Point::new(0., 15.),
        Point::new(0., 25.),
        Point::new(0., 35.),
        Point::new(0., 45.),
        Point::new(0., 55.),
        Point::new(0., 65.),
        Point::new(0., 75.),
        Point::new(0., 85.),
        Point::new(0., 95.),
        Point::new(0., -10.),
        Point::new(0., 20.),
        Point::new(0., -20.),
        Point::new(0., 30.),
        Point::new(0., -30.),
        Point::new(0., 40.),
        Point::new(0., -40.),
        Point::new(0., 50.),
        Point::new(0., -50.),
        Point::new(0., 90.),
        Point::new(0., -90.),
    ];

    'out: for row_idx in 0..len {
        let row = vg.dag.row(row_idx);

        // Construct a list of all of the boxes in the rows above and below.
        let mut all = Vec::new();
        if row_idx > 1 {
            for elem in vg.dag.row(row_idx - 1) {
                all.push(*elem);
            }
        }
        if row_idx < len - 1 {
            for elem in vg.dag.row(row_idx + 1) {
                all.push(*elem);
            }
        }

        for i in 0..row.len() {
            let curr = row[i];
            if !vg.is_connector(curr) {
                continue;
            }

            let pred = vg.dag.single_pred(curr);
            let succ = vg.dag.single_succ(curr);
            if let Some(pred) = pred {
                if let Some(succ) = succ {
                    let p0 = vg.pos(pred).center();
                    let p1 = vg.pos(curr).center();
                    let p2 = vg.pos(succ).center();
                    let seg0 = (p0, p1);
                    let seg1 = (p1, p2);

                    let mut pos_all = Vec::new();
                    let mut bounds = Vec::new();
                    if i > 0 {
                        bounds.push(vg.pos(row[i - 1]).bbox(false));
                        pos_all.push(vg.pos(row[i - 1]).bbox(false));
                    }
                    if i < row.len() - 1 {
                        bounds.push(vg.pos(row[i + 1]).bbox(false));
                        pos_all.push(vg.pos(row[i + 1]).bbox(false));
                    }

                    for e in all.iter() {
                        if *e != pred && *e != succ {
                            pos_all.push(vg.pos(*e).bbox(false));
                        }
                    }

                    if is_intersecting_any(&[seg0, seg1], &bounds) {
                        for offset in offsets {
                            let seg0 = (seg0.0, seg0.1.add(offset));
                            let seg1 = (seg1.0.add(offset), seg1.1);
                            if !is_intersecting_any(&[seg0, seg1], &pos_all) {
                                to_move.push((curr, offset));
                                continue 'out;
                            }
                        }
                    }
                }
            }
        }
    }

    // Straighten the edges by moving the center block.
    for elem in to_move {
        vg.pos_mut(elem.0).translate(elem.1);
        cnt += 1;
    }
    cnt
}

pub fn do_it(vg: &mut VisualGraph) {
    let mut cnt = 0;
    cnt += handle_disconnected_nodes(vg);
    cnt += align_self_edges(vg);
    align_to_left(vg);
    log::info!("Aligned {} edges.", cnt);

    cnt = straighten_edge(vg);
    log::info!("Straightened {} edges.", cnt);

    cnt = adjust_crossing_edges(vg);
    log::info!("Adjusted crossing {} edges.", cnt);
}
