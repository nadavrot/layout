//! This module contains the placer, the code that assigns X,Y coordinates to
//! all of the elements in the graph.

pub const EPSILON: f64 = 0.001;

/// Categorizes blocks to visible and invisible. We use this enum to tell the
/// passes which blocks they are allowed to touch.
#[derive(Clone, Copy)]
pub enum BlockKind {
    Box,
    Connector,
    Both,
    None,
}

impl BlockKind {
    pub fn is_box(&self) -> bool {
        match self {
            BlockKind::None | BlockKind::Connector => false,
            BlockKind::Both | BlockKind::Box => true,
        }
    }
    pub fn is_connector(&self) -> bool {
        match self {
            BlockKind::None | BlockKind::Box => false,
            BlockKind::Both | BlockKind::Connector => true,
        }
    }
}

mod bk;
mod edge_fixer;
mod move_between_rows;
mod simple;
mod verifier;

pub mod place;
pub use place::Placer;
