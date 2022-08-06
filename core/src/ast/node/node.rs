use crate::ast::node::loops::{Condition, InitializationExpression, LoopExpression};
use ethers_solc::artifacts::ast::{NodeType, SourceLocation};

pub struct Node {
    pub id: Option<usize>,
    pub node_type: NodeType,
    pub src: SourceLocation,
    pub nodes: Vec<Node>,
    pub body: Option<Box<Node>>,
    pub other: Box<Specific>,
}

pub enum Specific {
    ForStatement {
        condition: Condition,
        initialization_expression: InitializationExpression,
        loop_expression: LoopExpression,
    },
    FunctionCall {},
}
