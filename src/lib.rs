/*!
This crate provides a library for parsing and rendering GraphViz files.  It
supports many of the GraphViz features, like records, edge styles and
left-to-right graphs, but lacks other features, such as nested graphs or
embedded html. This crate also provides an API for constructing and rendering
graphs.

For more specific details on the API for regular expressions, see the
documentation for the specific sub modules.

The project also comes with a command line utility for rendering .DOT files to
.svg.

# Parser example: parse a dot file

This crate provides an API for parsing DOT files. For example,
to load, parse and print the AST:

```rust
    use layout::gv;
    use std::fs;

    let contents = "digraph { a -> b [label=\"foo\"]; }";
    let mut parser = gv::DotParser::new(&contents);
    let tree = parser.process();

    match tree {
        Result::Err(err) => {
            parser.print_error();
            log::error!("Error: {}", err);
        }

        Result::Ok(g) => {
                gv::dump_ast(&g);
        }
    }
```

The example above would print the program AST, or a readable error message,
such as:

```txt
digraph {

node [fillcolor="purple"] A B;
node [fillcolor="orange"] Z;
node [fillcolor="green"] G; a = ;
                                ^
Error: Expected an identifier.
```


# Graph Builder example: create a new graph

This crate provides an API creating and rendering graphs. For example, this
code builds a graph with two nodes that are connected with an edge.

```rust
fn simple_graph() {
    use layout::backends::svg::SVGWriter;
    use layout::core::base::Orientation;
    use layout::core::geometry::Point;
    use layout::core::style::*;
    use layout::core::utils::save_to_file;
    use layout::std_shapes::shapes::*;
    use layout::topo::layout::VisualGraph;
    use layout::topo::placer::Placer;
    use std::fs;

    // Create a new graph:
    let mut vg = VisualGraph::new(Orientation::LeftToRight);

    // Define the node styles:
    let sp0 = ShapeKind::new_box("one");
    let sp1 = ShapeKind::new_box("two");
    let look0 = StyleAttr::simple();
    let look1 = StyleAttr::simple();
    let sz = Point::new(100., 100.);
    // Create the nodes:
    let node0 = Element::create(sp0, look0, Orientation::LeftToRight, sz);
    let node1 = Element::create(sp1, look1, Orientation::LeftToRight, sz);

    // Add the nodes to the graph, and save a handle to each node.
    let handle0 = vg.add_node(node0);
    let handle1 = vg.add_node(node1);

    // Add an edge between the nodes.
    let arrow = Arrow::simple("123");
    vg.add_edge(arrow, handle0, handle1);

    // Render the nodes to some rendering backend.
    let mut svg = SVGWriter::new();
    vg.do_it(false, false, false, &mut svg);

    // Save the output.
    let _ = save_to_file("/tmp/graph.svg", &svg.finalize());
}
```

*/

pub mod adt;
pub mod backends;
pub mod core;
pub mod gv;
pub mod std_shapes;
pub mod topo;
