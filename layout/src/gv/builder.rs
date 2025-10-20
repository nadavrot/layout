//! A graph builder that converts parsed AST trees to graphs.

use super::record::record_builder;
use crate::adt::dag::NodeHandle;
use crate::adt::dag::SubgraphHandle;
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

type PropertyList = HashMap<String, String>;

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

#[derive(Debug, Clone)]
pub struct SubgraphBuilder {
    global_state: PropertyList,
    container_order: Vec<(Vec<String>, ContainerType)>,
    name: String,
    orientation: Orientation,
}
impl SubgraphBuilder {
    pub fn from_ast(name: String) -> Self {
        let subgraph_builder = SubgraphBuilder {
            global_state: PropertyList::new(),
            name,
            container_order: Vec::new(),
            orientation: Orientation::TopToBottom,
        };
        subgraph_builder
    }

    pub fn get(
        &self,
        graph_builder: &GraphBuilder,
        vg: &mut VisualGraph,
        node_map: &mut HashMap<String, (Vec<SubgraphHandle>, NodeHandle)>,
        subgraph_map: &mut HashMap<
            String,
            (Vec<SubgraphHandle>, SubgraphHandle),
        >,
    ) {
        for (subgraph_chain, name) in &self.container_order {
            match name {
                ContainerType::Node(name) => {
                    let node_prop = graph_builder.nodes.get(name).unwrap();
                    let element = GraphBuilder::get_shape_from_attributes(
                        self.orientation,
                        node_prop,
                        name,
                    );
                    let subgraph_chain = subgraph_chain
                        .iter()
                        .map(|x| subgraph_map.get(x).unwrap().1)
                        .collect::<Vec<SubgraphHandle>>();

                    let parent_subgraph_idx =
                        subgraph_chain.last().unwrap().clone();

                    let handle = vg.add_node(element, parent_subgraph_idx);

                    node_map.insert(name.clone(), (subgraph_chain, handle));
                }
                ContainerType::Subgraph(name) => {
                    let subgraph_builder =
                        graph_builder.subgraphs.get(name).unwrap();
                    let subgraph_chain_handle = subgraph_chain
                        .iter()
                        .map(|x| subgraph_map.get(x).unwrap().1)
                        .collect::<Vec<SubgraphHandle>>();
                    let parent_subgraph_idx =
                        subgraph_chain_handle.last().unwrap().clone();

                    // if subgraph is not cluster we only propagate attributes
                    // but do not create a new subgraph in the layout.
                    // this is because non cluster subgraphs do not affect positions of nodes.
                    if name.starts_with("cluster_") {
                        let element =
                            GraphBuilder::get_subgraph_shape_from_attributes(
                                self.orientation,
                                &subgraph_builder.global_state,
                            );
                        let handle =
                            vg.add_subgraph(element, parent_subgraph_idx);
                        subgraph_map.insert(
                            name.clone(),
                            (subgraph_chain_handle, handle),
                        );
                    } else {
                        subgraph_map.insert(
                            name.clone(),
                            (
                                subgraph_chain_handle
                                    [..subgraph_chain_handle.len() - 1]
                                    .to_vec(),
                                parent_subgraph_idx,
                            ),
                        );
                    }

                    subgraph_builder.get(
                        graph_builder,
                        vg,
                        node_map,
                        subgraph_map,
                    );
                }
            }
        }
    }

    pub fn visit_graph(
        &mut self,
        graph: &ast::Graph,
        graph_builder: &mut GraphBuilder,
    ) {
        graph_builder.global_attr.push();
        graph_builder.node_attr.push();
        graph_builder.edge_attr.push();
        graph_builder.subgraph_stack.push(graph.name.clone());
        for stmt in &graph.list.list {
            self.visit_stmt(stmt, graph_builder);
        }
        graph_builder.subgraph_stack.pop();

        self.global_state = graph_builder.global_attr.flatten();
        graph_builder.global_attr.pop();
        graph_builder.node_attr.pop();
        graph_builder.edge_attr.pop();
    }

    pub fn visit_stmt(
        &mut self,
        stmt: &ast::Stmt,
        graph_builder: &mut GraphBuilder,
    ) {
        match stmt {
            ast::Stmt::Edge(e) => {
                self.visit_edge(e, graph_builder);
            }
            ast::Stmt::Node(n) => {
                self.visit_node(n, graph_builder);
            }
            ast::Stmt::Attribute(a) => {
                self.visit_att(a, graph_builder);
            }
            ast::Stmt::Subgraph(g) => {
                let mut subgraph_builder =
                    SubgraphBuilder::from_ast(g.name.clone());
                self.container_order.push((
                    graph_builder.subgraph_stack.clone(),
                    ContainerType::Subgraph(g.name.clone()),
                ));
                subgraph_builder.visit_graph(g, graph_builder);
                graph_builder
                    .subgraphs
                    .insert(g.name.clone(), subgraph_builder);
            }
        }
    }

    pub fn visit_edge(
        &mut self,
        e: &ast::EdgeStmt,
        graph_builder: &mut GraphBuilder,
    ) {
        graph_builder.edge_attr.push();

        for att in e.list.iter() {
            graph_builder.edge_attr.insert(&att.0, &att.1);
        }

        graph_builder.init_node_with_name(&e.from.name, false, self);

        let mut prev = &e.from.name;
        for dest in &e.to {
            let curr = &dest.0.name;
            graph_builder.init_node_with_name(curr, false, self);

            let has_arrow = matches!(dest.1, ast::ArrowKind::Arrow);
            let prop_list = graph_builder.edge_attr.flatten();
            let edge = EdgeDesc {
                from: prev.clone(),
                to: curr.clone(),
                props: prop_list,
                is_directed: has_arrow,
                from_port: e.from.port.clone(),
                to_port: dest.0.port.clone(),
            };
            graph_builder.edges.push(edge);
            prev = curr;
        }
        graph_builder.edge_attr.pop();
    }

    pub fn visit_node(
        &mut self,
        n: &ast::NodeStmt,
        graph_builder: &mut GraphBuilder,
    ) {
        graph_builder.node_attr.push();

        for att in n.list.iter() {
            graph_builder.node_attr.insert(&att.0, &att.1);
        }

        graph_builder.init_node_with_name(&n.id.name, true, self);
        graph_builder.node_attr.pop();
    }

    pub fn visit_att(
        &mut self,
        att: &ast::AttrStmt,
        graph_builder: &mut GraphBuilder,
    ) {
        match att.target {
            ast::AttrStmtTarget::Graph => {
                for att in att.list.iter() {
                    graph_builder.global_attr.insert(&att.0, &att.1);
                }
            }
            ast::AttrStmtTarget::Node => {
                for att in att.list.iter() {
                    graph_builder.node_attr.insert(&att.0, &att.1);
                }
            }
            ast::AttrStmtTarget::Edge => {
                for att in att.list.iter() {
                    graph_builder.edge_attr.insert(&att.0, &att.1);
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
enum ContainerType {
    Node(String),
    Subgraph(String),
}

/// This class constructs a visual graph from the parsed AST.
#[derive(Debug)]
pub struct GraphBuilder {
    // This records the state of the top-level graph.
    main_graph: SubgraphBuilder,
    subgraph_stack: Vec<String>,
    // This keeps track of the construction order of the nodes, because
    subgraphs: HashMap<String, SubgraphBuilder>,
    // hashmap does not maintain a persistent iteration order.
    // Maps node names to their property list.
    nodes: HashMap<String, PropertyList>,
    // A list of edge properties.
    edges: Vec<EdgeDesc>,
    /// Scopes that maintain the property list that changes as we enter and
    /// leave different regions of the graph.
    global_attr: ScopedMap<String, String>,
    node_attr: ScopedMap<String, String>,
    edge_attr: ScopedMap<String, String>,
}
impl Default for GraphBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl GraphBuilder {
    pub fn new() -> Self {
        let main_graph = SubgraphBuilder {
            global_state: PropertyList::new(),
            container_order: Vec::new(),
            name: String::from("main"),
            orientation: Orientation::TopToBottom,
        };
        Self {
            main_graph,
            subgraphs: HashMap::new(),
            subgraph_stack: Vec::new(),
            nodes: HashMap::new(),
            edges: Vec::new(),
            global_attr: ScopedMap::new(),
            node_attr: ScopedMap::new(),
            edge_attr: ScopedMap::new(),
        }
    }
    pub fn visit_graph(&mut self, graph: &ast::Graph) {
        let mut main_graph = SubgraphBuilder::from_ast(graph.name.clone());
        main_graph.visit_graph(graph, self);
        self.main_graph = main_graph;
    }

    // If \p overwrite is set then we are declaring a node. This means that
    // we need to update the properties that already exist.
    fn init_node_with_name(
        &mut self,
        name: &str,
        overwrite: bool,
        subgraph_builder: &mut SubgraphBuilder,
    ) {
        let node_attr = self.node_attr.flatten();

        if let Option::Some(prop_list) = self.nodes.get_mut(name) {
            if !overwrite {
                return;
            }
            for p in node_attr {
                prop_list.insert(p.0, p.1);
            }
        } else {
            subgraph_builder.container_order.push((
                self.subgraph_stack.clone(),
                ContainerType::Node(name.to_string()),
            ));
            self.nodes.insert(name.to_string(), node_attr);
        }
    }

    pub fn get(&self) -> VisualGraph {
        let mut dir = Orientation::TopToBottom;

        // Set the graph orientation based on the 'rankdir' property.
        if let Option::Some(rd) = self.main_graph.global_state.get("rankdir") {
            if rd == "LR" {
                dir = Orientation::LeftToRight;
            }
        }
        let element = GraphBuilder::get_subgraph_shape_from_attributes(
            dir,
            &self.main_graph.global_state,
        );

        let mut vg = VisualGraph::new(dir, element);

        // Keeps track of the newly created nodes and indexes them by name.
        let mut node_map = HashMap::new();
        let mut subgraph_map = HashMap::new();

        subgraph_map.insert(
            self.main_graph.name.clone(),
            (Vec::new(), SubgraphHandle::new(0)),
        );

        self.main_graph
            .get(self, &mut vg, &mut node_map, &mut subgraph_map);

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

            vg.add_edge(shape.clone(), from.1, to.1);
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

        if let Option::Some(val) = lst.get(&"label".to_string()) {
            label = val.clone();
        }

        if let Option::Some(stl) = lst.get(&"style".to_string()) {
            if stl == "dashed" {
                line_style = LineStyleKind::Dashed;
            }
        }

        if let Option::Some(x) = lst.get(&"color".to_string()) {
            color = x.clone();
            color = Self::normalize_color(color);
        }

        if let Option::Some(pw) = lst.get(&"penwidth".to_string()) {
            if let Result::Ok(x) = pw.parse::<usize>() {
                line_width = x;
            } else {
                #[cfg(feature = "log")]
                log::info!("Can't parse integer \"{}\"", pw);
            }
        }

        if let Option::Some(fx) = lst.get(&"fontsize".to_string()) {
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
        let mut label = default_name.to_string();
        let mut edge_color = String::from("black");
        let mut fill_color = String::from("white");
        let mut font_size: usize = 14;
        let mut line_width: usize = 1;
        let mut make_xy_same = false;
        let mut rounded_corder_value = 0;

        if let Option::Some(val) = lst.get(&"label".to_string()) {
            label = val.clone();
        }

        let mut shape = ShapeKind::Circle(label.clone());

        // Set the shape.
        if let Option::Some(val) = lst.get(&"shape".to_string()) {
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
                    shape = record_builder(&label);
                }
                "Mrecord" => {
                    rounded_corder_value = 15;
                    shape = record_builder(&label);
                }
                _ => shape = ShapeKind::Circle(label),
            }
        }

        if let Option::Some(x) = lst.get(&"color".to_string()) {
            edge_color = x.clone();
            edge_color = Self::normalize_color(edge_color);
        }

        if let Option::Some(style) = lst.get(&"style".to_string()) {
            if style == "filled" && !lst.contains_key("fillcolor") {
                fill_color = "lightgray".to_string();
            }
        }

        if let Option::Some(x) = lst.get(&"fillcolor".to_string()) {
            fill_color = x.clone();
            fill_color = Self::normalize_color(fill_color);
        }

        if let Option::Some(fx) = lst.get(&"fontsize".to_string()) {
            if let Result::Ok(x) = fx.parse::<usize>() {
                font_size = x;
            } else {
                #[cfg(feature = "log")]
                log::info!("Can't parse integer \"{}\"", fx);
            }
        }

        if let Option::Some(pw) = lst.get(&"width".to_string()) {
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

    fn get_subgraph_shape_from_attributes(
        dir: Orientation,
        lst: &PropertyList,
    ) -> Element {
        let mut edge_color = String::from("black");
        let mut fill_color = String::from("white");
        let mut font_size: usize = 14;
        let mut line_width: usize = 1;

        if let Option::Some(x) = lst.get(&"color".to_string()) {
            edge_color = x.clone();
            edge_color = Self::normalize_color(edge_color);
        }

        if let Option::Some(style) = lst.get(&"style".to_string()) {
            if style == "filled" && !lst.contains_key("fillcolor") {
                fill_color = "lightgray".to_string();
            }
        }

        if let Option::Some(x) = lst.get(&"fillcolor".to_string()) {
            fill_color = x.clone();
            fill_color = Self::normalize_color(fill_color);
        }

        if let Option::Some(fx) = lst.get(&"fontsize".to_string()) {
            if let Result::Ok(x) = fx.parse::<usize>() {
                font_size = x;
            } else {
                #[cfg(feature = "log")]
                log::info!("Can't parse integer \"{}\"", fx);
            }
        }

        if let Option::Some(pw) = lst.get(&"width".to_string()) {
            if let Result::Ok(x) = pw.parse::<usize>() {
                line_width = x;
            } else {
                #[cfg(feature = "log")]
                log::info!("Can't parse integer \"{}\"", pw);
            }
        }

        let look = StyleAttr::new(
            Color::fast(&edge_color),
            line_width,
            Option::Some(Color::fast(&fill_color)),
            0,
            font_size,
        );
        let mut label = None;
        if let Option::Some(val) = lst.get(&"label".to_string()) {
            label = Some(val.to_string());
        }
        Element::create_subgraph(dir, label, &look)
    }
}
