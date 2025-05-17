//! An AST that represents the GraphViz file format.

// "first : <f0>"
#[derive(Debug, Clone)]
pub struct NodeId {
    pub name: String,
    pub port: Option<String>,
}
impl NodeId {
    pub fn new(name: &str, port: &Option<String>) -> Self {
        Self {
            name: name.to_string(),
            port: port.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum DotString {
    String(String),
    HtmlString(String),
}

impl std::fmt::Display for DotString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::String(x) => write!(f, "{}", x),
            Self::HtmlString(_) => write!(f, "htmlstring"),
        }
    }
}

// [a=b; c=d; ... ]
#[derive(Debug, Clone)]
pub struct AttributeList {
    pub list: Vec<(String, DotString)>,
}

impl AttributeList {
    pub fn new() -> Self {
        Self { list: Vec::new() }
    }
    pub fn add_attr_str(&mut self, from: &str, to: &str) {
        self.list
            .push((from.to_string(), DotString::String(to.to_string())));
    }

    pub fn add_attr_html(&mut self, from: &str, to: &str) {
        self.list
            .push((from.to_string(), DotString::HtmlString(to.to_string())));
    }

    pub fn iter(&self) -> std::slice::Iter<(String, DotString)> {
        self.list.iter()
    }
}

impl Default for AttributeList {
    fn default() -> Self {
        Self::new()
    }
}

// (graph | node | edge)
#[derive(Debug, Clone)]
pub enum AttrStmtTarget {
    Graph,
    Node,
    Edge,
}
// (graph | node | edge) [ ... ]
#[derive(Debug, Clone)]
pub struct AttrStmt {
    pub target: AttrStmtTarget,
    pub list: AttributeList,
}

impl AttrStmt {
    pub fn new(target: AttrStmtTarget, list: AttributeList) -> Self {
        Self { target, list }
    }
}

// node-name [ ... ]
#[derive(Debug, Clone)]
pub struct NodeStmt {
    pub id: NodeId,
    pub list: AttributeList,
}

impl NodeStmt {
    pub fn new(id: NodeId) -> Self {
        Self {
            id,
            list: AttributeList::new(),
        }
    }
    pub fn new_with_list(id: NodeId, list: AttributeList) -> Self {
        Self { id, list }
    }
}

// (-> | -- )
#[derive(Debug, Clone)]
pub enum ArrowKind {
    Arrow,
    Line,
}

// a -> b -> c [...]
#[derive(Debug, Clone)]
pub struct EdgeStmt {
    pub from: NodeId,
    pub to: Vec<(NodeId, ArrowKind)>,
    pub list: AttributeList,
}

impl EdgeStmt {
    pub fn new(from: NodeId) -> Self {
        Self {
            from,
            to: Vec::new(),
            list: AttributeList::new(),
        }
    }

    pub fn insert(&mut self, n: NodeId, ak: ArrowKind) {
        self.to.push((n, ak));
    }
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Edge(EdgeStmt),
    Node(NodeStmt),
    Attribute(AttrStmt),
    SubGraph(Graph),
}

// { ... }
#[derive(Debug, Clone)]
pub struct StmtList {
    pub list: Vec<Stmt>,
}

impl StmtList {
    pub fn new() -> Self {
        Self { list: Vec::new() }
    }
}

impl Default for StmtList {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct Graph {
    pub name: String,
    pub list: StmtList,
}

impl Graph {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            list: StmtList::new(),
        }
    }
}
