use crate::ast::node::{
    declaration::{Literal, VariableDeclaration},
    expression::Expression,
    types::Type,
};
use ethers_solc::artifacts::ast::{NodeType, SourceLocation};

pub struct Condition {
    argument_types: String,
    common_type: Type,
    id: u64,
    is_constant: bool,
    is_lvalue: bool,
    is_pure: bool,
    lvalue_requested: bool,
    left_expression: Expression,
    node_type: NodeType,
    operator: String,
    right_expression: Expression,
    src: SourceLocation,
    type_descriptions: Type,
}

pub struct InitializationExpression {
    assignments: Vec<u64>,
    declarations: Vec<VariableDeclaration>,
    id: u64,
    initial_value: Literal,
    node_type: NodeType,
    src: SourceLocation,
}

pub struct LoopExpression {
    expression: Expression,
    id: u64,
    node_type: NodeType,
    src: SourceLocation,
}
