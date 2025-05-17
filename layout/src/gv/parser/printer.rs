//! A collection of methods for printing the AST.

use super::ast::{self, DotString};

fn print_node_id(n: &ast::NodeId, indent: usize) {
    print!("{}", " ".repeat(indent));
    if let Option::Some(port) = &n.port {
        println!("{}:{}", n.name, port);
    } else {
        println!("{}", n.name)
    }
}
fn print_arrow(k: &ast::ArrowKind, indent: usize) {
    print!("{}", " ".repeat(indent));
    match k {
        ast::ArrowKind::Arrow => {
            println!("->");
        }
        ast::ArrowKind::Line => {
            println!("--");
        }
    }
}
fn print_attribute(a: &str, b: &DotString, indent: usize, i: usize) {
    print!("{}", " ".repeat(indent));
    println!("{})\"{}\" = \"{}\"", i, a, b);
}
fn print_attribute_list(ll: &ast::AttributeList, indent: usize) {
    for (i, att) in ll.list.iter().enumerate() {
        print_attribute(&att.0, &att.1, indent, i);
    }
}
fn print_edge(e: &ast::EdgeStmt, indent: usize) {
    print_node_id(&e.from, indent + 1);
    for dest in &e.to {
        print_arrow(&dest.1, indent + 1);
        print_node_id(&dest.0, indent + 1);
    }
    print_attribute_list(&e.list, indent + 1);
}
fn print_node(n: &ast::NodeStmt, indent: usize) {
    print!("Node {}", " ".repeat(indent));
    print_node_id(&n.id, indent + 1);
    print_attribute_list(&n.list, indent + 1);
}
fn print_att(att: &ast::AttrStmt, indent: usize) {
    print!("{}", " ".repeat(indent));

    match att.target {
        ast::AttrStmtTarget::Graph => {
            println!("Attribute Graph:");
        }
        ast::AttrStmtTarget::Node => {
            println!("Attribute Node:");
        }
        ast::AttrStmtTarget::Edge => {
            println!("Attribute Edge:");
        }
    }
    print_attribute_list(&att.list, indent + 1);
}

fn print_stmt(stmt: &ast::Stmt, indent: usize) {
    match stmt {
        ast::Stmt::Edge(e) => {
            print_edge(e, indent);
        }
        ast::Stmt::Node(n) => {
            print_node(n, indent);
        }
        ast::Stmt::Attribute(a) => {
            print_att(a, indent);
        }
        ast::Stmt::SubGraph(g) => {
            print_graph(g, indent);
        }
    }
}

fn print_graph(graph: &ast::Graph, indent: usize) {
    print!("{}", " ".repeat(indent));
    println!("Graph: {}", graph.name);
    for stmt in &graph.list.list {
        print_stmt(stmt, indent + 1);
    }
}

pub fn dump_ast(graph: &ast::Graph) {
    print_graph(graph, 0);
}
