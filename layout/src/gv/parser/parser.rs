use super::ast;
use super::lexer::Lexer;
use super::lexer::Token;

#[derive(Debug, Clone)]
pub struct DotParser {
    lexer: Lexer,
    tok: Token,
}

/// Creates an error from the string \p str.
fn to_error<T>(str: &str) -> Result<T, String> {
    Result::Err(str.to_string())
}

impl DotParser {
    pub fn new(input: &str) -> Self {
        let chars: Vec<char> = input.chars().collect();
        Self {
            lexer: Lexer::new(chars),
            tok: Token::Colon,
        }
    }

    pub fn print_error(&self) {
        self.lexer.print_error();
    }

    pub fn lex(&mut self) {
        match self.tok {
            Token::Error(_) => {
                panic!("can't parse after error");
            }
            Token::EOF => {
                panic!("can't parse after EOF");
            }
            _ => {
                // Lex the next token.
                self.tok = self.lexer.next_token();
            }
        }
    }
    pub fn lex_html(&mut self) {
        match self.tok {
            Token::Error(_) => {
                panic!("can't parse after error");
            }
            Token::EOF => {
                panic!("can't parse after EOF");
            }
            _ => {
                // Lex the next token.
                self.tok = self.lexer.next_token_html();
            }
        }
    }

    // graph : [ strict ] (graph | digraph) [ ID ] '{' stmt_list '}'
    //subgraph : [ subgraph [ ID ] ] '{' stmt_list '}'
    pub fn parse_graph(
        &mut self,
        is_subgraph: bool,
    ) -> Result<ast::Graph, String> {
        let mut graph = ast::Graph::new("");

        // Handle the subgraph structure.
        if is_subgraph {
            // Consume the 'subgraph' keyword.
            if let Token::SubgraphKW = self.tok.clone() {
                self.lex();
            } else {
                return to_error("Expected 'subgraph'");
            }

            // Consume the optional graph name.
            if let Token::Identifier(name) = self.tok.clone() {
                graph.name = name;
                self.lex();
            }

            if let Token::OpenBrace = self.tok.clone() {
                self.lex();
            } else {
                return to_error("Expected '{'");
            }
            graph.list = self.parse_stmt_list()?;
            return Result::Ok(graph);
        }

        // Consume the 'strict' keyword.
        if let Token::StrictKW = self.tok.clone() {
            self.lex();
        }

        match self.tok {
            Token::GraphKW => {
                self.lex();
            }
            Token::DigraphKW => {
                self.lex();
            }
            Token::SubgraphKW => {
                self.lex();
            }
            _ => {
                return to_error("Expected (graph|digraph)");
            }
        }

        // Consume the optional graph name.
        if let Token::Identifier(name) = self.tok.clone() {
            graph.name = name;
            self.lex();
        }

        if let Token::OpenBrace = self.tok.clone() {
            self.lex();
        } else {
            return to_error("Expected '{'");
        }
        graph.list = self.parse_stmt_list()?;
        Result::Ok(graph)
    }
    // stmt_list : [ stmt [ ';' ] stmt_list ]
    pub fn parse_stmt_list(&mut self) -> Result<ast::StmtList, String> {
        let mut lst = ast::StmtList::new();

        loop {
            if let Token::Semicolon = self.tok.clone() {
                // Consume the semicolon.
                self.lex();
            }

            if let Token::CloseBrace = self.tok.clone() {
                // Consume the '}' and exit.
                self.lex();
                return Result::Ok(lst);
            }
            let stmt = self.parse_stmt()?;
            lst.list.push(stmt);
        }
    }
    // stmt : node_stmt | edge_stmt | attr_stmt | ID '=' ID | subgraph
    pub fn parse_stmt(&mut self) -> Result<ast::Stmt, String> {
        match self.tok {
            Token::Identifier(_) => {
                let id0 = self.parse_node_id()?;
                match self.tok {
                    Token::ArrowLine => {
                        let es = self.parse_edge_stmt(id0)?;
                        Result::Ok(ast::Stmt::Edge(es))
                    }
                    Token::ArrowRight => {
                        let es = self.parse_edge_stmt(id0)?;
                        Result::Ok(ast::Stmt::Edge(es))
                    }
                    Token::Equal => {
                        let es = self.parse_attribute_stmt(id0)?;
                        Result::Ok(ast::Stmt::Attribute(es))
                    }
                    Token::Identifier(_) => {
                        let ns = ast::NodeStmt::new(id0);
                        let ns = ast::Stmt::Node(ns);
                        Result::Ok(ns)
                    }
                    Token::Semicolon => {
                        self.lex();
                        let ns = ast::NodeStmt::new(id0);
                        let ns = ast::Stmt::Node(ns);
                        Result::Ok(ns)
                    }
                    Token::CloseBrace => {
                        let ns = ast::NodeStmt::new(id0);
                        let ns = ast::Stmt::Node(ns);
                        Result::Ok(ns)
                    }
                    Token::OpenBracket => {
                        let al = self.parse_attr_list()?;
                        let ns = ast::NodeStmt::new_with_list(id0, al);
                        let ns = ast::Stmt::Node(ns);
                        Result::Ok(ns)
                    }
                    _ => to_error("Unsupported token"),
                }
            }
            Token::SubgraphKW => {
                let subgraph = self.parse_graph(true)?;
                let ns = ast::Stmt::SubGraph(subgraph);
                Result::Ok(ns)
            }
            //attr_stmt : (graph | node | edge) attr_list
            Token::GraphKW => {
                self.lex();
                let list = self.parse_attr_list()?;
                let atts = ast::AttrStmt::new(ast::AttrStmtTarget::Graph, list);
                Result::Ok(ast::Stmt::Attribute(atts))
            }
            Token::NodeKW => {
                self.lex();
                let list = self.parse_attr_list()?;
                let atts = ast::AttrStmt::new(ast::AttrStmtTarget::Node, list);
                Result::Ok(ast::Stmt::Attribute(atts))
            }
            Token::EdgeKW => {
                self.lex();
                let list = self.parse_attr_list()?;
                let atts = ast::AttrStmt::new(ast::AttrStmtTarget::Edge, list);
                Result::Ok(ast::Stmt::Attribute(atts))
            }

            Token::OpenBrace => {
                // Handle anonymous scopes:
                self.lex();
                let mut graph = ast::Graph::new("anonymous");
                graph.list = self.parse_stmt_list()?;
                Result::Ok(ast::Stmt::SubGraph(graph))
            }

            _ => to_error("Unknown token"),
        }
    }
    //attr_list : '[' [ a_list ] ']' [ attr_list ]
    pub fn parse_attr_list(&mut self) -> Result<ast::AttributeList, String> {
        let mut lst = ast::AttributeList::new();

        if let Token::OpenBracket = self.tok.clone() {
            self.lex();
        } else {
            return to_error("Expected '['");
        }

        while !matches!(self.tok, Token::CloseBracket) {
            let prop: String;

            if let Token::Identifier(id) = self.tok.clone() {
                prop = id;
                // Consume the property name.
                self.lex();
            } else {
                return to_error("Expected property name");
            }

            if let Token::Equal = self.tok.clone() {
                // Consume the '='.
                self.lex();
            } else {
                return to_error("Expected '='");
            }

            if let Token::HtmlStart = self.tok.clone() {
                if prop == "label" {
                    let html = self.parse_html_string()?;
                    lst.add_attr_html(&prop, &html);
                    // self.lexer.mode = super::lexer::LexerMode::Normal;
                    if let Token::HtmlEnd = self.tok.clone() {
                        self.lex();
                    } else {
                        return to_error(
                            format!("Expected '>', found {:?}", self.tok)
                                .as_str(),
                        );
                    }
                }
            } else if let Token::Identifier(value) = self.tok.clone() {
                lst.add_attr_str(&prop, &value);
                // Consume the value name.
                self.lex();
            } else {
                return to_error(
                    format!(
                        "Expected value after assignment, found {:?}",
                        self.tok
                    )
                    .as_str(),
                );
            }

            // Skip semicolon.
            if let Token::Semicolon = self.tok.clone() {
                self.lex()
            }
            // Skip commas.
            if let Token::Comma = self.tok.clone() {
                self.lex()
            }
        }
        if let Token::CloseBracket = self.tok.clone() {
            self.lex();
        } else {
            return to_error("Expected ']'");
        }
        Result::Ok(lst)
    }
    // Parses a string that is inside a HTML tag.
    pub fn parse_html_string(&mut self) -> Result<String, String> {
        self.lex_html();
        if let Token::Identifier(s) = self.tok.clone() {
            self.lex();
            Ok(s)
        } else {
            to_error("Expected a string")
        }
    }

    fn is_edge_token(&self) -> bool {
        matches!(self.tok, Token::ArrowLine | Token::ArrowRight)
    }

    // ID '=' ID
    pub fn parse_attribute_stmt(
        &mut self,
        id: ast::NodeId,
    ) -> Result<ast::AttrStmt, String> {
        let mut lst = ast::AttributeList::new();

        if id.port.is_some() {
            return to_error("Can't assign into a port");
        }

        if let Token::Equal = self.tok.clone() {
            self.lex();
        } else {
            return to_error("Expected '='");
        }

        if let Token::Identifier(val) = self.tok.clone() {
            lst.add_attr_str(&id.name, &val);
            self.lex();
        } else {
            return to_error("Expected identifier.");
        }

        Result::Ok(ast::AttrStmt::new(ast::AttrStmtTarget::Graph, lst))
    }

    //edge_stmt : (node_id | subgraph) edgeRHS [ attr_list ]
    pub fn parse_edge_stmt(
        &mut self,
        id: ast::NodeId,
    ) -> Result<ast::EdgeStmt, String> {
        let mut es = ast::EdgeStmt::new(id);

        while self.is_edge_token() {
            let ak = match self.tok {
                Token::ArrowLine => ast::ArrowKind::Line,
                Token::ArrowRight => ast::ArrowKind::Arrow,
                _ => {
                    return to_error("Expected '->' or '--' ");
                }
            };
            // Consume the arrow.
            self.lex();
            let id = self.parse_node_id()?;
            es.insert(id, ak);
        }
        // Parse the optional attribute list.
        if let Token::OpenBracket = self.tok.clone() {
            es.list = self.parse_attr_list()?;
        }

        Result::Ok(es)
    }

    //node_id : ID [ port ]
    pub fn parse_node_id(&mut self) -> Result<ast::NodeId, String> {
        let node_name: String;
        if let Token::Identifier(name) = self.tok.clone() {
            node_name = name;
            // Consume the value name.
            self.lex();
        } else {
            return to_error("port");
        }

        if let Token::Colon = self.tok.clone() {
            // Consume the colon.
            self.lex();
            if let Token::Identifier(port) = self.tok.clone() {
                // Consume the port name.
                self.lex();
                return Result::Ok(ast::NodeId::new(&node_name, &Some(port)));
            } else {
                return to_error("Expected a port name");
            }
        }
        Result::Ok(ast::NodeId::new(&node_name, &None))
    }

    /// Parses dot files, as specified here:
    /// <https://graphviz.org/doc/info/lang.html>
    pub fn process(&mut self) -> Result<ast::Graph, String> {
        self.lex();
        let result = self.parse_graph(false)?;
        if let Token::EOF = self.tok {
            return Result::Ok(result);
        }
        to_error("Unexpected content at the end of the file.")
    }
}
