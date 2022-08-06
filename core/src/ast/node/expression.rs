use crate::ast::node::types::Type;
use ethers_solc::artifacts::ast::{NodeType, SourceLocation};

pub struct Expression {
    argument_types: String,
    id: u64,
    name: String,
    node_type: NodeType,
    overloaded_declarations: Vec<String>,
    referenced_declaration: u64,
    src: SourceLocation,
    type_descriptions: Type,
}
