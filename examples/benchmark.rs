//! This is a sample program that prints a bunch of elements to an SVG file so
//! we can visually see if the things that we render look right.

use layout::backends::svg::SVGWriter;
use layout::core::base::Orientation;
use layout::core::color::Color;
use layout::core::geometry::Point;
use layout::core::style::StyleAttr;
pub const LAYOUT_HELPER: bool = true;

fn test_main(n_node: usize, _n_edge: usize) {
    let mut svg = SVGWriter::new();
    let mut gb = VisualGraph::new(Orientation::LeftToRight);

    for i in 0..n_node {
        let elem = Element::create(
            ShapeKind::Circle(format!("hi_{}", i)),
            StyleAttr::new(Color::transparent(), 0, None, 0, 0),
            Orientation::LeftToRight,
            Point::zero(),
        );
        gb.add_node(elem);
    }
    let t0 = std::time::Instant::now();

    gb.do_it(false, true, false, &mut svg);

    let duration = t0.elapsed();
    println!("Time elapsed in expensive_function() is: {:?}", duration);
    println!("--------------------------------------");
}
use layout::std_shapes::shapes::{Element, ShapeKind};
use layout::topo::layout::VisualGraph;

fn main() {
    let n_edge = 16;
    for n_node in [16, 32, 64, 128, 256, 512, 1024, 2048, 4096] {
        test_main(n_node, n_edge);
    }
}
