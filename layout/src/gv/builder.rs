//! A graph builder that converts parsed AST trees to graphs.

use super::html::{parse_html_string, HtmlGrid};
use super::parser::ast::DotString;
use super::record::record_builder;
use crate::adt::dag::NodeHandle;
use crate::adt::map::ScopedMap;
use crate::core::base::Orientation;
use crate::core::color::Color;
use crate::core::style::*;
use crate::gv::parser::ast;
use crate::std_shapes::render::get_shape_size;
use crate::std_shapes::shapes::ShapeKind;
use crate::std_shapes::shapes::*;
use crate::topo::layout::VisualGraph;
use std::collections::HashMap;

type PropertyList = HashMap<String, DotString>;

// The methods in this file are responsible for converting the parsed Graphviz
// AST into the VisualGraph data-structure that we use for layout and rendering
// of the graph.

#[derive(Debug)]
struct EdgeDesc {
    from: String,
    to: String,
    props: PropertyList,
    is_directed: bool,
    from_port: Option<String>,
    to_port: Option<String>,
}

/// This class constructs a visual graph from the parsed AST.
#[derive(Debug)]
pub struct GraphBuilder {
    // This records the state of the top-level graph.
    global_state: PropertyList,
    // This keeps track of the construction order of the nodes, because
    // hashmap does not maintain a persistent iteration order.
    node_order: Vec<String>,
    // Maps node names to their property list.
    nodes: HashMap<String, PropertyList>,
    // A list of edge properties.
    edges: Vec<EdgeDesc>,
    /// Scopes that maintain the property list that changes as we enter and
    /// leave different regions of the graph.
    global_attr: ScopedMap<String, DotString>,
    node_attr: ScopedMap<String, DotString>,
    edge_attr: ScopedMap<String, DotString>,
}
impl Default for GraphBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl GraphBuilder {
    pub fn new() -> Self {
        Self {
            global_state: PropertyList::new(),
            node_order: Vec::new(),
            nodes: HashMap::new(),
            edges: Vec::new(),
            global_attr: ScopedMap::new(),
            node_attr: ScopedMap::new(),
            edge_attr: ScopedMap::new(),
        }
    }
    pub fn visit_graph(&mut self, graph: &ast::Graph) {
        self.global_attr.push();
        self.node_attr.push();
        self.edge_attr.push();
        for stmt in &graph.list.list {
            self.visit_stmt(stmt);
        }

        // TODO: we dump the property list when we close the scope. This is not
        // correct for sub graphs.
        self.global_state = self.global_attr.flatten();

        self.global_attr.pop();
        self.node_attr.pop();
        self.edge_attr.pop();
    }
    fn visit_stmt(&mut self, stmt: &ast::Stmt) {
        match stmt {
            ast::Stmt::Edge(e) => {
                self.visit_edge(e);
            }
            ast::Stmt::Node(n) => {
                self.visit_node(n);
            }
            ast::Stmt::Attribute(a) => {
                self.visit_att(a);
            }
            ast::Stmt::SubGraph(g) => {
                self.visit_graph(g);
            }
        }
    }

    fn visit_edge(&mut self, e: &ast::EdgeStmt) {
        self.edge_attr.push();

        for att in e.list.iter() {
            self.edge_attr.insert(&att.0, &att.1);
        }

        self.init_node_with_name(&e.from.name, false);

        let mut prev = &e.from.name;
        for dest in &e.to {
            let curr = &dest.0.name;
            self.init_node_with_name(curr, false);

            let has_arrow = matches!(dest.1, ast::ArrowKind::Arrow);
            let prop_list = self.edge_attr.flatten();

            let edge = EdgeDesc {
                from: prev.clone(),
                to: curr.clone(),
                props: prop_list,
                is_directed: has_arrow,
                from_port: e.from.port.clone(),
                to_port: dest.0.port.clone(),
            };
            self.edges.push(edge);
            prev = curr;
        }
        self.edge_attr.pop();
    }

    // If \p overwrite is set then we are declaring a node. This means that
    // we need to update the properties that already exist.
    fn init_node_with_name(&mut self, name: &str, overwrite: bool) {
        let node_attr = self.node_attr.flatten();

        if let Option::Some(prop_list) = self.nodes.get_mut(name) {
            if !overwrite {
                return;
            }
            for p in node_attr {
                prop_list.insert(p.0, p.1);
            }
        } else {
            self.node_order.push(name.to_string());
            self.nodes.insert(name.to_string(), node_attr);
        }
    }

    fn visit_node(&mut self, n: &ast::NodeStmt) {
        self.node_attr.push();

        for att in n.list.iter() {
            self.node_attr.insert(&att.0, &att.1);
        }

        self.init_node_with_name(&n.id.name, true);
        self.node_attr.pop();
    }

    fn visit_att(&mut self, att: &ast::AttrStmt) {
        match att.target {
            ast::AttrStmtTarget::Graph => {
                for att in att.list.iter() {
                    self.global_attr.insert(&att.0, &att.1);
                }
            }
            ast::AttrStmtTarget::Node => {
                for att in att.list.iter() {
                    self.node_attr.insert(&att.0, &att.1);
                }
            }
            ast::AttrStmtTarget::Edge => {
                for att in att.list.iter() {
                    self.edge_attr.insert(&att.0, &att.1);
                }
            }
        }
    }

    pub fn get(&self) -> VisualGraph {
        let mut dir = Orientation::TopToBottom;

        // Set the graph orientation based on the 'rankdir' property.
        if let Option::Some(DotString::String(rd)) =
            self.global_state.get("rankdir")
        {
            if rd == "LR" {
                dir = Orientation::LeftToRight;
            }
        }

        let mut vg = VisualGraph::new(dir);

        // Keeps track of the newly created nodes and indexes them by name.
        let mut node_map: HashMap<String, NodeHandle> = HashMap::new();

        assert_eq!(self.nodes.len(), self.node_order.len());

        // Create and register all of the nodes.
        for node_name in self.node_order.iter() {
            let node_prop = self.nodes.get(node_name).unwrap();

            let shape =
                Self::get_shape_from_attributes(dir, node_prop, node_name);
            let handle = vg.add_node(shape);
            node_map.insert(node_name.to_string(), handle);
        }

        // Create and register all of the edges.
        for edge_prop in &self.edges {
            let shape = Self::get_arrow_from_attributes(
                &edge_prop.props,
                edge_prop.is_directed,
                edge_prop.from_port.clone(),
                edge_prop.to_port.clone(),
            );
            let from = node_map.get(&edge_prop.from).unwrap();
            let to = node_map.get(&edge_prop.to).unwrap();
            vg.add_edge(shape, *from, *to);
        }

        vg
    }

    fn get_arrow_from_attributes(
        lst: &PropertyList,
        has_arrow: bool,
        from_port: Option<String>,
        to_port: Option<String>,
    ) -> Arrow {
        let mut line_width = 1;
        let mut font_size: usize = 14;
        let start = LineEndKind::None;
        let end = if has_arrow {
            LineEndKind::Arrow
        } else {
            LineEndKind::None
        };
        let mut label = String::from("");
        let mut color = String::from("black");
        let mut line_style = LineStyleKind::Normal;

        if let Option::Some(DotString::String(val)) =
            lst.get(&"label".to_string())
        {
            label = val.clone();
        }

        if let Option::Some(DotString::String(stl)) =
            lst.get(&"style".to_string())
        {
            if stl == "dashed" {
                line_style = LineStyleKind::Dashed;
            }
        }

        if let Option::Some(DotString::String(x)) =
            lst.get(&"color".to_string())
        {
            color = x.clone();
            color = Self::normalize_color(color);
        }

        if let Option::Some(DotString::String(pw)) =
            lst.get(&"penwidth".to_string())
        {
            if let Result::Ok(x) = pw.parse::<usize>() {
                line_width = x;
            } else {
                #[cfg(feature = "log")]
                log::info!("Can't parse integer \"{}\"", pw);
            }
        }

        if let Option::Some(DotString::String(fx)) =
            lst.get(&"fontsize".to_string())
        {
            if let Result::Ok(x) = fx.parse::<usize>() {
                font_size = x;
            } else {
                #[cfg(feature = "log")]
                log::info!("Can't parse integer \"{}\"", fx);
            }
        }

        let color = Color::fast(&color);
        let look = StyleAttr::new(color, line_width, None, 0, font_size);
        Arrow::new(start, end, line_style, &label, &look, &from_port, &to_port)
    }

    /// Convert the color to some color that we can handle.
    fn normalize_color(color: String) -> String {
        let mut color = color;
        if let Option::Some(idx) = color.find(':') {
            color = color[0..idx].to_string();
        }
        if color == "transparent" {
            color = "white".to_string();
        }
        color
    }

    fn get_shape_from_attributes(
        dir: Orientation,
        lst: &PropertyList,
        default_name: &str,
    ) -> Element {
        let mut label = ShapeContent::String(default_name.to_string());
        let mut edge_color = String::from("black");
        let mut fill_color = String::from("white");
        let mut font_size: usize = 14;
        let mut line_width: usize = 1;
        let mut make_xy_same = false;
        let mut rounded_corder_value = 0;
        // let mut shape = ShapeKind::Circle(label.clone());
        let mut shape = ShapeKind::Circle(label.clone());

        if let Option::Some(x) = lst.get(&"label".to_string()) {
            // label = val.clone();
            match x {
                DotString::String(val) => {
                    label = ShapeContent::String(val.clone());
                    shape =
                        ShapeKind::Circle(ShapeContent::String(val.clone()));
                }
                DotString::HtmlString(val) => {
                    label = ShapeContent::Html(parse_html_string(val).unwrap());
                    shape = ShapeKind::Circle(label.clone());
                }
            }
        }

        // Set the shape.
        if let Option::Some(DotString::String(val)) =
            lst.get(&"shape".to_string())
        {
            match &val[..] {
                "box" => {
                    shape = ShapeKind::Box(label);
                    make_xy_same = false;
                }
                "doublecircle" => {
                    shape = ShapeKind::DoubleCircle(label);
                    make_xy_same = true;
                }
                "record" => {
                    // shape = record_builder(&label);
                    match label {
                        ShapeContent::String(s) => {
                            shape = record_builder(&s);
                        }
                        ShapeContent::Html(_) => {}
                    }
                }
                "Mrecord" => {
                    rounded_corder_value = 15;
                    match label {
                        ShapeContent::String(s) => {
                            shape = record_builder(&s);
                        }
                        ShapeContent::Html(_) => {}
                    }
                }
                _ => {}
            }
        }

        if let Option::Some(DotString::String(x)) =
            lst.get(&"color".to_string())
        {
            edge_color = x.clone();
            edge_color = Self::normalize_color(edge_color);
        }

        if let Option::Some(DotString::String(style)) =
            lst.get(&"style".to_string())
        {
            if style == "filled" && !lst.contains_key("fillcolor") {
                fill_color = "lightgray".to_string();
            }
        }

        if let Option::Some(DotString::String(x)) =
            lst.get(&"fillcolor".to_string())
        {
            fill_color = x.clone();
            fill_color = Self::normalize_color(fill_color);
        }

        if let Option::Some(DotString::String(fx)) =
            lst.get(&"fontsize".to_string())
        {
            if let Result::Ok(x) = fx.parse::<usize>() {
                font_size = x;
            } else {
                #[cfg(feature = "log")]
                log::info!("Can't parse integer \"{}\"", fx);
            }
        }

        if let Option::Some(DotString::String(pw)) =
            lst.get(&"width".to_string())
        {
            if let Result::Ok(x) = pw.parse::<usize>() {
                line_width = x;
            } else {
                #[cfg(feature = "log")]
                log::info!("Can't parse integer \"{}\"", pw);
            }
        }

        // We flip the orientation before we create the shape. In graphs that
        // grow top down the records grow to the left.
        let dir = dir.flip();

        // match &mut shape {
        //     ShapeKind::Html(HtmlGrid::FontTable(x)) => {
        //         x.resize(font_size);
        //     }

        //     _ => {}
        // }
        match &mut shape {
            ShapeKind::Circle(ShapeContent::Html(HtmlGrid::FontTable(x))) => {
                x.resize(font_size);
            }
            ShapeKind::Box(ShapeContent::Html(HtmlGrid::FontTable(x))) => {
                x.resize(font_size);
            }
            ShapeKind::DoubleCircle(ShapeContent::Html(
                HtmlGrid::FontTable(x),
            )) => {
                x.resize(font_size);
            }
            _ => {}
        }

        let sz = get_shape_size(dir, &shape, font_size, make_xy_same);
        let look = StyleAttr::new(
            Color::fast(&edge_color),
            line_width,
            Option::Some(Color::fast(&fill_color)),
            rounded_corder_value,
            font_size,
        );
        Element::create(shape, look, dir, sz)
    }
}
