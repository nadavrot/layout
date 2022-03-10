//! A module that contains everything that has to do with handling the GraphViz
//! file format (parsing, building a compatible graph, etc.)

pub mod builder;
pub mod parser;
pub mod record;

pub use builder::GraphBuilder;
pub use parser::lexer::Lexer;
pub use parser::lexer::Token;
pub use parser::printer::dump_ast;
pub use parser::DotParser;
