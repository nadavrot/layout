//! This module contains the implementation of the placer, which assigns the
//! final (x,y) coordinates to all of the elements in the graph.

extern crate log;
use crate::topo::layout::VisualGraph;

use crate::topo::placer::bk::BK;
use crate::topo::placer::edge_fixer;
use crate::topo::placer::move_between_rows;
use crate::topo::placer::simple;
use crate::topo::placer::verifier;

pub struct Placer<'a> {
    vg: &'a mut VisualGraph,
}

impl<'a> Placer<'a> {
    pub fn new(vg: &'a mut VisualGraph) -> Self {
        Self { vg }
    }

    pub fn layout(&mut self, no_layout: bool) {
        log::info!("Starting layout of {} nodes. ", self.vg.num_nodes());

        // We implement left-to-right layout by transposing the graph.
        let need_transpose = !self.vg.orientation().is_top_to_bottom();
        if need_transpose {
            log::info!("Placing nodes in Left-to-right mode.");
            self.vg.transpose();
        } else {
            log::info!("Placing nodes in Top-to-Bottom mode.");
        }

        move_between_rows::do_it(self.vg);

        // Adjust the boxes within the line (along y) and assign consecutive X
        // coordinates.
        simple::do_it(self.vg);

        // Check that the spacial order of the blocks matches the order in the
        // rank.
        verifier::do_it(self.vg);

        if no_layout {
            log::info!("Skipping the layout phase.");
            // Finalize left-to-right graphs.
            if need_transpose {
                self.vg.transpose();
            }
            return;
        }

        BK::new(self.vg).do_it();

        verifier::do_it(self.vg);

        edge_fixer::do_it(self.vg);

        // Finalize left-to-right graphs.
        if need_transpose {
            self.vg.transpose();
        }
    }
}
